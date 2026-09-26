//! [`RichTextEditor`]: the model's doc rendered into a `contenteditable`, every edit
//! routed through the model (web `beforeinput`), composition reconciled after it ends.

use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::dialogs::{LinkArgs, LinkChoice, LinkDialog, announcement, shortcut_rows};
use super::handle::{RichTextHandle, Status};
use super::input::{Intent, intent, text_diff};
use super::model::{
    Action, BlockKind, Builtin, CommandName, Commands, Doc, Editor, EditorState, Inline, KeyPress,
    Keymap, Mark, MarkKind, NodeKey, NodeRegistry, Position, Record, Selection, UndoStack,
};
use super::node_view::NodeViews;
use super::offsets::{to_dom, to_model};
use super::render::{RenderCtx, blocks};
use super::surface::{ROOT_ATTR, Report, Surface};
use super::toolbar::{Group, OVERFLOW, Tool, hidden as overflow_count, tools};
use crate::{
    components::{
        accessibility::use_announcer,
        buttons::{ActionIcon, Button, Toolbar, ToolbarGroup, ToolbarSeparator},
        common::{Glyph, HtmlTag, Input},
        form::{field_props, use_bound, use_field, use_field_frame},
        layout::use_box,
        overlay::{Menu, MenuEntry, MenuItem, Shortcut, ShortcutHelp, use_menu},
    },
    context::IconSlot,
    hooks::{
        HistoryHandle, ModalScope, UndoHistory, use_element, use_history, use_localization,
        use_modal, use_theme,
    },
    platform::mod_is_meta,
    sx::{StaticSx, sx},
    theme::{ANCHOR_COLOR, CODE_FONT_FAMILY, ColorCss, ColorShade},
};

impl UndoStack for HistoryHandle<EditorState> {
    fn present(&self) -> Rc<EditorState> {
        self.read(|history| history.present().clone())
    }

    fn push(&mut self, state: EditorState) {
        HistoryHandle::push(self, state);
    }

    fn merge(&mut self, state: EditorState) {
        HistoryHandle::merge(self, state);
    }

    fn seal(&mut self) {
        HistoryHandle::seal(self);
    }

    fn undo(&mut self) -> bool {
        HistoryHandle::undo(self)
    }

    fn redo(&mut self) -> bool {
        HistoryHandle::redo(self)
    }

    fn reset(&mut self, state: EditorState) {
        HistoryHandle::reset(self, state);
    }

    fn can_undo(&self) -> bool {
        self.read(|history| history.can_undo())
    }

    fn can_redo(&self) -> bool {
        self.read(|history| history.can_redo())
    }
}

type LiveEditor = Editor<HistoryHandle<EditorState>>;

/// Typing pauses longer than this start a new undo step.
const GROUP_MS: u64 = 500;

static SURFACE_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .width("100%")
        .min_height("4.5em")
        .padding("0.5rem 0.75rem")
        .outline("none")
        .white_space("pre-wrap")
        .overflow_wrap("anywhere")
        .selector(
            "& > :first-child, & > * > :first-child",
            sx().margin_top("0"),
        )
        .selector(
            "& > :last-child, & > * > :last-child",
            sx().margin_bottom("0"),
        )
        .selector("& pre", sx().font_family("monospace").margin("0"))
        // List items hold paragraphs: no paragraph gaps between bullets.
        .selector("& li > p", sx().margin("0"))
        // Inline code, links and quotes as `Code`, `Anchor` and `Blockquote` draw them.
        .selector(
            "& :not(pre) > code",
            sx().background("muted.2")
                .border_radius("4px")
                .padding("0 0.25em")
                .font_family(CODE_FONT_FAMILY.value())
                .font_size("0.875em"),
        )
        .selector("& a", sx().color(ANCHOR_COLOR.value()))
        .selector(
            "& blockquote",
            sx().margin("1em 0")
                .padding("0 0.75rem")
                .border_left(format!(
                    "2px solid {}",
                    ColorCss::MUTED.value(ColorShade::S4)
                )),
        )
        .selector(
            "& [data-fence]",
            sx().font_family("monospace")
                .color("text-dimmed")
                .user_select("none"),
        )
        .selector(
            "&[data-empty]::before",
            sx().content("attr(data-placeholder)")
                .color("text-placeholder")
                .pointer_events("none")
                .position("absolute"),
        )
});

