use std::{any::Any, rc::Rc};

use dioxus::prelude::*;

use crate::{
    components::{
        ActionIcon, Box, ComboboxOption, HtmlTag, Input, States,
        common::{attr, field_props, input_from_str},
        form::{
            combobox::COMBOBOX_DROPDOWN_SX,
            field_control_sx,
            glyphs::{ChevronIcon, CloseIcon},
            use_combobox, use_field, use_field_frame,
        },
        layout::{ScrollArea, use_box},
        navigation::TreeNodeErased,
    },
    hooks::{PopoverOptions, PopoverWidth, use_element, use_popover, use_theme},
    platform::ElementApi,
    str_enum::str_enum,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
};

use super::nodes::{
    FlatPath, children_at, disabled_at, first_enabled, flatten_paths, ids_at, indices_for_ids,
    join_labels, last_enabled, node_at, step,
};

str_enum! {
    /// How an open `Cascader` draws its tree.
    #[state_prefix = "layout"]
    pub enum CascaderLayout {
        /// One listbox per level, side by side - the column walk the value's
        /// shape is named after.
        #[default]
        Columns = "columns",
        /// One row per full path, joined by `separator`. Also what a search
        /// renders, whatever this says.
        Paths = "paths",
    }
}

input_from_str!(CascaderLayout);

impl From<CascaderLayout> for Input<CascaderLayout> {
    fn from(value: CascaderLayout) -> Self {
        Self::Value(value)
    }
}

/// Type-erased mirror of `CascaderNodeArgs<T>`. `Cascader<T>` wraps the typed
/// callback in one that downcasts `data` back to `T`, so a downcast is the
/// only per-`T` cost of a custom row.
pub(super) struct CascaderNodeArgsErased {
    pub data: Rc<dyn Any>,
    pub level: usize,
    pub expanded: bool,
    pub selected: bool,
}

/// `Rc<dyn Fn>` wrapper, so `CascaderCoreProps` derives `Clone`/`PartialEq`
/// without being generic over `T`. Always equal, like `Tree`'s
/// `ErasedRenderNode`: the closure instance does not change a row's output for
/// given args, and what keeps a *stale* one from surviving is `nodes` - see
/// the note on `CascaderCoreProps::nodes`.
#[derive(Clone)]
pub(super) struct CascaderRender(Rc<dyn Fn(CascaderNodeArgsErased) -> Element>);

impl CascaderRender {
    pub(super) fn new(f: impl Fn(CascaderNodeArgsErased) -> Element + 'static) -> Self {
        Self(Rc::new(f))
    }
}

impl PartialEq for CascaderRender {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

/// The trigger: the joined path or the placeholder on one line, and the
/// chevron at its end - inside the control rather than in the frame's trailing
/// slot, so a click on the chevron opens the list too.
static CASCADER_TRIGGER_SX: StaticSx = StaticSx::new(|| {
    field_control_sx()
        .display("flex")
        .align_items("center")
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
        .selector("& > [data-placeholder]", sx().color("grey.6"))
        .selector(
            "& > svg",
            sx().flex("0 0 auto")
                .width("1em")
                .height("1em")
                .color("grey.6"),
        )
        .when("disabled", sx().cursor("not-allowed"))
});

/// What both layouts do to the rows inside them. A row is a `ComboboxOption`,
/// so its tint, its ring and its `aria-selected` are already settled - this is
/// only the label/chevron split, and the one mark `ComboboxOption` has no
/// state for.
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
            .transform("rotate(-90deg)"),
    )
    .selector(
        "& [data-slot='branch'] > svg",
        sx().width("1em").height("1em"),
    )
    // The committed path, which is *not* what `selected` means here:
    // `selected` follows the cursor, so `aria-activedescendant` always has a
    // marked row under it. The value then needs a mark of its own, or it is
    // invisible the moment the arrows wander off it.
    .selector("& [data-state~='committed']", sx().font_weight("700"))
}

