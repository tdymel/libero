//! The editor's toolbar as drawn: block type and language menus, the buttons a narrow bar
//! moves into More, and the slot that skips redraws while its look holds.

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::model::{BlockKind, Builtin, CommandName, Keymap};
use super::toolbar::{Group, Metrics, OVERFLOW, RichTextTool, Tool, hidden_beside_language};
use crate::{
    components::{
        buttons::{ActionIcon, Button, Toolbar, ToolbarGroup, ToolbarSeparator},
        common::{Glyph, HtmlTag, coarse_hit_area_sx},
        data_display::Icon,
        layout::use_box,
        overlay::{Menu, MenuEntry, MenuItem, MenuState, use_menu},
        typography::Language,
    },
    context::IconSlot,
    hooks::{ElementHandle, listener, use_element},
    localization::RichTextEditorLabels,
    platform::ElementApi,
    sx::{StaticSx, sx},
    theme::PaperDefaults,
};

/// Sticky, so the bar stays in reach while the caret is far down (Mantine's sticky toolbar).
/// The gap keeps the focused frame's ring off a pressed button.
static BAR_SX: StaticSx = StaticSx::new(|| {
    sx().position("sticky")
        .top("0")
        .z_index("1")
        .and(PaperDefaults::background_sx())
        .margin_bottom("xs")
        // A 44px press under a finger, its reach capped by the toolbar's gap; `Button` grows itself.
        .selector(
            "& [role=toolbar] button",
            sx().and(coarse_hit_area_sx("::before")),
        )
});

/// What [`use_bar`] draws from; `run_command` is the editor's runner for every command.
pub(super) struct Bar<R> {
    pub run_command: R,
    pub words: RichTextEditorLabels,
    pub buttons: Vec<Tool>,
    pub custom_tools: Vec<(RichTextTool, Option<bool>)>,
    pub block_kind: BlockKind,
    pub language_label: Option<String>,
    pub language_items: Vec<MenuEntry>,
    pub language_menu: MenuState,
    pub more_menu: MenuState,
    pub more_button: ElementHandle,
    pub focused: Signal<bool>,
    pub keymap: CopyValue<Keymap>,
    pub keymap_revision: CopyValue<u32>,
    /// The surface, where the toolbar's arrow keys hand focus back.
    pub element: ElementHandle,
    pub shown: bool,
}

/// Pressing a button must not take focus or the selection from the text.
pub(super) fn keep(event: MouseEvent) {
    event.prevent_default();
}

/// The first chord bound to `command`, as a tooltip shows it.
fn first_chord(keymap: &Keymap, command: impl Into<CommandName>) -> Option<String> {
    keymap
        .chords_for(command)
        .first()
        .map(|chord| chord.to_string().to_lowercase())
}

/// The code language menu: plain text, the catalog, and a typed name it does not know.
pub(super) fn language_items(
    current: &str,
    words: &RichTextEditorLabels,
    pick: impl Fn(String) + Copy + 'static,
) -> Vec<MenuEntry> {
    // A typed name the catalog does not know stays listed, so it shows as chosen.
    let unknown = (!current.is_empty() && Language::label_of(current).is_none())
        .then(|| (current.to_string(), current.to_string()));
    std::iter::once((words.plain_text.to_string(), String::new()))
        .chain(Language::catalog().map(|(label, name)| (label.into(), name.into())))
        .chain(unknown)
        .map(|(label, name): (String, String)| {
            let on = Language::label_of(&name) == Language::label_of(current)
                && (name.is_empty() == current.is_empty());
            MenuItem::new(label)
                .radio(on)
                .onselect(move |_| pick(name.clone()))
                .into()
        })
        .collect()
}

