use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        accessibility::VisuallyHidden,
        common::{
            ChevronDownIcon, HtmlTag, Input, NavigationChord, States, attr, has_shortcut_modifier,
            input_from_str, navigation_chord,
        },
        form::{
            ComboboxOption, ComboboxState, PreparedField, clear_button,
            combobox::{COMBOBOX_DROPDOWN_SX, nothing_found_row},
            field_control_sx, field_props, use_combobox, use_field, use_field_frame,
            use_refocus_on_close, with_drawn_placeholder,
        },
        layout::{Box, BoxStyle, ScrollArea, use_box},
    },
    hooks::{
        ElementHandle, PopoverHandle, PopoverOptions, PopoverWidth, TYPEAHEAD_RESET, Typeahead,
        typeahead_match, use_element, use_field_list_layer, use_localization, use_popover_on,
        use_theme, use_typeahead,
    },
    localization::{CascaderLabels, fill},
    platform::{ElementApi, blur_counts, logical_key},
    str_enum::str_enum,
    sx::{StaticSx, Sx, sx},
    theme::{CssVar, Size},
};

use super::{
    nodes::{
        FlatPath, children_at, disabled_at, first_enabled, flatten_paths, join_labels,
        last_enabled, node_at, step, step_in,
    },
    option::CascaderNode,
};

str_enum! {
    /// How an open `Cascader` draws its tree.
    #[state_prefix = "layout"]
    pub enum CascaderLayout {
        /// One listbox per level, side by side.
        #[default]
        Columns = "columns",
        /// One row per full path, joined by `separator`. A search always renders this.
        Paths = "paths",
    }
}

input_from_str!(CascaderLayout);

/// The value-free tree, compared by `Rc` pointer; see `CascaderCoreProps::options`.
#[derive(Clone)]
pub(super) struct CascaderTree(pub Rc<Vec<CascaderNode>>);