field_props! {
    extends(div);
    pub struct RichTextEditorProps {
        /// The document. `None` leaves the editor uncontrolled.
        #[props(default, into)]
        value: Option<Doc>,
        /// Fires with the new document after every edit; selection moves do not fire.
        #[props(default)]
        onchange: Option<EventHandler<Doc>>,
        /// Rules over the document, shown on blur or submit.
        #[props(default, into)]
        validate: crate::components::form::Validators<Doc>,
        /// A path binds the editor to the surrounding `Form`.
        #[props(default, into)]
        name: crate::components::form::FieldName<Doc>,
        /// Shown while the document is one empty paragraph.
        #[props(default, into)]
        placeholder: Option<String>,
        /// Shows the formatting toolbar above the text.
        #[props(default = true)]
        toolbar: bool,
        /// Key bindings; the default binds the usual editor chords.
        #[props(default)]
        keymap: Keymap,
        /// What the keymap and toolbar run; the default holds every built-in.
        #[props(default)]
        commands: Commands,
        /// Runs commands and reads the state from your own toolbar; see [`use_rich_text_editor`](super::handle::use_rich_text_editor).
        #[props(default, into)]
        handle: Option<RichTextHandle>,
        /// Components by node name, built-ins too (not `code_block`); a custom node
        /// without one draws a plain fallback.
        #[props(default)]
        nodes: NodeViews,
        /// The caller's node types: copy writes them through their codecs, and debug
        /// builds warn about a `nodes` name it lacks.
        #[props(default, into)]
        registry: Option<NodeRegistry>,
    }
}

/// The last docs this editor emitted: a controlled parent may echo any of them late.
#[derive(Default)]
struct Echoes(VecDeque<Doc>);

impl Echoes {
    const KEPT: usize = 8;

    fn record(&mut self, doc: Doc) {
        if self.0.len() == Self::KEPT {
            self.0.pop_front();
        }
        self.0.push_back(doc);
    }

    fn clear(&mut self) {
        self.0.clear();
    }

    /// Whether `incoming` is neither the live doc nor one of ours: a reset.
    fn foreign(&self, current: &Doc, incoming: &Doc) -> bool {
        incoming != current && !self.0.contains(incoming)
    }
}

fn next_token() -> String {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed).to_string()
}

fn dom_position(editor: &LiveEditor, at: Position) -> (u64, usize) {
    let offset = editor
        .doc()
        .get(at.block)
        .map_or(0, |block| to_dom(block.inlines(), at.offset));
    (at.block.0, offset)
}

fn model_position(editor: &LiveEditor, key: u64, units: usize) -> Option<Position> {
    let block = editor
        .doc()
        .get(NodeKey(key))
        .filter(|block| block.is_leaf())?;
    Some(Position::new(block.key, to_model(block.inlines(), units)))
}

fn is_empty(doc: &Doc) -> bool {
    matches!(doc.blocks.as_slice(), [only] if only.is_leaf() && only.len() == 0 && !only.kind.is_code())
}