/// The toolbar, `None` when not `shown`. Its hooks run either way.
pub(super) fn use_bar<R>(bar: Bar<R>) -> Option<Element>
where
    R: Fn(CommandName, bool, bool) -> bool + Copy + 'static,
{
    let Bar {
        run_command,
        words,
        buttons,
        custom_tools,
        block_kind,
        language_label,
        language_items,
        language_menu,
        more_menu,
        more_button,
        focused,
        keymap,
        keymap_revision,
        element,
        shown,
    } = bar;
    let run = move |builtin: Builtin| {
        move |_: MouseEvent| {
            run_command(builtin.into(), true, true);
        }
    };
    let block_menu = use_menu();
    let heading = |level: u8| words.heading.replace("{level}", &level.to_string());
    let block_label = match block_kind {
        BlockKind::Heading { level } => heading(level),
        BlockKind::CodeBlock { .. } => words.code_block.to_string(),
        _ => words.paragraph.to_string(),
    };
    let block_items: Vec<MenuEntry> = {
        use Builtin as B;
        let paragraph = std::iter::once((
            B::Paragraph,
            words.paragraph.to_string(),
            block_kind == BlockKind::Paragraph,
        ));
        let headings = [
            B::Heading1,
            B::Heading2,
            B::Heading3,
            B::Heading4,
            B::Heading5,
            B::Heading6,
        ]
        .into_iter()
        .zip(1u8..)
        .map(|(builtin, level)| {
            (
                builtin,
                heading(level),
                block_kind == BlockKind::Heading { level },
            )
        });
        paragraph
            .chain(headings)
            .map(|(builtin, label, on)| {
                MenuItem::new(label)
                    .radio(on)
                    .onselect(move |_| {
                        run_command(builtin.into(), true, true);
                    })
                    .into()
            })
            .collect()
    };
    // The wrapper's width over the measured parts: the buttons a narrow bar moves into More.
    let mut bar_width = use_signal(|| None::<f64>);
    let metrics = use_signal(|| None::<Metrics>);
    let (bold, italic, block_button) = (use_element(), use_element(), use_element());
    let (language_button, link_button) = (use_element(), use_element());
    // A press keeps focus in the text; with the text unfocused a menu or dialog opener takes
    // it, so closing returns there, not to the page.
    let opener = move |handle: ElementHandle| {
        listener("onmousedown", move |_: MouseEvent| {
            if !*focused.peek() {
                let _ = handle.focus();
            }
        })
    };
    // The widest language label seen, so moving between code blocks never reflows the bar.
    let mut language_width = use_signal(|| None::<f64>);
    let (crowded, language_hidden) = bar_width().map_or((0, false), |width| {
        let measured = metrics().unwrap_or_default();
        let language = match language_label {
            Some(_) => language_width().unwrap_or(measured.block_type),
            None => 0.0,
        };
        hidden_beside_language(
            width,
            Metrics {
                language,
                ..measured
            },
            custom_tools.len(),
        )
    });
    // Reads start here, not in the task: Blitz fails a read made inside one.
    let measure = move || {
        let reads = (
            bold.dimensions(),
            bold.client_offset(),
            italic.client_offset(),
            block_button.dimensions(),
        );
        spawn(async move {
            let (Ok(icon), Ok((from, _)), Ok((to, _)), Ok(block)) =
                (reads.0.await, reads.1.await, reads.2.await, reads.3.await)
            else {
                return;
            };
            if icon.width <= 0.0 {
                return;
            }
            let mut metrics = metrics;
            let known = metrics.peek().unwrap_or_default();
            // Bold and Italic on two rows of a wrapped bar give no gap.
            let gap = (to - from).abs() - icon.width;
            let next = Some(Metrics {
                icon: icon.width,
                gap: if (0.0..icon.width).contains(&gap) {
                    gap
                } else {
                    known.gap
                },
                // The widest label seen, so moving the caret never reflows the bar.
                block_type: block
                    .width
                    .max(metrics.peek().map_or(0.0, |m| m.block_type)),
                language: 0.0,
            });
            if *metrics.peek() != next {
                metrics.set(next);
            }
        });
    };
    let hidden = &OVERFLOW[..crowded];
    // A bar too narrow even for the language button lists it first, its menu as a submenu.
    let language_entry = language_label
        .as_deref()
        .filter(|_| language_hidden)
        .map(|language| {
            MenuItem::new(format!("{}: {language}", words.language))
                .submenu(language_items.clone())
                .into()
        });
    let more_items: Vec<MenuEntry> = language_entry
        .into_iter()
        .chain(
            buttons
                .iter()
                .filter(|tool| hidden.contains(&tool.builtin))
                .map(|tool| {
                    let builtin = tool.builtin;
                    let item = MenuItem::new(tool.label)
                        // Bare, a leading glyph is 0px wide in the menu's slot.
                        .leading(rsx! {
                            Icon { variant: "standard", size: "sm", color: "inherit",
                                Glyph { slot: tool.slot, icon: tool.icon }
                            }
                        })
                        .disabled(tool.disabled)
                        .onselect(move |_| {
                            run_command(builtin.into(), true, true);
                        });
                    match tool.selected {
                        Some(on) => item.checkbox(on),
                        None => item,
                    }
                    .into()
                }),
        )
        .collect();
    let shown_in = |group: Group| -> Vec<Tool> {
        buttons
            .iter()
            .filter(|tool| tool.group == group && !hidden.contains(&tool.builtin))
            .copied()
            .collect()
    };
    let (marks, block_tools, history) = (
        shown_in(Group::Marks),
        shown_in(Group::Blocks),
        shown_in(Group::History),
    );
    // A theme or a new label resizes Bold or the block type button: the parts are read again.
    let sized = move |handle: ElementHandle| {
        vec![
            listener("onmounted", handle.mount()),
            listener("onresize", move |_: Event<ResizeData>| measure()),
        ]
    };
    let button = move |tool: Tool| {
        let attributes = match tool.builtin {
            Builtin::Bold => sized(bold),
            Builtin::Italic => vec![listener("onmounted", italic.mount())],
            Builtin::Link => vec![
                listener("onmounted", link_button.mount()),
                opener(link_button),
            ],
            _ => Vec::new(),
        };
        let shortcut = first_chord(&keymap.peek(), tool.builtin);
        rsx! {
            ActionIcon {
                key: "{tool.label}",
                attributes,
                aria_label: tool.label,
                tooltip: true,
                shortcut,
                selected: tool.selected,
                disabled: tool.disabled,
                onclick: run(tool.builtin),
                Glyph { slot: tool.slot, icon: tool.icon }
            }
        }
    };
    let custom_button = move |tool: RichTextTool, selected: Option<bool>| {
        let shortcut = first_chord(&keymap.peek(), tool.command.clone());
        let command = tool.command.clone();
        rsx! {
            ActionIcon {
                key: "{tool.command}",
                aria_label: tool.label,
                tooltip: true,
                shortcut,
                selected,
                onclick: move |_| {
                    run_command(command.clone(), true, true);
                },
                {tool.icon}
            }
        }
    };
    let onresize = move |event: Event<ResizeData>| {
        if let Ok(size) = event.get_content_box_size()
            && *bar_width.peek() != Some(size.width)
        {
            bar_width.set(Some(size.width));
        }
        if metrics.peek().is_none() {
            measure();
        }
    };
    let mut block_attributes = block_menu.a11y_attributes();
    block_attributes.extend(sized(block_button));
    block_attributes.push(opener(block_button));
    let mut language_attributes = language_menu.a11y_attributes();
    language_attributes.extend([
        listener("onmounted", language_button.mount()),
        opener(language_button),
    ]);
    let mut more_attributes = more_menu.a11y_attributes();
    more_attributes.extend([
        listener("onmounted", more_button.mount()),
        opener(more_button),
    ]);
    language_attributes.push(listener("onresize", move |event: Event<ResizeData>| {
        if let Ok(size) = event.get_border_box_size()
            && size.width > language_width.peek().unwrap_or(0.0)
        {
            language_width.set(Some(size.width));
        }
    }));
    let look = ToolbarLook {
        words,
        tools: buttons
            .iter()
            .map(|tool| (tool.builtin, tool.selected, tool.disabled))
            .collect(),
        custom_tools: custom_tools.clone(),
        block_kind: block_kind.clone(),
        crowded,
        language_hidden,
        menus: [
            block_menu.a11y_attributes(),
            language_menu.a11y_attributes(),
            more_menu.a11y_attributes(),
        ],
        keymap: *keymap_revision.peek(),
    };
    let bar_box = use_box().framework_sx(&BAR_SX).prepare();
    shown.then(|| {
        // A new count draws a new bar: diffing out the history group beside More panicked (4771).
        let toolbar = rsx! {
            Toolbar { key: "{crowded}-{language_hidden}", "aria-label": words.toolbar, focus_from: element, onmousedown: keep,
                ToolbarGroup { "aria-label": words.marks,
                    for tool in marks { {button(tool)} }
                }
                ToolbarSeparator {}
                ToolbarGroup { "aria-label": words.blocks,
                    Menu { state: block_menu, items: block_items,
                        Button {
                            attributes: block_attributes,
                            "aria-label": "{words.block_type}: {block_label}",
                            variant: "standard",
                            color: "ink",
                            "{block_label}"
                        }
                    }
                    if let Some(language) = language_label.as_deref().filter(|_| !language_hidden) {
                        Menu { state: language_menu, items: language_items,
                            Button {
                                attributes: language_attributes,
                                "aria-label": "{words.language}: {language}",
                                variant: "standard",
                                color: "ink",
                                "{language}"
                            }
                        }
                    }
                    for tool in block_tools { {button(tool)} }
                }
                if !custom_tools.is_empty() {
                    ToolbarSeparator {}
                    ToolbarGroup { "aria-label": words.custom_tools,
                        for (tool, selected) in custom_tools {
                            {custom_button(tool, selected)}
                        }
                    }
                }
                if !history.is_empty() {
                    ToolbarSeparator {}
                    ToolbarGroup { "aria-label": words.history,
                        for tool in history { {button(tool)} }
                    }
                }
                if !more_items.is_empty() {
                    Menu { state: more_menu, items: more_items,
                        ActionIcon { attributes: more_attributes, aria_label: words.more,
                            Glyph { slot: IconSlot::More, icon: lucide::ellipsis_vertical::outlined }
                        }
                    }
                }
            }
        };
        let bar = bar_box
            .event("onresize", onresize)
            .render(HtmlTag::Div, vec![], toolbar);
        rsx! {
            ToolbarSlot { look, bar }
        }
    })
}

/// Everything the toolbar shows. Its handlers read only stable handles, so a bar drawn from
/// an equal look is still current.
#[derive(Clone, PartialEq)]
struct ToolbarLook {
    words: RichTextEditorLabels,
    tools: Vec<(Builtin, Option<bool>, bool)>,
    custom_tools: Vec<(RichTextTool, Option<bool>)>,
    /// The block type and code language menus' radios and labels.
    block_kind: BlockKind,
    crowded: usize,
    language_hidden: bool,
    menus: [Vec<Attribute>; 3],
    keymap: u32,
}

#[derive(Props, Clone)]
struct ToolbarSlotProps {
    look: ToolbarLook,
    bar: Element,
}

/// Compares the look alone: `bar` never compares equal.
impl PartialEq for ToolbarSlotProps {
    fn eq(&self, other: &Self) -> bool {
        self.look == other.look
    }
}

/// The toolbar, skipped while its look holds: typing redrew every button and tooltip (todo 2002).
#[component]
fn ToolbarSlot(props: ToolbarSlotProps) -> Element {
    props.bar
}