impl PartialEq for CascaderTree {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

/// One row by position; `Cascader<T>` looks its option up by `indices`.
pub(super) struct CascaderRowArgs {
    pub indices: Vec<usize>,
    pub expanded: bool,
    pub selected: bool,
}

/// Always equal, like `Tree`'s `ErasedRenderNode`; the never-equal `options` keeps it fresh.
#[derive(Clone)]
pub(super) struct CascaderRender(pub Rc<dyn Fn(CascaderRowArgs) -> Element>);

impl PartialEq for CascaderRender {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

/// Query, index path and joined label in; whether the path survives out.
type MatchFn = dyn Fn(&str, &[usize], String) -> bool;

/// A caller's `filter`, erased the same way as `CascaderRender`.
#[derive(Clone)]
pub(super) struct CascaderMatch(pub Rc<MatchFn>);

impl PartialEq for CascaderMatch {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

/// The chevron sits inside the control, not the frame's trailing slot, so clicking it opens the list.
static CASCADER_TRIGGER_SX: StaticSx = StaticSx::new(|| {
    field_control_sx()
        .display("flex")
        .align_items("center")
        // The frame's height, not its contents' (todo 532, as 520).
        .align_self("stretch")
        .gap("4px")
        .cursor("pointer")
        .user_select("none")
        .selector(
            "& > [data-slot='value']",
            sx().flex("1 1 auto")
                .min_width("0")
                .overflow("hidden")
                .text_overflow("ellipsis")
                .white_space("nowrap"),
        )
        .selector("& > [data-placeholder]", sx().color("text-dimmed"))
        .selector(
            "& > svg",
            sx().flex("0 0 auto")
                .width("1em")
                .height("1em")
                .color("muted.6"),
        )
        .when("disabled", sx().cursor("not-allowed"))
});

/// The label/chevron split and the committed mark; the rest is `ComboboxOption`'s.
fn cascader_rows_sx() -> Sx {
    sx().selector(
        "& [data-slot='label']",
        sx().flex("1 1 auto")
            .min_width("0")
            .overflow("hidden")
            .text_overflow("ellipsis")
            .white_space("nowrap"),
    )
    .selector(
        "& [data-slot='branch']",
        sx().flex("0 0 auto")
            .display("inline-flex")
            .width("1em")
            .height("1em")
            .transform("rotate(-90deg)")
            // The next column opens to the left.
            .rtl(sx().transform("rotate(90deg)")),
    )
    .selector(
        "& [data-slot='branch'] > svg",
        sx().width("1em").height("1em"),
    )
    // `selected` follows the cursor here, so the committed path needs its own mark.
    .selector("& [data-state~='committed']", sx().font_weight("700"))
}

/// `column_width`, set on each column's `style` so the narrow rule can override the width.
const COLUMN_WIDTH: CssVar = CssVar::new("--lsx-cascader-column-width");

/// Below the theme's `sm`, where two columns no longer fit a phone.
fn narrow_query() -> String {
    format!("not all and (min-width: {})", Size::Sm.breakpoint_value())
}

/// The narrow header: back to the parent's level. A `button` inherits neither font nor color.
fn drill_back_sx() -> Sx {
    sx().display("flex")
        .align_items("center")
        .gap("4px")
        .width("100%")
        .padding("4px 8px")
        .border("none")
        .border_bottom("1px solid")
        .border_color("muted.2")
        .background("transparent")
        .color("inherit")
        .font_family("inherit")
        .font_size("inherit")
        .font_weight("600")
        .line_height("1.5")
        .text_align("start")
        .cursor("pointer")
        .hover(sx().background("muted.1"))
        .selector(
            "& > [data-slot='back']",
            sx().flex("0 0 auto")
                .display("inline-flex")
                .width("1em")
                .height("1em")
                .transform("rotate(90deg)")
                .rtl(sx().transform("rotate(-90deg)")),
        )
        .selector(
            "& > [data-slot='back'] > svg",
            sx().width("1em").height("1em"),
        )
        .selector(
            "& > [data-slot='title']",
            sx().min_width("0")
                .overflow("hidden")
                .text_overflow("ellipsis")
                .white_space("nowrap"),
        )
}

/// One listbox per level, side by side, each scrolling on its own.
/// Narrow, only the cursor's column shows, under a back header (todo 1084).
static CASCADER_COLUMNS_SX: StaticSx = StaticSx::new(|| {
    cascader_rows_sx()
        .display("flex")
        .align_items("stretch")
        .gap("4px")
        // Only where the dropdown is narrower than its columns; never on a desktop.
        .overflow_x("auto")
        // Grows into a wider trigger's room, or the rows' chevrons stop short of the edge.
        .selector(
            "& > [data-slot='column']",
            sx().flex("1 0 auto")
                .width(COLUMN_WIDTH.value())
                .min_width("0")
                .display("flex")
                .flex_direction("column"),
        )
        .selector(
            "& > [data-slot='column'] + [data-slot='column']",
            sx().border_left("1px solid")
                .border_color("muted.2")
                .rtl(sx().border_left("none").border_right("1px solid")),
        )
        .selector("& > [data-slot='drill-back']", sx().display("none"))
        .selector("& [data-slot='pick-parent']", sx().display("none"))
        .media(
            narrow_query(),
            sx().flex_direction("column")
                .gap("0")
                .selector("& > [data-slot='drill-back']", drill_back_sx())
                .selector(
                    "& > [data-slot='column']:not(:last-child)",
                    sx().display("none"),
                )
                .selector(
                    "& > [data-slot='column']",
                    sx().width("auto").min_width(COLUMN_WIDTH.value()),
                )
                .selector(
                    "& > [data-slot='column'] + [data-slot='column']",
                    sx().border_left("none").rtl(sx().border_right("none")),
                )
                .selector("& [data-slot='pick-parent']", sx().display("flex")),
        )
});

static CASCADER_PATHS_SX: StaticSx = StaticSx::new(cascader_rows_sx);

/// The search box above the rows, as on `Select`. Not a field control, so it carries its own chrome.
static CASCADER_SEARCH_SX: StaticSx = StaticSx::new(|| {
    sx().width("100%")
        .border("none")
        .outline("none")
        .background("transparent")
        .color("inherit")
        .font_family("inherit")
        .font_size("inherit")
        .line_height("1.5")
        .padding("4px 8px")
        .border_bottom("1px solid")
        // The box's only boundary: 3:1, as a field frame (WCAG 1.4.11, todo 490).
        .border_color("muted.6")
        .selector("::placeholder", sx().color("text-dimmed"))
});

/// [`CASCADER_SEARCH_SX`]'s padding plus bottom border, for a drawn placeholder.
const SEARCH_INSET: &str = "4px 8px 5px";

field_props! {
    pub(crate) struct CascaderCoreProps {
        /// Wrapped fresh each render on purpose: never equal, so a stale `node` or `filter` can't
        /// survive memoization ([[codebase/dioxus-memoization-traps]]).
        options: CascaderTree,
        committed: Option<Vec<usize>>,
        /// The option to commit next, by index path; `None` clears.
        onpick: EventHandler<Option<Vec<usize>>>,
        /// `None` draws the label.
        #[props(default)]
        node: Option<CascaderRender>,
        /// The trigger's text. Empty shows `placeholder`.
        display: String,
        separator: String,
        any_level: bool,
        allow_deselect: bool,
        layout: CascaderLayout,
        searchable: bool,
        column_width: String,
        #[props(default)]
        placeholder: Option<String>,
        #[props(default)]
        search_placeholder: Option<String>,
        #[props(default)]
        clearable: bool,
        /// A hidden input carrying `form_value`: the trigger is a `div` and can't post.
        #[props(default)]
        name: Option<String>,
        /// `None` posts nothing.
        #[props(default)]
        form_value: Option<String>,
        #[props(default)]
        rules: Option<crate::components::form::FieldStatus>,
        /// `None` is a case-insensitive `contains` over the joined labels.
        #[props(default)]
        filter: Option<CascaderMatch>,
    }
}

/// The `T`-free engine under `Cascader`: index paths in and out.
/// Focus stays on the trigger (or the open search box), so its blur closes the list.
#[component]
pub(crate) fn CascaderCore(props: CascaderCoreProps) -> Element {
    let theme = use_theme();
    let words = use_localization();
    let nothing_found = words.combobox.nothing_found;
    let drill_labels = words.cascader;
    let size = props.size.copied_or(theme.cascader.size);
    let radius = props.radius.copied_or(theme.cascader.radius);
    let disabled = props.disabled.unwrap_or(false);
    // As `SelectCore`: read-only stays focusable and posting, but never opens.
    let readonly = props.readonly.unwrap_or(false);
    let required = props.required.unwrap_or(false);

    let state = use_combobox();
    let opened = state.is_open() && !disabled && !readonly;
    let searchable = props.searchable && !disabled;

    // One index per level. `[2, 0]` highlights the third root's first child without expanding it.
    let cursor = use_signal(Vec::<usize>::new);
    let query = use_signal(String::new);
    let search = use_element();
    let trigger_element = use_element();
    let typed = use_typeahead(TYPEAHEAD_RESET);

    let nodes = props.options.0.clone();
    let any_level = props.any_level;

    let committed = props.committed.clone();
    let searching = searchable && opened && !query().is_empty();
    let layout = match searching {
        true => CascaderLayout::Paths,
        false => props.layout,
    };

    // The `Paths` rows the query leaves. `Columns` ignores it; it costs one walk.
    let visible: Rc<Vec<FlatPath>> = Rc::new(visible_paths(
        &nodes,
        any_level,
        searching.then(|| query.read().clone()).as_deref(),
        &props.separator,
        props.filter.as_ref(),
    ));

    let cursor_now = cursor.read().clone();
    // The cursor as a row of `visible`, the `Paths` keyboard's index.
    let path_row = visible.iter().position(|path| path.indices == cursor_now);

    let id = state.id();
    let listbox_id = format!("{id}-listbox");
    let controlled_id = controlled_id(&listbox_id, layout, &cursor_now);

    // `Rc`: each is shared by several handlers and captures a non-`Copy` path.
    let open = open_handler(cursor, state, committed.clone());
    let onpick = props.onpick;
    let commit = commit_handler(state, onpick, committed.clone(), props.allow_deselect);

    use_refocus_on_close(opened, searchable, trigger_element, query);

    let field = use_field()
        .labelled_by()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(props.rules.clone())
        .name(props.name.as_deref())
        .required(required)
        .disabled(disabled)
        .size(size)
        .radius(radius)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .attributes(&props.attributes)
        .prepare();

    let clear = clear_button(
        props.clearable && committed.is_some() && !disabled && !readonly,
        size,
        trigger_element,
        move |_| {
            onpick.call(None);
            state.close();
        },
    );

    let frame = use_field_frame()
        .trailing(&clear)
        .states(field.states())
        .prepare();

    // The frame draws the ring.
    let control = use_box()
        .framework_sx(&CASCADER_TRIGGER_SX)
        .focus_ring(false)
        .states(field.states())
        .prepare();

    let keys = Rc::new(CascaderKeys {
        nodes: nodes.clone(),
        visible: visible.clone(),
        cursor,
        state,
        open: open.clone(),
        commit: commit.clone(),
        committed: committed.clone(),
        typed,
        disabled: disabled || readonly,
        searchable,
        any_level,
        layout,
    });

    let rows = CascaderRows {
        nodes,
        node: props.node.clone(),
        separator: props.separator.clone(),
        id: id.clone(),
        cursor,
        cursor_now: cursor_now.clone(),
        committed: committed.clone().unwrap_or_default(),
        commit,
        any_level,
        size,
        radius,
        labels: drill_labels,
        state,
    };
    let body = use_cascader_body(
        rows,
        Body {
            layout,
            depth: cursor_now.len(),
            visible: visible.clone(),
            path_row,
            listbox_id: listbox_id.clone(),
            label_id: field.label_id(),
            column_width: props.column_width.clone(),
            max_height: theme.combobox.max_dropdown_height,
        },
    );

    // ---- the dropdown -------------------------------------------------
    let Dropdown {
        anchor,
        popover,
        search_box,
        wrapper,
        dropdown,
    } = use_cascader_dropdown(DropdownSetup {
        opened,
        searchable,
        search,
        size,
        radius,
        layout,
        // A new column or a shorter match list reshapes the open box.
        remeasure: (cursor_now.len() * 4096 + visible.len()) as u64,
        gap: theme.popover.gap,
        padding: theme.popover.padding,
    });

    // Drawn in the list's place and said by the status region below.
    let nothing_found = (searching && visible.is_empty()).then_some(nothing_found);
    let body = match nothing_found {
        Some(text) => nothing_found_row(text),
        None => body,
    };

    let descendant = active_descendant(&id, layout, &cursor_now, path_row);
    let header = searchable.then(|| {
        let search_placeholder = props.search_placeholder.clone().unwrap_or_default();
        with_drawn_placeholder(
            Some(&search_placeholder),
            SEARCH_INSET,
            search_header(
                search_box,
                CascaderSearch {
                    element: search,
                    query,
                    cursor,
                    state,
                },
                controlled_id.clone(),
                descendant.clone(),
                search_placeholder.clone(),
                &field,
                required,
            ),
        )
    });

    let dropdown_keys = keys.clone();
    popover.show(opened.then(|| {
        dropdown_box(
            dropdown,
            popover.floating(),
            dropdown_keys,
            rsx! {
                {header}
                {body}
            },
        )
    }));

    // ---- the trigger --------------------------------------------------
    // The open search box is the combobox; a role-less trigger may not carry `aria-required`.
    let control = match searchable && opened {
        true => control.attr("id", field.id().to_string()),
        false => field
            .aria(control)
            .attr("aria-labelledby", field.label_id()),
    };
    let trigger = cascader_trigger(
        control,
        Trigger {
            element: trigger_element,
            state,
            open: open.clone(),
            keys,
            disabled,
            readonly,
            searchable,
            chevron: clear.is_none(),
            display: props.display.clone(),
            placeholder: props.placeholder.clone(),
            controlled_id,
            descendant,
        },
        opened,
        props.attributes,
    );

    let hidden = hidden_input(props.name.clone(), props.form_value.clone(), disabled);

    field.render(rsx! {
        {
            wrapper
                .element(&anchor)
                .render(HtmlTag::Div, Vec::new(), frame.render(trigger))
        }
        {hidden}
        // Always mounted: a region inserted with its text is not announced.
        VisuallyHidden { role: "status",
            if let Some(text) = nothing_found {
                "{text}"
            }
        }
    })
}

/// Every path of the tree, narrowed to what `query` matches while searching.
fn visible_paths(
    nodes: &[CascaderNode],
    any_level: bool,
    query: Option<&str>,
    separator: &str,
    filter: Option<&CascaderMatch>,
) -> Vec<FlatPath> {
    let all_paths = flatten_paths(nodes, any_level);
    let Some(query) = query else {
        return all_paths;
    };
    let needle = query.to_lowercase();
    all_paths
        .into_iter()
        .filter(|path| {
            let label = join_labels(&path.labels, separator);
            match filter {
                Some(filter) => (filter.0)(query, &path.indices, label),
                None => label.to_lowercase().contains(&needle),
            }
        })
        .collect()
}

fn option_id(id: &str, level: usize, index: usize) -> String {
    format!("{id}-option-{level}-{index}")
}

/// Only one element is the combobox: while the search box is open, the trigger keeps just `aria-haspopup`.
fn trigger_aria(
    searchable: bool,
    opened: bool,
    controlled_id: &str,
    descendant: Option<String>,
) -> Vec<Attribute> {
    if searchable && opened {
        return vec![attr("aria-haspopup", "listbox")];
    }
    let mut trigger = vec![
        attr("role", "combobox"),
        attr("aria-haspopup", "listbox"),
        attr("aria-expanded", opened.to_string()),
    ];
    // The list is mounted only while open; a dangling id is invalid.
    if opened {
        trigger.push(attr("aria-controls", controlled_id.to_string()));
    }
    if let Some(target) = descendant.filter(|_| opened) {
        trigger.push(attr("aria-activedescendant", target));
    }
    trigger
}

/// The keyboard, by layout: `Columns` walks the tree, `Paths` a flat list (Left/Right stay the caret's).
/// In an `Rc`: the trigger and the portaled dropdown share it.
struct CascaderKeys {
    nodes: Rc<Vec<CascaderNode>>,
    visible: Rc<Vec<FlatPath>>,
    cursor: Signal<Vec<usize>>,
    state: ComboboxState,
    open: Rc<dyn Fn(bool)>,
    commit: Rc<dyn Fn(Vec<usize>, bool)>,
    committed: Option<Vec<usize>>,
    typed: Typeahead,
    disabled: bool,
    searchable: bool,
    any_level: bool,
    layout: CascaderLayout,
}

impl CascaderKeys {
    fn handle(&self, event: KeyboardEvent) {
        if self.disabled {
            return;
        }
        // APG: Alt+ArrowDown opens in place, Alt+ArrowUp closes; other chords aren't ours.
        match navigation_chord(&event) {
            Some(NavigationChord::Open) => {
                event.prevent_default();
                if !self.state.is_open() {
                    (self.open)(true);
                }
                return;
            }
            Some(NavigationChord::Close) if self.state.is_open() => {
                event.prevent_default();
                self.leave();
                return;
            }
            Some(_) => return,
            None => {}
        }
        if self.typeahead(&event) {
            return;
        }
        if !self.state.is_open() {
            self.closed(event);
            return;
        }
        let key = event.key();
        let paths_layout = self.layout == CascaderLayout::Paths;
        match key {
            Key::Escape => {
                event.prevent_default();
                self.state.close();
            }
            Key::Tab => self.leave(),
            // APG select-only: Space is Enter, except in the search box.
            Key::Character(ref character) if character == " " && !self.searchable => {
                event.prevent_default();
                self.activate();
            }
            Key::ArrowDown | Key::ArrowUp | Key::Home | Key::End | Key::Enter if paths_layout => {
                self.paths(event)
            }
            _ => self.columns(event),
        }
    }