/// A rich text field: paragraphs, headings, lists, quotes, code blocks and marks,
/// with undo, the usual shortcuts and Markdown typing shortcuts.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{RichTextEditor, rich_text::Doc};
/// # fn app() -> Element {
/// let mut notes = use_signal(Doc::new);
/// rsx! {
///     RichTextEditor {
///         label: "Notes",
///         value: notes(),
///         onchange: move |doc| notes.set(doc),
///     }
///     pre { {notes.read().to_markdown()} }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/rich-text-editor>
#[component]
pub fn RichTextEditor(props: RichTextEditorProps) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.textarea.size);
    let radius = props.radius.copied_or(theme.textarea.radius);
    let required = props.required.unwrap_or(false);

    let bound = use_bound(&props.name, props.onchange.is_some());
    let disabled = bound.disabled(props.disabled);
    let readonly = props.readonly.unwrap_or(false);
    let value = bound.value().or_else(|| props.value.clone());
    let editable = !disabled && !readonly && crate::platform::edits_rich_text();

    let initial = value.clone().unwrap_or_default();
    let history = use_history(|| UndoHistory::new(EditorState::new(initial)), GROUP_MS);
    let mut editor = use_hook(|| CopyValue::new(Editor::with_history(history)));
    // Bumped on every change, selection included: the toolbar's state.
    let mut revision = use_signal(|| 0u32);
    // Bumped when the model moved the caret, so the DOM selection follows it.
    let mut placed = use_signal(|| (0u32, false));
    // Bumped after a composition, so the browser's own text nodes are rebuilt.
    let mut generation = use_signal(|| 0u32);
    let mut focused = use_signal(|| false);
    let mut composing = use_hook(|| CopyValue::new(false));
    // While the DOM has not caught up with a model caret, its reports are stale.
    let mut syncing = use_hook(|| CopyValue::new(false));
    let token = use_hook(next_token);
    let mut can_edit = use_hook(|| CopyValue::new(editable));
    if *can_edit.peek() != editable {
        can_edit.set(editable);
    }

    let emit = use_hook(|| CopyValue::new(None::<Rc<dyn Fn(Doc)>>));
    {
        let mut emit_slot = emit;
        let next = bound
            .emit(props.onchange)
            .map(|emit| Rc::new(emit) as Rc<dyn Fn(Doc)>);
        emit_slot.set(next);
    }

    let commands = use_hook(|| CopyValue::new(Commands::default()));
    let keymap = use_hook(|| CopyValue::new(Keymap::default()));
    if *commands.peek() != props.commands {
        let mut commands = commands;
        commands.set(props.commands.clone());
    }
    if *keymap.peek() != props.keymap {
        let mut keymap = keymap;
        keymap.set(props.keymap.clone());
    }
    let registry = use_hook(|| CopyValue::new(None::<NodeRegistry>));
    if *registry.peek() != props.registry {
        let mut registry = registry;
        registry.set(props.registry.clone());
    }
    #[cfg(debug_assertions)]
    {
        let mut checked = use_hook(|| CopyValue::new(None::<(NodeViews, NodeRegistry)>));
        if let Some(registry) = &props.registry {
            let pair = (props.nodes.clone(), registry.clone());
            if checked.peek().as_ref() != Some(&pair) {
                for name in props.nodes.unregistered(registry) {
                    crate::utils::warn(&format!(
                        "NodeViews: \"{name}\" is not a registered node; nodes of that name never reach this view."
                    ));
                }
                checked.set(Some(pair));
            }
        }
    }

    let mut echoes = use_hook(|| CopyValue::new(Echoes::default()));
    // A foreign controlled value replaces the doc; a late echo of our own does not.
    use_effect(use_reactive!(|value| {
        if let Some(doc) = value
            && echoes.peek().foreign(editor.peek().doc(), &doc)
        {
            echoes.write().clear();
            editor.write().reset(doc);
            revision += 1;
        }
    }));

    // After a model edit: tell the caller, re-render, put the DOM caret where the model has it.
    let mut changed = move |before: Option<Doc>, focus: bool| {
        if let Some(before) = before {
            let doc = editor.peek().doc().clone();
            if doc != before
                && let Some(emit) = emit.peek().clone()
            {
                echoes.write().record(doc.clone());
                emit(doc);
            }
        }
        revision += 1;
        syncing.set(true);
        let (count, _) = *placed.peek();
        placed.set((count + 1, focus));
    };

    let mut edit = move |run: &dyn Fn(&mut LiveEditor) -> bool, focus: bool| -> bool {
        let before = editor.peek().doc().clone();
        let done = run(&mut editor.write());
        if done {
            changed(Some(before), focus);
        }
        done
    };

    let words = use_localization().rich_text_editor;
    let announcer = use_announcer();
    let link_modal = use_modal(|scope: ModalScope<LinkArgs, LinkChoice>| {
        rsx! { LinkDialog { args: scope.args(), onchoose: move |choice| scope.resolve(choice) } }
    });
    let help_modal = use_modal(move |scope: ModalScope<Vec<Shortcut>>| {
        rsx! { ShortcutHelp { title: words.shortcuts, shortcuts: scope.args() } }
    });

    // Opens a view command's dialog; the caret goes back to the text when it closes.
    let open_view = move |name: &CommandName| -> bool {
        if *name == Builtin::Link.into() {
            let href = link_href(editor.peek().state());
            link_modal
                .open_with(LinkArgs { href })
                .onresult(move |choice| {
                    let mut edit = edit;
                    match choice {
                        Some(LinkChoice::Set(href)) => edit(
                            &|live| {
                                live.apply(Record::Step, |state| {
                                    state.set_link(&href).unwrap_or(false)
                                })
                            },
                            true,
                        ),
                        Some(LinkChoice::Remove) => {
                            edit(&|live| live.run(&commands.peek(), Builtin::Unlink), true)
                        }
                        None => false,
                    };
                    let mut changed = changed;
                    changed(None, true);
                });
            return true;
        }
        if *name == Builtin::Shortcuts.into() {
            help_modal
                .open_with(shortcut_rows(&keymap.peek(), &words))
                .onresult(move |_| {
                    let mut changed = changed;
                    changed(None, true);
                });
            return true;
        }
        false
    };

    // Every command goes through here: keys, the toolbar and the handle.
    let run_command = move |name: CommandName, focus: bool, announce: bool| -> bool {
        if !*can_edit.peek() {
            return false;
        }
        if matches!(commands.peek().get(&name), Some(Action::View)) {
            return open_view(&name);
        }
        let mut edit = edit;
        let ran = edit(&|live| live.run(&commands.peek(), name.clone()), focus);
        if ran
            && announce
            && let Some(message) = announcement(&name, editor.peek().state(), &words)
        {
            announcer.say(message);
        }
        ran
    };

    let handle = use_hook(|| {
        let handle = props.handle;
        if let Some(handle) = handle {
            let owner = token.clone();
            handle.attach(&owner, Rc::new(move |name| run_command(name, true, false)));
        }
        handle
    });
    {
        let token = token.clone();
        use_drop(move || {
            if let Some(handle) = handle {
                handle.detach(&token);
            }
        });
    }
    {
        let token = token.clone();
        use_effect(move || {
            let _ = revision();
            if let Some(handle) = handle {
                let live = editor.peek();
                handle.publish(
                    &token,
                    Status::of(live.state(), live.can_undo(), live.can_redo()),
                );
            }
        });
    }

    let surface = use_hook(|| {
        Surface::start(token.clone(), move |report: Report| {
            if let Some((anchor_key, anchor, head_key, head)) = report.selection {
                if *syncing.peek() || *composing.peek() {
                    return;
                }
                let selection = {
                    let live = editor.peek();
                    model_position(&live, anchor_key, anchor)
                        .zip(model_position(&live, head_key, head))
                        .map(|(anchor, head)| Selection::range(anchor, head))
                };
                if let Some(selection) = selection
                    && selection != editor.peek().state().selection
                {
                    editor.write().select(selection);
                    revision += 1;
                }
            }
            if report.synced {
                syncing.set(false);
            }
            if let Some((key, text)) = report.text {
                reconcile(editor, NodeKey(key), &text, &mut changed);
                generation += 1;
            }
            if let Some((key, at_end)) = report.code
                && *can_edit.peek()
            {
                enter_code(editor, NodeKey(key), at_end);
                changed(None, true);
            }
        })
    });

    use_effect(move || {
        let (_, focus) = placed();
        let live = editor.peek();
        let selection = live.state().selection;
        surface.select(
            dom_position(&live, selection.anchor),
            dom_position(&live, selection.head),
            focus,
        );
    });

    let _ = revision();
    let live = editor.read();
    let state = live.state();
    let caret = state.caret().block;
    let source_code = (focused() && state.block(caret).kind.is_code()).then_some(caret);
    let on_code = editable.then(|| {
        EventHandler::new(move |key: NodeKey| {
            enter_code(editor, key, true);
            focused.set(true);
            changed(None, true);
        })
    });
    let content = blocks(
        &live.doc().blocks,
        RenderCtx {
            source_code,
            on_code,
            views: &props.nodes,
        },
    );
    let empty = is_empty(live.doc());
    let mut buttons = tools(&words);
    for tool in &mut buttons {
        let mark = |kind| Some(state.is_active(kind));
        tool.selected = match tool.builtin {
            Builtin::Bold => mark(MarkKind::Bold),
            Builtin::Italic => mark(MarkKind::Italic),
            Builtin::Underline => mark(MarkKind::Underline),
            Builtin::Strike => mark(MarkKind::Strike),
            Builtin::Code => mark(MarkKind::Code),
            Builtin::Link => mark(MarkKind::Link),
            Builtin::BulletList => Some(state.list_kind() == Some(false)),
            Builtin::OrderedList => Some(state.list_kind() == Some(true)),
            Builtin::Quote => Some(state.in_quote()),
            Builtin::CodeBlock => Some(state.block_kind().is_code()),
            _ => None,
        };
        tool.disabled = match tool.builtin {
            Builtin::Undo => !live.can_undo(),
            Builtin::Redo => !live.can_redo(),
            _ => false,
        };
    }
    let block_kind = state.block_kind().clone();
    drop(live);

    let field = use_field()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(bound.check(&props.validate, value.clone()))
        .bound(&bound)
        .required(required)
        .disabled(disabled)
        .size(size)
        .radius(radius)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&props.states)
        .attributes(&props.attributes)
        .prepare();
    let frame = use_field_frame()
        .states(field.states())
        .multiline()
        .prepare();
    let control = use_box()
        .framework_sx(&SURFACE_SX)
        .focus_ring(false)
        .prepare();
    let element = use_element();
    let apple = mod_is_meta();

    let onkeydown = move |event: KeyboardEvent| {
        // Not `is_composing()`: Gboard keeps a composing region open over typed words.
        if *composing.peek() {
            return;
        }
        let modifiers = event.modifiers();
        let press = KeyPress {
            key: event.key().to_string(),
            code: event.code().to_string(),
            ctrl: modifiers.ctrl(),
            meta: modifiers.meta(),
            alt: modifiers.alt(),
            shift: modifiers.shift(),
        };
        let name = keymap.peek().command_for(&press, apple).cloned();
        if name.is_some_and(|name| run_command(name, false, true)) {
            event.prevent_default();
        }
    };

    let onbeforeinput = move |event: Event<BeforeInputData>| {
        let data = event.data();
        match intent(&data.input_type().to_string(), data.data()) {
            Intent::Pass => composing.set(true),
            Intent::Cancel => event.prevent_default(),
            Intent::Type(text) => {
                event.prevent_default();
                edit(&|live| live.type_text(&text), false);
            }
            Intent::Run(builtin) => {
                event.prevent_default();
                edit(&|live| live.run(&commands.peek(), builtin), false);
            }
        }
    };

    let onpaste = move |event: ClipboardEvent| {
        event.prevent_default();
        let Some(text) = event.data().data_transfer().get_as_text() else {
            return;
        };
        edit(
            &|live| live.apply(Record::Step, |state| paste(state, &text)),
            false,
        );
    };

    // The selection as plain text and Markdown; `false` when nothing was written.
    let copy = move |event: &ClipboardEvent| -> bool {
        let fragment = {
            let live = editor.peek();
            if live.state().selection.is_collapsed() {
                return false;
            }
            live.state().selected_doc()
        };
        let registry = registry.peek().clone().unwrap_or_default();
        let transfer = event.data().data_transfer();
        let written = transfer
            .set_data("text/plain", &fragment.plain_text_with(&registry))
            .is_ok();
        if written {
            let _ = transfer.set_data("text/markdown", &fragment.to_markdown_with(&registry));
            event.prevent_default();
        }
        written
    };
    let oncopy = move |event: ClipboardEvent| {
        copy(&event);
    };
    // The browser's own cut would edit the DOM behind the model.
    let oncut = move |event: ClipboardEvent| {
        event.prevent_default();
        if copy(&event) {
            edit(
                &|live| live.apply(Record::Step, |state| state.delete_selection()),
                false,
            );
        }
    };

    // Gboard opens composing regions over words the model typed; only text the browser
    // composed itself (`insertCompositionText`) is read back, or a lagging DOM undoes keys.
    let oncompositionend = move |_: CompositionEvent| {
        if *composing.peek() {
            composing.set(false);
            surface.read(editor.peek().state().caret().block.0);
        }
    };

    let surface_element = field
        .aria(control)
        .element(&element)
        .attr(ROOT_ATTR, token.clone())
        .attr("role", "textbox")
        .attr("aria-multiline", "true")
        .attr("aria-readonly", readonly)
        .attr("aria-placeholder", props.placeholder.clone())
        .attr("data-placeholder", props.placeholder.clone())
        .attr("data-empty", empty)
        .attr("contenteditable", if editable { "true" } else { "false" })
        .attr("spellcheck", "false")
        .event("onkeydown", editable.then_some(onkeydown))
        .event("onbeforeinput", editable.then_some(onbeforeinput))
        .event("onpaste", editable.then_some(onpaste))
        .event("oncopy", editable.then_some(oncopy))
        .event("oncut", editable.then_some(oncut))
        .event("oncompositionend", editable.then_some(oncompositionend))
        .event("onfocusin", move |_: FocusEvent| focused.set(true))
        .event("onfocusout", move |_: FocusEvent| focused.set(false))
        .render(
            HtmlTag::Div,
            props.attributes,
            rsx! {
                for generation in [generation()] {
                    div { key: "{generation}", style: "display: contents", {content.clone()} }
                }
            },
        );

    let run = move |builtin: Builtin| {
        move |_: MouseEvent| {
            run_command(builtin.into(), true, false);
        }
    };
    // Pressing a button must not take focus or the selection from the text.
    let keep = |event: MouseEvent| event.prevent_default();
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
                        run_command(builtin.into(), true, false);
                    })
                    .into()
            })
            .collect()
    };
    // Measured by the wrapper: the buttons a narrow bar moves into the More menu.
    let mut crowded = use_signal(|| 0usize);
    let more_menu = use_menu();
    let hidden = &OVERFLOW[..crowded()];
    let more_items: Vec<MenuEntry> = buttons
        .iter()
        .filter(|tool| hidden.contains(&tool.builtin))
        .map(|tool| {
            let builtin = tool.builtin;
            let item = MenuItem::new(tool.label)
                .leading(rsx! { Glyph { slot: tool.slot, icon: tool.icon } })
                .disabled(tool.disabled)
                .onselect(move |_| {
                    run_command(builtin.into(), true, false);
                });
            match tool.selected {
                Some(on) => item.checkbox(on),
                None => item,
            }
            .into()
        })
        .collect();
    let shown = |group: Group| -> Vec<Tool> {
        buttons
            .iter()
            .filter(|tool| tool.group == group && !hidden.contains(&tool.builtin))
            .copied()
            .collect()
    };
    let (marks, block_tools, history) = (
        shown(Group::Marks),
        shown(Group::Blocks),
        shown(Group::History),
    );
    let button = move |tool: Tool| {
        rsx! {
            ActionIcon {
                key: "{tool.label}",
                aria_label: tool.label,
                selected: tool.selected,
                disabled: tool.disabled,
                onclick: run(tool.builtin),
                Glyph { slot: tool.slot, icon: tool.icon }
            }
        }
    };
    let onresize = move |event: Event<ResizeData>| {
        if let Ok(size) = event.get_content_box_size() {
            let count = overflow_count(size.width);
            if *crowded.peek() != count {
                crowded.set(count);
            }
        }
    };
    let toolbar = (props.toolbar && editable).then(|| {
        rsx! {
            div { onresize,
                Toolbar { "aria-label": words.toolbar, focus_from: element, onmousedown: keep,
                    ToolbarGroup { "aria-label": words.marks,
                        for tool in marks { {button(tool)} }
                    }
                    ToolbarSeparator {}
                    ToolbarGroup { "aria-label": words.blocks,
                        Menu { state: block_menu, items: block_items,
                            Button {
                                attributes: block_menu.a11y_attributes(),
                                "aria-label": "{words.block_type}: {block_label}",
                                variant: "standard",
                                color: "ink",
                                "{block_label}"
                            }
                        }
                        for tool in block_tools { {button(tool)} }
                    }
                    if !history.is_empty() {
                        ToolbarSeparator {}
                        ToolbarGroup { "aria-label": words.history,
                            for tool in history { {button(tool)} }
                        }
                    }
                    if !more_items.is_empty() {
                        Menu { state: more_menu, items: more_items,
                            ActionIcon { attributes: more_menu.a11y_attributes(), aria_label: words.more,
                                Glyph { slot: IconSlot::More, icon: lucide::ellipsis_vertical::outlined }
                            }
                        }
                    }
                }
            }
        }
    });

    field.render(rsx! {
        {toolbar}
        {frame.render(surface_element)}
        {announcer.render()}
    })
}