/// One listbox per level, side by side, each scrolling on its own.
static CASCADER_COLUMNS_SX: StaticSx = StaticSx::new(|| {
    cascader_rows_sx()
        .display("flex")
        .align_items("stretch")
        .gap("4px")
        // A column never shrinks below `column_width`, and grows into the
        // room a trigger wider than the open columns leaves - or the rows,
        // and the chevrons at their ends, stop short of the dropdown's edge.
        .selector(
            "& > [data-slot='column']",
            sx().flex("1 0 auto")
                .min_width("0")
                .display("flex")
                .flex_direction("column"),
        )
        .selector(
            "& > [data-slot='column'] + [data-slot='column']",
            sx().border_left("1px solid").border_color("grey.2"),
        )
});

/// One row per full path. No columns, so the rows only need the shared bits.
static CASCADER_PATHS_SX: StaticSx = StaticSx::new(cascader_rows_sx);

/// The search box at the top of the list - the shape `Select` settled on. It
/// is not a field control: it sits inside the dropdown, above the rows and
/// outside their scroll, so it carries its own chrome.
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
        .border_color("grey.3")
        .selector("::placeholder", sx().color("grey.6"))
});

field_props! {
    pub(crate) struct CascaderCoreProps {
        /// The erased tree, **re-erased on every render on purpose**.
        /// `TreeNodeErased` compares its payload by `Rc` pointer, so a fresh
        /// erasure never compares equal - which is what stops this component
        /// memoizing and taking a stale `node` callback with it
        /// ([[codebase/components/combobox]], the memoization bug). `Tree`
        /// caches its erasure because it has no callback prop that can go
        /// stale.
        nodes: Vec<TreeNodeErased>,
        /// The committed path's ids, root to leaf. Empty is no selection.
        value: Vec<String>,
        /// The path to commit next - empty to clear.
        onpick: EventHandler<Vec<String>>,
        /// Draws one row's content.
        node: CascaderRender,
        /// The trigger's text. Empty shows `placeholder`.
        display: String,
        separator: String,
        /// Mantine's `changeOnSelect`: a branch commits as well as expanding.
        any_level: bool,
        /// Committing the path that is already committed clears it instead.
        allow_deselect: bool,
        layout: CascaderLayout,
        /// Puts a search box at the top of the list, which switches it to
        /// `Paths` over the matches.
        searchable: bool,
        column_width: String,
        #[props(default)]
        placeholder: Option<String>,
        #[props(default)]
        search_placeholder: Option<String>,
        /// Shows an x in place of the chevron while a path is committed.
        #[props(default)]
        clearable: bool,
        /// One hidden input of that name per level, so the path posts with a
        /// native form - the shape `MultiSelect` sends. The trigger is a `div`
        /// and cannot carry a `name` itself.
        #[props(default)]
        name: Option<String>,
        /// What the skin's `validate` rules say; `T` never reaches here.
        #[props(default)]
        rules: Option<crate::components::FieldStatus>,
        /// Which flattened paths survive the query, one `bool` per entry of
        /// `flatten_paths(nodes, any_level)`. `None` is the default filter,
        /// which the core runs itself over the joined labels it already has.
        #[props(default)]
        matches: Option<Callback<String, Vec<bool>>>,
    }
}