    /// APG select-only: Tab and Alt+ArrowUp keep a pickable highlight, then
    /// close. Re-picking the committed path would clear it under `allow_deselect`.
    fn leave(&self) {
        let here = self.cursor.read().clone();
        let pickable = match self.layout {
            CascaderLayout::Paths => self
                .visible
                .iter()
                .any(|path| path.indices == here && !path.disabled),
            CascaderLayout::Columns => {
                !here.is_empty()
                    && !disabled_at(&self.nodes, &here)
                    && (self.any_level || children_at(&self.nodes, &here).is_empty())
            }
        };
        if pickable && self.committed.as_ref() != Some(&here) {
            (self.commit)(here, true);
        }
        self.state.close();
    }

    /// Enter, or Space on a list with no search box, on the highlight.
    fn activate(&self) -> bool {
        let here = self.cursor.read().clone();
        match self.layout {
            CascaderLayout::Paths => {
                // Nothing highlighted: Enter bubbles, so a form still submits.
                let Some(path) = self.visible.iter().find(|path| path.indices == here) else {
                    return false;
                };
                if !path.disabled {
                    (self.commit)(path.indices.clone(), true);
                }
                true
            }
            CascaderLayout::Columns => {
                if here.is_empty() || disabled_at(&self.nodes, &here) {
                    return false;
                }
                let children = children_at(&self.nodes, &here);
                if children.is_empty() {
                    (self.commit)(here, true);
                    return true;
                }
                // A branch expands; with `any_level` it is also picked, and the list stays open.
                if self.any_level {
                    (self.commit)(here.clone(), false);
                }
                if let Some(index) = first_enabled(children) {
                    let mut cursor = self.cursor;
                    let mut next = here;
                    next.push(index);
                    cursor.set(next);
                }
                true
            }
        }
    }