/// The href of the link around the caret.
fn link_href(state: &EditorState) -> Option<String> {
    let (key, from, _) = state.link_at_caret()?;
    let mut at = 0;
    let mark = state.block(key).inlines().iter().find_map(|inline| {
        let start = at;
        at += inline.len();
        match inline {
            Inline::Text { marks, .. } if start <= from && from < at => {
                marks.get(MarkKind::Link).cloned()
            }
            _ => None,
        }
    })?;
    match mark {
        Mark::Link { href, .. } => Some(href.as_str().to_string()),
        _ => None,
    }
}

/// Plain text at the caret; each line after the first starts a new block.
fn paste(state: &mut EditorState, text: &str) -> bool {
    let text = text.replace("\r\n", "\n");
    let mut changed = state.delete_selection();
    for (index, line) in text.split('\n').enumerate() {
        if index > 0 {
            changed |= state.split_block();
        }
        changed |= state.insert_text(line);
    }
    changed
}

/// Puts the caret at the start or end of code block `key`, which then renders as source.
fn enter_code(mut editor: CopyValue<LiveEditor>, key: NodeKey, at_end: bool) {
    let at = match editor.peek().doc().get(key) {
        Some(block) if at_end => block.len(),
        Some(_) => 0,
        None => return,
    };
    editor
        .write()
        .select(Selection::caret(Position::new(key, at)));
}