/// The engine under `Cascader`. It never sees `T`: the skin hands it an erased
/// tree and a row renderer, and takes a path of ids back.
///
/// Focus stays on the trigger the whole time - the rows and the list cancel
/// `mousedown` - so losing it is what closes the list on an outside click,
/// with no window-level listener. While `searchable` and open, focus is in the
/// dropdown's search box instead and that box's blur closes the list.
#[component]
pub(crate) fn CascaderCore(props: CascaderCoreProps) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.cascader.size);
    let radius = props.radius.copied_or(theme.cascader.radius);
    let disabled = props.disabled.unwrap_or(false);
    let required = props.required.unwrap_or(false);

    let state = use_combobox();
    let opened = state.opened() && !disabled;
    let searchable = props.searchable && !disabled;

    // One index per level. `[2, 0]` highlights the first child of the third
    // root, and - Mantine's `getCascaderColumns` rule - does *not* expand it:
    // the columns never run ahead of the cursor.
    let mut cursor = use_signal(Vec::<usize>::new);
    let mut query = use_signal(String::new);
    let search = use_element();
    let trigger_element = use_element();
    let mut was_open = use_signal(|| false);

    let nodes = Rc::new(props.nodes.clone());
    let separator = props.separator.clone();
    let any_level = props.any_level;
    let allow_deselect = props.allow_deselect;

    let committed = indices_for_ids(&nodes, &props.value);
    let searching = searchable && opened && !query().is_empty();
    let layout = match searching {
        true => CascaderLayout::Paths,
        false => props.layout,
    };

    // Every path the `Paths` layout could draw, and which of them the query
    // leaves. `Columns` never reads it, and building it costs one walk of a
    // tree the component is holding anyway.
    let all_paths = flatten_paths(&nodes, any_level);
    let visible: Rc<Vec<FlatPath>> = Rc::new(match (searching, props.matches.as_ref()) {
        (false, _) => all_paths,
        (true, Some(matches)) => {
            let mask = matches.call(query());
            all_paths
                .into_iter()
                .enumerate()
                .filter(|(index, _)| mask.get(*index).copied().unwrap_or(false))
                .map(|(_, path)| path)
                .collect()
        }
        // The default filter needs nothing from `T` - the labels are already
        // on the erased nodes, so the skin is never asked for a callback.
        (true, None) => {
            let needle = query().to_lowercase();
            all_paths
                .into_iter()
                .filter(|path| {
                    join_labels(&path.labels, &separator)
                        .to_lowercase()
                        .contains(&needle)
                })
                .collect()
        }
    });

    let cursor_now = cursor.read().clone();
    let committed_now = committed.clone().unwrap_or_default();
    // The cursor as a row of `visible`, which is the `Paths` keyboard's index.
    let path_row = visible.iter().position(|path| path.indices == cursor_now);

    let id = state.id();
    let listbox_id = format!("{id}-listbox");
    let option_id = {
        let id = id.clone();
        move |level: usize, index: usize| format!("{id}-option-{level}-{index}")
    };

    // `Rc` rather than a bare closure: every one of these is needed in two or
    // more handlers, and what they capture - a path, a tree - is not `Copy`.
    let open: Rc<dyn Fn(bool)> = {
        let seed = committed.clone();
        Rc::new(move |next: bool| {
            // `Signal` is `Copy`, so a local copy is what lets an `Fn` closure
            // write one.
            let mut cursor = cursor;
            if next && !state.opened() {
                // The list opens on what is already committed, like a native
                // `<select>`. With nothing committed there is no highlight
                // until a key makes one.
                cursor.set(seed.clone().unwrap_or_default());
            }
            state.set_opened(next);
        })
    };

    // The one place a path is committed, shared by the keyboard and the mouse.
    // `allow_deselect` turns a re-pick into a clear, which is the same edit the
    // x makes. `close` is false for the one commit that is not the end of the
    // interaction: an `any_level` branch, which is picked *and* drilled into.
    let onpick = props.onpick;
    let commit: Rc<dyn Fn(Vec<String>, bool)> = {
        let picked = props.value.clone();
        Rc::new(move |ids: Vec<String>, close: bool| {
            let next = match allow_deselect && ids == picked {
                true => Vec::new(),
                false => ids,
            };
            onpick.call(next);
            if close {
                state.close();
            }
        })
    };

    // Closing clears the query and hands focus back to the trigger, which
    // would otherwise be lost to the body - the box the user was typing in has
    // just unmounted. Opening is deliberately not handled here: the list is
    // `visibility: hidden` until `use_popover` has measured it, and focusing a
    // hidden element does nothing while still reporting success.
    use_effect(use_reactive!(|(opened, searchable)| {
        if !searchable {
            return;
        }
        let previously = *was_open.peek();
        if previously && !opened {
            query.set(String::new());
            let _ = trigger_element.focus();
        }
        if previously != opened {
            was_open.set(opened);
        }
    }));

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

    let icon_size: Input<ThemeAwareValue> = ThemeAwareValue::Size(size).into();
    let clear = (props.clearable && !props.value.is_empty() && !disabled).then(|| {
        rsx! {
            ActionIcon {
                aria_label: "Clear",
                size: icon_size,
                onclick: move |_| {
                    onpick.call(Vec::new());
                    state.close();
                },
                CloseIcon {}
            }
        }
    });

    let frame = use_field_frame()
        .trailing(&clear)
        .states(field.states())
        .prepare();

    // The frame draws the ring, so the trigger must not draw a second one.
    let control = use_box()
        .framework_sx(&CASCADER_TRIGGER_SX)
        .focus_ring(false)
        .states(field.states())
        .prepare();

    // ---- the keyboard -------------------------------------------------
    //
    // Two tables on one handler, chosen by the layout that is on screen.
    // `Columns` walks the tree; `Paths` walks a flat list and leaves Left and
    // Right to the search box's caret.
    let keys: Rc<dyn Fn(KeyboardEvent)> = {
        let nodes = nodes.clone();
        let visible = visible.clone();
        let open = open.clone();
        let commit = commit.clone();
        Rc::new(move |event: KeyboardEvent| {
            let mut cursor = cursor;
            if disabled {
                return;
            }
            let key = event.key();

            if !state.opened() {
                match key {
                    Key::ArrowDown | Key::ArrowRight | Key::Enter => {
                        event.prevent_default();
                        open(true);
                        if cursor.read().is_empty()
                            && let Some(index) = first_enabled(&nodes)
                        {
                            cursor.set(vec![index]);
                        }
                    }
                    Key::ArrowUp => {
                        event.prevent_default();
                        open(true);
                        if cursor.read().is_empty()
                            && let Some(index) = last_enabled(&nodes)
                        {
                            cursor.set(vec![index]);
                        }
                    }
                    Key::Character(ref character) if character == " " => {
                        event.prevent_default();
                        open(true);
                    }
                    _ => {}
                }
                return;
            }

            let here = cursor.read().clone();
            let row = visible.iter().position(|path| path.indices == here);
            let paths_layout = layout == CascaderLayout::Paths;

            match key {
                Key::Escape => {
                    event.prevent_default();
                    state.close();
                }
                Key::Tab => state.close(),
                // The page must not scroll under an open list. While
                // `searchable` the focus is in the search box, where a space
                // is ordinary typing.
                Key::Character(ref character) if character == " " && !searchable => {
                    event.prevent_default();
                }
                Key::ArrowDown | Key::ArrowUp | Key::Home | Key::End if paths_layout => {
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
                Key::Enter if paths_layout => {
                    // Nothing highlighted means Enter is not ours: it bubbles,
                    // so a form still submits.
                    let Some(path) = row.and_then(|row| visible.get(row)) else {
                        return;
                    };
                    event.prevent_default();
                    if !path.disabled {
                        // A `Paths` row is a whole path, so there is nothing
                        // left to drill into - every pick here is the end.
                        commit(path.ids.clone(), true);
                    }
                }
                Key::ArrowDown | Key::ArrowUp | Key::Home | Key::End => {
                    event.prevent_default();
                    let (parents, from) = match here.split_last() {
                        Some((last, parents)) => (parents.to_vec(), Some(*last)),
                        None => (Vec::new(), None),
                    };
                    let column = children_at(&nodes, &parents);
                    let from = match key {
                        Key::Home | Key::End => None,
                        _ => from,
                    };
                    let forward = matches!(key, Key::ArrowDown | Key::Home);
                    if let Some(index) =
                        step(column.len(), |index| column[index].disabled, from, forward)
                    {
                        let mut next = parents;
                        next.push(index);
                        cursor.set(next);
                    }
                }
                // `Paths` has no levels to walk, so Left and Right are left
                // alone - which is what lets them move the search box's caret.
                Key::ArrowRight if !paths_layout => {
                    event.prevent_default();
                    let column = children_at(&nodes, &here);
                    if !here.is_empty()
                        && let Some(index) = first_enabled(column)
                    {
                        let mut next = here;
                        next.push(index);
                        cursor.set(next);
                    }
                }
                Key::ArrowLeft if !paths_layout => {
                    event.prevent_default();
                    // At the root there is nothing to go up to.
                    if here.len() > 1 {
                        let mut next = here;
                        next.pop();
                        cursor.set(next);
                    }
                }
                Key::Enter => {
                    if here.is_empty() || disabled_at(&nodes, &here) {
                        return;
                    }
                    event.prevent_default();
                    let children = children_at(&nodes, &here);
                    if children.is_empty() {
                        commit(ids_at(&nodes, &here), true);
                        return;
                    }
                    // A branch expands. With `any_level` it is picked on the
                    // way, and the list stays open so the walk can go on.
                    if any_level {
                        commit(ids_at(&nodes, &here), false);
                    }
                    if let Some(index) = first_enabled(children) {
                        let mut next = here;
                        next.push(index);
                        cursor.set(next);
                    }
                }
                _ => {}
            }
        })
    };

    // ---- the rows -----------------------------------------------------
    //
    // One node, through the skin's renderer. A `Paths` row is several of these
    // with the separator between them, so the caller's `node` still draws
    // every level rather than being skipped for the flat layout.
    let draw_node = |prefix: &[usize]| match node_at(&nodes, prefix) {
        Some(node) => (props.node.0)(CascaderNodeArgsErased {
            data: node.data.clone(),
            level: prefix.len().saturating_sub(1),
            expanded: cursor_now.starts_with(prefix) && cursor_now.len() > prefix.len(),
            selected: committed_now.as_slice() == prefix,
        }),
        None => rsx! {},
    };

    let draw_row = |indices: Vec<usize>, level: usize, index: usize, whole_path: bool| {
        let node = node_at(&nodes, &indices);
        let has_children = node.is_some_and(|node| !node.children.is_empty());
        let row_disabled = disabled_at(&nodes, &indices);
        // On the cursor's own chain, which in each column is exactly one row -
        // so the row `aria-activedescendant` points at always carries
        // `aria-selected`, the APG contract a committed-only mark would break
        // the moment two columns are open.
        let on_cursor = cursor_now.starts_with(&indices);
        let is_cursor = cursor_now == indices;
        let is_committed = committed_now == indices;
        let content = match whole_path {
            false => draw_node(&indices),
            // Every level, separated - a `Paths` row is the path, which is
            // what makes one row of it enough to pick by.
            true => {
                let levels: Vec<Element> = (1..=indices.len())
                    .map(|depth| draw_node(&indices[..depth]))
                    .collect();
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
        let ids = ids_at(&nodes, &indices);
        // Clicking a branch puts the cursor on its first child, not on the
        // branch itself - the deepest highlighted node is never expanded, so
        // stopping on the branch would show no children at all.
        let next_cursor = match first_enabled(children_at(&nodes, &indices)) {
            Some(child) => {
                let mut next = indices.clone();
                next.push(child);
                next
            }
            None => indices.clone(),
        };
        let commit = commit.clone();
        rsx! {
            ComboboxOption {
                key: "{level}-{index}",
                id: option_id(level, index),
                size,
                radius,
                selected: on_cursor,
                active: is_cursor,
                states: States::new().with("committed", is_committed),
                "aria-disabled": row_disabled.then_some("true"),
                onpick: move |_| {
                    if row_disabled {
                        return;
                    }
                    cursor.set(next_cursor.clone());
                    match has_children {
                        false => commit(ids.clone(), true),
                        true if any_level => commit(ids.clone(), false),
                        true => {}
                    }
                },
                span { "data-slot": "label", {content} }
                if has_children && !whole_path {
                    span { "data-slot": "branch", ChevronIcon {} }
                }
            }
        }
    };

    let max_height = theme.combobox.max_dropdown_height;
    let body = match layout {
        CascaderLayout::Columns => {
            // One column per level the cursor has reached, and the roots when
            // it has reached none. The deepest highlighted node is *not*
            // expanded, so the columns never run ahead of the cursor.
            let depth = cursor_now.len().max(1);
            let columns: Vec<Element> = (0..depth)
                .map(|level| {
                    let parents = cursor_now[..level].to_vec();
                    let column = children_at(&nodes, &parents);
                    let highlighted = cursor_now.get(level).copied();
                    let scroll_y = highlighted
                        .filter(|_| column.len() > 1)
                        .map(|index| index as f64 / (column.len() - 1) as f64 * 100.0);
                    // Named by the row it hangs off, so no column needs an
                    // English literal to be announced by.
                    let labelled_by = match level {
                        0 => field.label_id(),
                        _ => Some(option_id(level - 1, cursor_now[level - 1])),
                    };
                    let rows: Vec<Element> = column
                        .iter()
                        .enumerate()
                        .map(|(index, _)| {
                            let mut indices = parents.clone();
                            indices.push(index);
                            draw_row(indices, level, index, false)
                        })
                        .collect();
                    let width = props.column_width.clone();
                    rsx! {
                        div {
                            key: "{level}",
                            "data-slot": "column",
                            style: "width:{width}",
                            ScrollArea {
                                sx: sx().max_height(max_height),
                                scroll_position_y: scroll_y,
                                id: format!("{listbox_id}-{level}"),
                                "role": "listbox",
                                "aria-labelledby": labelled_by,
                                for row in rows {
                                    {row}
                                }
                            }
                        }
                    }
                })
                .collect();
            rsx! {
                Box {
                    framework_sx: &CASCADER_COLUMNS_SX,
                    id: listbox_id.clone(),
                    "role": "presentation",
                    for column in columns {
                        {column}
                    }
                }
            }
        }
        CascaderLayout::Paths => {
            let rows: Vec<Element> = visible
                .iter()
                .enumerate()
                .map(|(index, path)| draw_row(path.indices.clone(), 0, index, true))
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
                        id: listbox_id.clone(),
                        "role": "listbox",
                        "aria-labelledby": field.label_id(),
                        for row in rows {
                            {row}
                        }
                    }
                }
            }
        }
    };

    // ---- the dropdown -------------------------------------------------
    let anchor = use_element();
    // The box changes shape while it is open: a column appears, and a query
    // shortens the list.
    let remeasure = (cursor_now.len() * 4096 + visible.len()) as u64;
    let popover = use_popover(
        anchor,
        opened,
        PopoverOptions::new(theme.popover.gap, theme.popover.padding)
            // The columns are the content and are wider than the trigger, so
            // the box must not be clipped to it. A `Paths` row is a whole
            // joined path, which is long for the same reason.
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
        .framework_sx(&COMBOBOX_DROPDOWN_SX)
        .states(&dropdown_states)
        .style(popover.style())
        .prepare();

    // Placed means measured, which means visible - the first moment at which
    // focusing anything inside the box can take.
    let placed = popover.placed();
    use_effect(use_reactive!(|(opened, searchable, placed)| {
        if opened && searchable && placed {
            // Out of this dispatch: the click that opened the list ends by
            // focusing the trigger, so focusing inline is undone a moment
            // later.
            spawn(async move {
                let _ = search.focus();
            });
        }
    }));

    let descendant = active_descendant(&option_id, layout, &cursor_now, path_row);
    let search_placeholder = props.search_placeholder.clone().unwrap_or_default();
    let header = searchable.then(|| {
        search_box
            .element(&search)
            .attr_default("type", "text")
            .attr("value", query())
            .attr("data-controlled", true)
            .attr("placeholder", search_placeholder)
            // Ours is the list underneath; the browser's would cover it.
            .attr("autocomplete", "off")
            .attr("aria-autocomplete", "list")
            .attr("role", "combobox")
            .attr("aria-haspopup", "listbox")
            .attr("aria-expanded", "true")
            .attr("aria-controls", listbox_id.clone())
            .attr("aria-activedescendant", descendant.clone())
            .event("oninput", move |event: FormEvent| {
                query.set(event.value());
                // The list under the highlight just changed; nothing in the
                // new one is armed until an arrow says so.
                cursor.set(Vec::new());
            })
            // The trigger's blur no longer closes while searchable - this
            // does, and the rows and the list cancel `mousedown`, so a click
            // inside never reaches it.
            .event("onblur", move |_: FocusEvent| state.close())
            // **No `onkeydown` here.** The box is inside the portaled
            // dropdown, which carries the very same handler, so a second one
            // would run the whole table twice per key - and the second pass
            // sees the state the first left. An Enter that committed and
            // closed was reopened by its own second pass, measured in
            // Chromium.
            .render(HtmlTag::Input, Vec::new(), ())
    });

    let dropdown_keys = keys.clone();
    popover.show(opened.then(|| {
        dropdown
            .element(popover.floating())
            // Clicking the list's padding or its scrollbar must not move focus
            // off the trigger: a trigger that closes on blur would close under
            // the click. The rows cancel it for themselves already.
            .event("onmousedown", move |event: MouseEvent| {
                event.prevent_default()
            })
            // The same handler as the wrapper's, because the dropdown is
            // portaled: it is no descendant of that wrapper, so a key pressed
            // inside it would otherwise bubble to `PortalOutlet` and die.
            .event("onkeydown", move |event: KeyboardEvent| {
                dropdown_keys(event)
            })
            .render(
                HtmlTag::Div,
                Vec::new(),
                rsx! {
                    {header}
                    {body}
                },
            )
    }));

    // ---- the trigger --------------------------------------------------
    //
    // Two elements cannot both be the combobox. While the search box is open
    // it owns the role, `aria-controls` and `aria-activedescendant`; the
    // trigger keeps only what says a list hangs off it.
    let mut attributes = match searchable && opened {
        true => vec![
            attr("aria-haspopup", "listbox"),
            attr("aria-expanded", "true"),
        ],
        false => {
            let mut trigger = vec![
                attr("role", "combobox"),
                attr("aria-haspopup", "listbox"),
                attr("aria-expanded", opened.to_string()),
                attr("aria-controls", listbox_id.clone()),
            ];
            if let Some(target) = descendant.filter(|_| opened) {
                trigger.push(attr("aria-activedescendant", target));
            }
            trigger
        }
    };
    attributes.extend(props.attributes);

    let placeholder = props.placeholder.clone().unwrap_or_default();
    let display = props.display.clone();
    let value_slot = match display.is_empty() {
        false => rsx! {
            span { "data-slot": "value", "{display}" }
        },
        true => rsx! {
            span { "data-slot": "value", "data-placeholder": "true", "{placeholder}" }
        },
    };

    let toggle = open.clone();
    let trigger = field
        .aria(control)
        .element(&trigger_element)
        .attr("aria-labelledby", field.label_id())
        .attr("aria-disabled", disabled.then_some("true"))
        .attr("tabindex", (!disabled).then_some("0"))
        .event("onclick", move |_: MouseEvent| {
            if !disabled {
                toggle(!state.opened());
            }
        })
        // While searchable the focus moves into the search box, so closing on
        // the trigger's blur would shut the list before a key could land.
        .event("onblur", move |_: FocusEvent| {
            if !searchable {
                state.close();
            }
        })
        .render(
            HtmlTag::Div,
            attributes,
            rsx! {
                {value_slot}
                if clear.is_none() {
                    ChevronIcon {}
                }
            },
        );

    // A hidden input is the only way a control that is not a form element can
    // post - the shape `Slider` and `PinField` use. One per level, all sharing
    // the field's name: a repeated name is an ordered list on the wire, which
    // is what `MultiSelect` has posted since 2026-09-14.
    let hidden = props.name.clone().map(|name| {
        rsx! {
            for step in props.value.iter().cloned() {
                input {
                    r#type: "hidden",
                    name: name.clone(),
                    value: step,
                    disabled: disabled.then_some(true),
                }
            }
        }
    });

    field.render(rsx! {
        {
            wrapper
                .element(&anchor)
                // The trigger is inside this wrapper, so the keys are caught
                // where they bubble to. The portaled dropdown carries the very
                // same handler, for focus that moved into the search box.
                .event("onkeydown", move |event: KeyboardEvent| keys(event))
                .render(HtmlTag::Div, Vec::new(), frame.render(trigger))
        }
        {hidden}
    })
}

/// Which row `aria-activedescendant` points at - the cursor's deepest node in
/// `Columns`, and its row in the flat list in `Paths`.
fn active_descendant(
    option_id: &impl Fn(usize, usize) -> String,
    layout: CascaderLayout,
    cursor: &[usize],
    path_row: Option<usize>,
) -> Option<String> {
    match layout {
        CascaderLayout::Paths => path_row.map(|row| option_id(0, row)),
        CascaderLayout::Columns => {
            let (last, parents) = cursor.split_last()?;
            Some(option_id(parents.len(), *last))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::Stylesheet;

    /// The dropdown is at least as wide as the trigger, so one 220px column
    /// in a wider field leaves a gap to its right, and every row's chevron
    /// ends short of the dropdown's edge. The columns grow into that space
    /// and never shrink below `column_width`. The dropdown only exists in a
    /// browser, so this reads the rule rather than a layout.
    #[test]
    fn the_columns_fill_the_dropdown() {
        let css = Stylesheet::from(&*CASCADER_COLUMNS_SX);
        let css = css.as_str();

        assert!(css.contains("flex:1 0 auto;"), "{css}");
    }
}