    /// A closed list: the keys that open it.
    fn closed(&self, event: KeyboardEvent) {
        let mut cursor = self.cursor;
        let open = &self.open;
        let key = logical_key(&event);
        let forward = match key {
            Key::ArrowDown | Key::ArrowRight | Key::Enter | Key::Home => true,
            Key::ArrowUp | Key::End => false,
            Key::Character(ref character) if character == " " => true,
            _ => return,
        };
        event.prevent_default();
        open(true);
        // APG select-only: Home/End open on the first/last row; other keys keep the committed path.
        if (matches!(key, Key::Home | Key::End) || cursor.read().is_empty())
            && let Some(next) = self.edge(forward)
        {
            cursor.set(next);
        }
    }

    /// The first or last enabled row of what opens: a root, or a `Paths` row.
    fn edge(&self, forward: bool) -> Option<Vec<usize>> {
        match self.layout {
            CascaderLayout::Columns => match forward {
                true => first_enabled(&self.nodes),
                false => last_enabled(&self.nodes),
            }
            .map(|index| vec![index]),
            CascaderLayout::Paths => step(
                self.visible.len(),
                |row| self.visible[row].disabled,
                None,
                forward,
            )
            .map(|row| self.visible[row].indices.clone()),
        }
    }

    /// APG typeahead within the cursor's column; opens a closed list on the roots. Off while `searchable`.
    fn typeahead(&self, event: &KeyboardEvent) -> bool {
        if self.searchable || has_shortcut_modifier(event) {
            return false;
        }
        let Key::Character(ref key) = event.key() else {
            return false;
        };
        let Some(ch) = key.chars().next() else {
            return false;
        };
        // A space mid-query is part of "new york", otherwise an activation.
        if ch == ' ' && !self.typed.is_typing() {
            return false;
        }
        let open = self.state.is_open();
        let query = self.typed.push(ch);
        let here = match open {
            true => self.cursor.read().clone(),
            false => Vec::new(),
        };
        let found = match self.layout {
            CascaderLayout::Paths => {
                let labels: Vec<String> = self
                    .visible
                    .iter()
                    .map(|path| path.labels.join(" "))
                    .collect();
                let current = self.visible.iter().position(|path| path.indices == here);
                typeahead_match(labels.len(), current, &query, |row| {
                    (!self.visible[row].disabled).then(|| labels[row].as_str())
                })
                .map(|row| self.visible[row].indices.clone())
            }
            CascaderLayout::Columns => {
                let (parents, current) = match here.split_last() {
                    Some((last, parents)) => (parents.to_vec(), Some(*last)),
                    None => (Vec::new(), None),
                };
                let column = children_at(&self.nodes, &parents);
                typeahead_match(column.len(), current, &query, |index| {
                    let mut path = parents.clone();
                    path.push(index);
                    (!disabled_at(&self.nodes, &path)).then(|| column[index].label.as_str())
                })
                .map(|index| {
                    let mut next = parents.clone();
                    next.push(index);
                    next
                })
            }
        };
        let Some(next) = found else {
            return false;
        };
        event.prevent_default();
        if !open {
            (self.open)(true);
        }
        let mut cursor = self.cursor;
        cursor.set(next);
        true
    }