/// Takes the text the browser composed into leaf `key` into the model.
fn reconcile(
    mut editor: CopyValue<LiveEditor>,
    key: NodeKey,
    dom: &str,
    changed: &mut impl FnMut(Option<Doc>, bool),
) {
    let Some(old) = editor.peek().doc().get(key).map(|block| block.text()) else {
        return;
    };
    let Some((from, to, inserted)) = text_diff(&old, dom) else {
        return;
    };
    let before = editor.peek().doc().clone();
    editor.write().apply(Record::Merge, |state| {
        state.select(Selection::range(
            Position::new(key, from),
            Position::new(key, to),
        ));
        state.delete_selection();
        state.insert_text(&inserted);
        true
    });
    changed(Some(before), false);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn late_echoes_of_typing_do_not_reset() {
        let mut editor = Editor::new(Doc::new());
        let mut echoes = Echoes::default();
        let mut sent = Vec::new();
        for c in ["a", "b", "c"] {
            editor.type_text(c);
            echoes.record(editor.doc().clone());
            sent.push(editor.doc().clone());
        }
        // The parent echoes each one a keystroke or more late.
        for doc in &sent {
            assert!(!echoes.foreign(editor.doc(), doc));
        }
        assert!(editor.can_undo());
        assert_eq!(editor.doc().plain_text(), "abc");
    }

    #[test]
    fn a_reloaded_doc_is_no_reset_and_a_new_one_is() {
        let mut editor = Editor::new(Doc::new());
        editor.type_text("x");
        let echoes = Echoes::default();
        let json = serde_json::to_string(editor.doc()).unwrap();
        let reloaded: Doc = serde_json::from_str(&json).unwrap();
        assert!(!echoes.foreign(editor.doc(), &reloaded));
        assert!(echoes.foreign(editor.doc(), &Doc::new()));
    }
}