    /// The flat list: Up, Down, Home, End and Enter.
    fn paths(&self, event: KeyboardEvent) {
        let (mut cursor, visible) = (self.cursor, &self.visible);
        let here = cursor.read().clone();
        let row = visible.iter().position(|path| path.indices == here);
        let key = event.key();
        if key == Key::Enter {
            if self.activate() {
                event.prevent_default();
            }
            return;
        }
        event.prevent_default();
        let from = match key {
            Key::Home | Key::End => None,
            _ => row,
        };
        let forward = matches!(key, Key::ArrowDown | Key::Home);
        if let Some(index) = step(
            visible.len(),
            |index| visible[index].disabled,
            from,
            forward,
        ) {
            cursor.set(visible[index].indices.clone());
        }
    }

    /// The tree. Under `Paths`, Left and Right stay the search box's caret keys.
    fn columns(&self, event: KeyboardEvent) {
        let (mut cursor, nodes) = (self.cursor, &self.nodes);
        let paths_layout = self.layout == CascaderLayout::Paths;
        let here = cursor.read().clone();
        let key = logical_key(&event);
        match key {
            Key::ArrowDown | Key::ArrowUp | Key::Home | Key::End => {
                event.prevent_default();
                let (parents, from) = match here.split_last() {
                    Some((last, parents)) => (parents.to_vec(), Some(*last)),
                    None => (Vec::new(), None),
                };
                let from = match key {
                    Key::Home | Key::End => None,
                    _ => from,
                };
                let forward = matches!(key, Key::ArrowDown | Key::Home);
                if let Some(index) = step_in(nodes, &parents, from, forward) {
                    let mut next = parents;
                    next.push(index);
                    cursor.set(next);
                }
            }
            Key::ArrowRight if !paths_layout => {
                event.prevent_default();
                let column = children_at(nodes, &here);
                // A disabled branch does not open, as a click on it does not.
                if !here.is_empty()
                    && !disabled_at(nodes, &here)
                    && let Some(index) = first_enabled(column)
                {
                    let mut next = here;
                    next.push(index);
                    cursor.set(next);
                }
            }
            Key::ArrowLeft if !paths_layout => {
                event.prevent_default();
                if here.len() > 1 {
                    let mut next = here;
                    next.pop();
                    cursor.set(next);
                }
            }
            // Unhandled, Enter bubbles, so a form still submits.
            Key::Enter if self.activate() => event.prevent_default(),
            _ => {}
        }
    }
}

/// Draws the rows of both layouts, for one render.
struct CascaderRows {
    nodes: Rc<Vec<CascaderNode>>,
    node: Option<CascaderRender>,
    separator: String,
    /// The combobox's id, the base of every option id.
    id: String,
    cursor: Signal<Vec<usize>>,
    cursor_now: Vec<usize>,
    /// Empty for none.
    committed: Vec<usize>,
    commit: Rc<dyn Fn(Vec<usize>, bool)>,
    any_level: bool,
    size: Size,
    radius: Size,
    labels: CascaderLabels,
    state: ComboboxState,
}

impl CascaderRows {
    /// One node through the caller's `node`; a `Paths` row draws one per level.
    fn node(&self, prefix: &[usize]) -> Element {
        let cursor = &self.cursor_now;
        match (node_at(&self.nodes, prefix), &self.node) {
            (Some(_), Some(draw)) => (draw.0)(CascaderRowArgs {
                indices: prefix.to_vec(),
                expanded: cursor.starts_with(prefix) && cursor.len() > prefix.len(),
                selected: self.committed.as_slice() == prefix,
            }),
            (Some(node), None) => rsx! { "{node.label}" },
            (None, _) => rsx! {},
        }
    }

    fn row(&self, indices: Vec<usize>, level: usize, index: usize, whole_path: bool) -> Element {
        let nodes = &self.nodes;
        let node = node_at(nodes, &indices);
        let has_children = node.is_some_and(|node| !node.children.is_empty());
        let row_disabled = disabled_at(nodes, &indices);
        // One row per column: the active descendant always carries `aria-selected` (APG).
        let on_cursor = self.cursor_now.starts_with(&indices);
        let is_cursor = self.cursor_now == indices;
        // `Paths` is one listbox holding the cursor's ancestors too, so only the cursor row is marked.
        let marked = match whole_path {
            true => is_cursor,
            false => on_cursor,
        };
        let is_committed = self.committed == indices;
        let content = match whole_path {
            false => self.node(&indices),
            true => {
                let levels: Vec<Element> = (1..=indices.len())
                    .map(|depth| self.node(&indices[..depth]))
                    .collect();
                let separator = &self.separator;
                rsx! {
                    for (at , drawn) in levels.into_iter().enumerate() {
                        if at > 0 {
                            span { "data-slot": "separator", "{separator}" }
                        }
                        {drawn}
                    }
                }
            }
        };
        let picked = indices.clone();
        // A clicked branch moves the cursor to its first child, since the cursor row never expands.
        let next_cursor = match first_enabled(children_at(nodes, &indices)) {
            Some(child) => {
                let mut next = indices.clone();
                next.push(child);
                next
            }
            None => indices.clone(),
        };
        let (commit, any_level, mut cursor) = (self.commit.clone(), self.any_level, self.cursor);
        rsx! {
            ComboboxOption {
                key: "{level}-{index}",
                id: option_id(&self.id, level, index),
                size: self.size,
                radius: self.radius,
                selected: marked,
                active: is_cursor,
                states: States::new().with("committed", is_committed),
                "aria-disabled": row_disabled.then_some("true"),
                onpick: move |_| {
                    if row_disabled {
                        return;
                    }
                    cursor.set(next_cursor.clone());
                    match has_children {
                        false => commit(picked.clone(), true),
                        true if any_level => commit(picked.clone(), false),
                        true => {}
                    }
                },
                span { "data-slot": "label", {content} }
                if has_children && !whole_path {
                    span { "data-slot": "branch", ChevronDownIcon {} }
                }
            }
        }
    }

    /// The narrow header over a child level: pops the cursor, as Left does.
    /// Never focused (`tabindex=-1`, the box cancels `mousedown`): focus stays on the trigger.
    fn drill_back(&self) -> Option<Element> {
        let (_, parents) = self.cursor_now.split_last()?;
        let parent = node_at(&self.nodes, parents)?;
        let label = parent.label.clone();
        let name = fill(self.labels.back, &[("label", &label)]);
        let (mut cursor, back_to) = (self.cursor, parents.to_vec());
        Some(rsx! {
            button {
                "data-slot": "drill-back",
                r#type: "button",
                tabindex: "-1",
                "aria-label": name,
                onclick: move |_| cursor.set(back_to.clone()),
                span { "data-slot": "back", ChevronDownIcon {} }
                span { "data-slot": "title", "{label}" }
            }
        })
    }

    /// Under `any_level`, the narrow list's first row picks the parent: a tap on it only drilled in.
    /// Outside the keyboard's rows: Enter on the parent already picks it.
    fn pick_parent(&self, level: usize) -> Option<Element> {
        let parents = self.cursor_now.get(..level).filter(|_| level > 0)?;
        let parent = node_at(&self.nodes, parents).filter(|_| self.any_level)?;
        if disabled_at(&self.nodes, parents) {
            return None;
        }
        let text = fill(self.labels.select, &[("label", &parent.label)]);
        let is_committed = self.committed.as_slice() == parents;
        let (commit, state, picked) = (self.commit.clone(), self.state, parents.to_vec());
        Some(rsx! {
            ComboboxOption {
                key: "{level}-parent",
                id: format!("{}-option-{level}-parent", self.id),
                size: self.size,
                radius: self.radius,
                selected: false,
                states: States::new().with("committed", is_committed),
                "data-slot": "pick-parent",
                onpick: move |_| match is_committed {
                    // Re-picking would clear it under `allow_deselect`.
                    true => state.close(),
                    false => commit(picked.clone(), true),
                },
                span { "data-slot": "label", "{text}" }
            }
        })
    }

    /// One column per level the cursor reached (the roots at least); never ahead of the cursor.
    fn columns(
        &self,
        strip: BoxStyle,
        listbox_id: &str,
        label_id: Option<String>,
        width: &str,
        max_height: &'static str,
    ) -> Element {
        let cursor = &self.cursor_now;
        let depth = cursor.len().max(1);
        let columns: Vec<Element> = (0..depth)
            .map(|level| {
                let parents = cursor[..level].to_vec();
                let column = children_at(&self.nodes, &parents);
                let highlighted = cursor.get(level).copied();
                let scroll_y = highlighted
                    .filter(|_| column.len() > 1)
                    .map(|index| index as f64 / (column.len() - 1) as f64 * 100.0);
                // Named by its parent row, so no English literal is needed.
                let labelled_by = match level {
                    0 => label_id.clone(),
                    _ => Some(option_id(&self.id, level - 1, cursor[level - 1])),
                };
                let rows: Vec<Element> = column
                    .iter()
                    .enumerate()
                    .map(|(index, _)| {
                        let mut indices = parents.clone();
                        indices.push(index);
                        self.row(indices, level, index, false)
                    })
                    .collect();
                let pick_parent = (level + 1 == depth)
                    .then(|| self.pick_parent(level))
                    .flatten();
                rsx! {
                    div {
                        key: "{level}",
                        "data-slot": "column",
                        style: "{COLUMN_WIDTH.name()}:{width}",
                        ScrollArea {
                            sx: sx().max_height(max_height),
                            scroll_position_y: scroll_y,
                            id: format!("{listbox_id}-{level}"),
                            "role": "listbox",
                            "aria-labelledby": labelled_by,
                            {pick_parent}
                            for row in rows {
                                {row}
                            }
                        }
                    }
                }
            })
            .collect();
        let back = self.drill_back();
        strip
            .attr("id", listbox_id.to_string())
            .attr("role", "presentation")
            .render(
                HtmlTag::Div,
                Vec::new(),
                rsx! {
                    {back}
                    for column in columns {
                        {column}
                    }
                },
            )
    }

    /// One row per path in `visible`; `path_row` is the cursor's.
    fn paths(
        &self,
        visible: &[FlatPath],
        path_row: Option<usize>,
        listbox_id: &str,
        label_id: Option<String>,
        max_height: &'static str,
    ) -> Element {
        let rows: Vec<Element> = visible
            .iter()
            .enumerate()
            .map(|(index, path)| self.row(path.indices.clone(), 0, index, true))
            .collect();
        let scroll_y = path_row
            .filter(|_| visible.len() > 1)
            .map(|row| row as f64 / (visible.len() - 1) as f64 * 100.0);
        rsx! {
            Box {
                framework_sx: &CASCADER_PATHS_SX,
                ScrollArea {
                    sx: sx().max_height(max_height),
                    scroll_position_y: scroll_y,
                    id: listbox_id.to_string(),
                    "role": "listbox",
                    "aria-labelledby": label_id,
                    for row in rows {
                        {row}
                    }
                }
            }
        }
    }
}

/// Opens on the committed path, like a native `<select>`; with none, nothing is highlighted.
fn open_handler(
    cursor: Signal<Vec<usize>>,
    state: ComboboxState,
    seed: Option<Vec<usize>>,
) -> Rc<dyn Fn(bool)> {
    Rc::new(move |next: bool| {
        // A local copy lets an `Fn` closure write the signal.
        let mut cursor = cursor;
        if next && !state.is_open() {
            cursor.set(seed.clone().unwrap_or_default());
        }
        state.set_open(next);
    })
}

/// The one commit path for keyboard and mouse. `allow_deselect` turns a re-pick into a clear;
/// `close` is false only for an `any_level` branch, which is picked and drilled into.
fn commit_handler(
    state: ComboboxState,
    onpick: EventHandler<Option<Vec<usize>>>,
    picked: Option<Vec<usize>>,
    allow_deselect: bool,
) -> Rc<dyn Fn(Vec<usize>, bool)> {
    Rc::new(move |indices: Vec<usize>, close: bool| {
        let next = match allow_deselect && picked.as_ref() == Some(&indices) {
            true => None,
            false => Some(indices),
        };
        onpick.call(next);
        if close {
            state.close();
        }
    })
}

#[derive(Clone, Copy)]
struct CascaderSearch {
    element: ElementHandle,
    query: Signal<String>,
    cursor: Signal<Vec<usize>>,
    state: ComboboxState,
}

/// The search box at the top of an open, `searchable` list.
fn search_header(
    style: BoxStyle,
    search: CascaderSearch,
    controlled_id: String,
    descendant: Option<String>,
    placeholder: String,
    // The open box is the combobox, so it carries the field's label and captions.
    field: &PreparedField,
    required: bool,
) -> Element {
    let CascaderSearch {
        element,
        mut query,
        mut cursor,
        state,
    } = search;
    style
        .element(&element)
        .attr_default("type", "text")
        .attr("value", query())
        .attr("data-controlled", true)
        .attr("placeholder", placeholder)
        // Ours is the list underneath; the browser's would cover it.
        .attr("autocomplete", "off")
        .attr("aria-autocomplete", "list")
        .attr("role", "combobox")
        .attr("aria-haspopup", "listbox")
        .attr("aria-expanded", "true")
        .attr("aria-controls", controlled_id)
        .attr("aria-labelledby", field.label_id())
        .attr("aria-describedby", field.describedby())
        .attr("aria-invalid", field.invalid().then_some("true"))
        .attr("aria-required", required.then_some("true"))
        .attr("aria-activedescendant", descendant)
        .event("oninput", move |event: FormEvent| {
            query.set(event.value());
            // A new list: nothing is armed until an arrow says so.
            cursor.set(Vec::new());
        })
        // Closes while searchable; the rows cancel `mousedown`, so a click inside never blurs.
        .event("onblur", move |event: FocusEvent| {
            if blur_counts(&event) {
                state.close();
            }
        })
        // No `onkeydown`: the portaled dropdown has it, and a second pass reopened a committing Enter.
        .render(HtmlTag::Input, Vec::new(), ())
}

/// The trigger's text: the joined path, or the placeholder.
fn value_slot(display: &str, placeholder: Option<&str>) -> Element {
    match display.is_empty() {
        false => rsx! {
            span { "data-slot": "value", "{display}" }
        },
        true => {
            let placeholder = placeholder.unwrap_or_default();
            rsx! {
                span { "data-slot": "value", "data-placeholder": "true", "{placeholder}" }
            }
        }
    }
}

/// Posts the value alone, as `Select` does; nothing selected posts nothing.
fn hidden_input(
    name: Option<String>,
    form_value: Option<String>,
    disabled: bool,
) -> Option<Element> {
    name.map(|name| {
        rsx! {
            for value in form_value.iter().cloned() {
                input {
                    r#type: "hidden",
                    name: name.clone(),
                    value,
                    disabled: disabled.then_some(true),
                }
            }
        }
    })
}

/// Focuses the search box once the list is placed, the first moment it is visible and focus can take.
fn use_focus_search(opened: bool, searchable: bool, placed: bool, search: ElementHandle) {
    use_effect(use_reactive!(|(opened, searchable, placed)| {
        if opened && searchable && placed {
            // Deferred: the opening click ends by focusing the trigger.
            spawn(async move {
                let _ = search.focus();
            });
        }
    }));
}

/// The open list's box, holding the search box and the rows.
fn dropdown_box(
    style: BoxStyle,
    floating: &ElementHandle,
    keys: Rc<CascaderKeys>,
    content: Element,
) -> Element {
    style
        .element(floating)
        // A click on the padding or scrollbar must not blur the trigger, which would close the list.
        .event("onmousedown", move |event: MouseEvent| {
            event.prevent_default()
        })
        // Portaled, so keys inside would bubble to `PortalOutlet`, not the trigger.
        .event("onkeydown", move |event: KeyboardEvent| keys.handle(event))
        .render(HtmlTag::Div, Vec::new(), content)
}

/// The listbox `aria-controls` names: in `Columns` the cursor's column, since the strip is only `presentation`.
fn controlled_id(listbox_id: &str, layout: CascaderLayout, cursor: &[usize]) -> String {
    match layout {
        CascaderLayout::Paths => listbox_id.to_string(),
        CascaderLayout::Columns => format!("{listbox_id}-{}", cursor.len().saturating_sub(1)),
    }
}

/// The row `aria-activedescendant` names: the cursor's deepest node, or its `Paths` row.
fn active_descendant(
    id: &str,
    layout: CascaderLayout,
    cursor: &[usize],
    path_row: Option<usize>,
) -> Option<String> {
    match layout {
        CascaderLayout::Paths => path_row.map(|row| option_id(id, 0, row)),
        CascaderLayout::Columns => {
            let (last, parents) = cursor.split_last()?;
            Some(option_id(id, parents.len(), *last))
        }
    }
}

struct DropdownSetup {
    opened: bool,
    searchable: bool,
    search: ElementHandle,
    size: Size,
    radius: Size,
    layout: CascaderLayout,
    remeasure: u64,
    gap: f64,
    padding: f64,
}

struct Dropdown {
    anchor: ElementHandle,
    popover: PopoverHandle,
    search_box: BoxStyle,
    wrapper: BoxStyle,
    dropdown: BoxStyle,
}

fn use_cascader_dropdown(setup: DropdownSetup) -> Dropdown {
    let DropdownSetup {
        opened,
        searchable,
        search,
        size,
        radius,
        layout,
        remeasure,
        gap,
        padding,
    } = setup;

    // On the Escape stack while open, so a surrounding `HoverCard` leaves the press to it.
    use_field_list_layer(opened);
    let anchor = use_element();
    let popover = use_popover_on(
        anchor,
        use_element(),
        opened,
        PopoverOptions::new(gap, padding)
            // Columns and joined paths run wider than the trigger.
            .width(PopoverWidth::Min)
            .remeasure(remeasure),
    );

    let search_box = use_box().framework_sx(&CASCADER_SEARCH_SX).prepare();
    let wrapper = use_box().prepare();
    let dropdown_states: Input<States> = States::new()
        .with(size.state_name(), true)
        .with(radius.radius_state_name(), true)
        .with("bordered", true)
        .with(layout.state_name(), true)
        .into();
    let dropdown = use_box()
        // `use_popover` caps it to the viewport; wide columns scroll inside instead of off-screen.
        .framework_sx(&COMBOBOX_DROPDOWN_SX)
        .states(&dropdown_states)
        .style(popover.style())
        .prepare();

    use_focus_search(opened, searchable, popover.placed(), search);

    Dropdown {
        anchor,
        popover,
        search_box,
        wrapper,
        dropdown,
    }
}

struct Trigger {
    element: ElementHandle,
    state: ComboboxState,
    open: Rc<dyn Fn(bool)>,
    keys: Rc<CascaderKeys>,
    disabled: bool,
    readonly: bool,
    searchable: bool,
    /// Nothing to clear, so the trigger draws its chevron.
    chevron: bool,
    display: String,
    placeholder: Option<String>,
    controlled_id: String,
    descendant: Option<String>,
}

/// The frame's control, and the one tab stop.
fn cascader_trigger(
    control: BoxStyle,
    parts: Trigger,
    opened: bool,
    extra: Vec<Attribute>,
) -> Element {
    let Trigger {
        element,
        state,
        open,
        keys,
        disabled,
        readonly,
        searchable,
        chevron,
        display,
        placeholder,
        controlled_id,
        descendant,
    } = parts;

    let mut attributes = trigger_aria(searchable, opened, &controlled_id, descendant);
    attributes.extend(extra);

    let value_slot = value_slot(&display, placeholder.as_deref());

    let toggle = open;
    control
        .element(&element)
        .attr("aria-disabled", disabled.then_some("true"))
        .attr("aria-readonly", readonly.then_some("true"))
        .attr("tabindex", (!disabled).then_some("0"))
        .event("onclick", move |_: MouseEvent| {
            if !disabled && !readonly {
                toggle(!state.is_open());
            }
        })
        // While searchable, focus moves to the search box, whose blur closes instead.
        .event("onblur", move |event: FocusEvent| {
            if !searchable && blur_counts(&event) {
                state.close();
            }
        })
        // Not on a frame wrapper: it would take the x's Enter and Space.
        .event("onkeydown", move |event: KeyboardEvent| keys.handle(event))
        .render(
            HtmlTag::Div,
            attributes,
            rsx! {
                {value_slot}
                if chevron {
                    ChevronDownIcon {}
                }
            },
        )
}

struct Body {
    layout: CascaderLayout,
    /// The cursor's depth, the number of open columns.
    depth: usize,
    visible: Rc<Vec<FlatPath>>,
    path_row: Option<usize>,
    listbox_id: String,
    label_id: Option<String>,
    column_width: String,
    max_height: &'static str,
}

/// A strip of columns, or the flat list of paths.
fn use_cascader_body(rows: CascaderRows, parts: Body) -> Element {
    let Body {
        layout,
        depth,
        visible,
        path_row,
        listbox_id,
        label_id,
        column_width,
        max_height,
    } = parts;

    // A narrow strip scrolls sideways; each new column scrolls into view, as the cursor lives there.
    let strip = use_element();
    use_effect(use_reactive!(|depth| {
        let _ = depth;
        // Subscribes to every reopen, which mounts a new strip at the same handle.
        if strip.mount_token().is_some() {
            // Past the end on purpose: the browser clamps it.
            let _ = strip.scroll_to(f64::from(u32::MAX), 0.0);
        }
    }));
    let columns = use_box()
        .framework_sx(&CASCADER_COLUMNS_SX)
        .prepare()
        .element(&strip);
    match layout {
        CascaderLayout::Columns => rows.columns(
            columns,
            &listbox_id,
            label_id.clone(),
            &column_width,
            max_height,
        ),
        CascaderLayout::Paths => rows.paths(&visible, path_row, &listbox_id, label_id, max_height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::Stylesheet;

    /// Columns grow into a wider trigger's room, so chevrons reach the edge. Reads the rule, not a layout.
    #[test]
    fn the_columns_fill_the_dropdown() {
        let css = Stylesheet::from(&*CASCADER_COLUMNS_SX);
        let css = css.as_str();

        assert!(css.contains("flex:1 0 auto;"), "{css}");
    }

    /// Below `sm` only the cursor's column shows, under the back header; the width moves to `min-width`.
    #[test]
    fn a_narrow_screen_drills_into_one_column() {
        let css = Stylesheet::from(&*CASCADER_COLUMNS_SX);
        let css = css.as_str();
        let narrow = css
            .split_once("@media not all and (min-width: 48rem){")
            .map(|(_, rest)| rest)
            .unwrap_or_else(|| panic!("no narrow rule: {css}"));

        assert!(
            narrow.contains("[data-slot='column']:not(:last-child){display:none;}"),
            "{narrow}"
        );
        assert!(
            narrow.contains("[data-slot='drill-back']{display:flex;"),
            "{narrow}"
        );
        assert!(
            narrow.contains("min-width:var(--lsx-cascader-column-width);"),
            "{narrow}"
        );
        // Wide, the header and the parent row stay hidden.
        let (wide, _) = css.split_once("@media").unwrap();
        assert!(
            wide.contains("[data-slot='drill-back']{display:none;}"),
            "{wide}"
        );
        assert!(
            wide.contains("width:var(--lsx-cascader-column-width);"),
            "{wide}"
        );
    }

    /// `aria-controls` must name the column that holds the `aria-activedescendant` row.
    #[test]
    fn aria_controls_names_the_column_the_active_row_is_in() {
        let cursor = vec![1, 0];
        let controls = controlled_id("x-listbox", CascaderLayout::Columns, &cursor);
        let active = active_descendant("x", CascaderLayout::Columns, &cursor, None).unwrap();
        assert_eq!(controls, "x-listbox-1");
        // Column `{listbox_id}-{level}`, row `{id}-option-{level}-{index}`: the levels match.
        assert_eq!(active, "x-option-1-0");

        // Nothing highlighted: the one root column is named.
        assert_eq!(
            controlled_id("x-listbox", CascaderLayout::Columns, &[]),
            "x-listbox-0"
        );
        // `Paths` is one listbox, and it carries `listbox_id` itself.
        assert_eq!(
            controlled_id("x-listbox", CascaderLayout::Paths, &cursor),
            "x-listbox"
        );
    }

    /// Wider than a phone, the columns scroll inside the viewport-capped dropdown (`tests/all/combobox.rs`).
    #[test]
    fn the_columns_stay_inside_the_viewport() {
        let columns = Stylesheet::from(&*CASCADER_COLUMNS_SX);
        assert!(
            columns.as_str().contains("overflow-x:auto;"),
            "{}",
            columns.as_str()
        );
    }
}
