use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        Box, ComboboxOption, HtmlTag, Input, States,
        common::{ChevronDownIcon, attr, field_props, input_from_str},
        form::{
            ComboboxState, clear_button, combobox::COMBOBOX_DROPDOWN_SX, field_control_sx,
            use_combobox, use_field, use_field_frame, use_refocus_on_close,
        },
        layout::{BoxStyle, ScrollArea, use_box},
    },
    hooks::{
        ElementHandle, PopoverOptions, PopoverWidth, use_element, use_field_list_layer,
        use_popover, use_theme,
    },
    platform::ElementApi,
    str_enum::str_enum,
    sx::{StaticSx, Sx, sx},
    theme::Size,
};

use super::{
    nodes::{
        FlatPath, children_at, disabled_at, first_enabled, flatten_paths, join_labels,
        last_enabled, node_at, step,
    },
    option::CascaderNode,
};

str_enum! {
    /// How an open `Cascader` draws its tree.
    #[state_prefix = "layout"]
    pub enum CascaderLayout {
        /// One listbox per level, side by side - the column walk the
        /// component is named after.
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

/// The option tree without its values, compared by `Rc` pointer. `Cascader`
/// erases its `data` into a fresh one every render, so `CascaderCoreProps`
/// never compares equal - see `CascaderCoreProps::options`.
#[derive(Clone)]
pub(super) struct CascaderTree(pub Rc<Vec<CascaderNode>>);

impl PartialEq for CascaderTree {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

/// One row as the engine knows it: where it is, not what it holds.
/// `Cascader<T>` looks the option up by `indices` for the caller's `node`.
pub(super) struct CascaderRowArgs {
    pub indices: Vec<usize>,
    pub expanded: bool,
    pub selected: bool,
}

/// `Rc<dyn Fn>` wrapper, so `CascaderCoreProps` derives `Clone`/`PartialEq`
/// without being generic over `T`. Always equal, like `Tree`'s
/// `ErasedRenderNode`; what keeps a *stale* one from surviving is `options`.
#[derive(Clone)]
pub(super) struct CascaderRender(pub Rc<dyn Fn(CascaderRowArgs) -> Element>);

impl PartialEq for CascaderRender {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

/// The query, a path's index path and its joined label in; whether the path
/// survives out.
type MatchFn = dyn Fn(&str, &[usize], String) -> bool;

/// A caller's `filter`, erased the same way as `CascaderRender`.
#[derive(Clone)]
pub(super) struct CascaderMatch(pub Rc<MatchFn>);

impl PartialEq for CascaderMatch {
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
        .selector("& > [data-placeholder]", sx().color("text-dimmed"))
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
        // Only where the dropdown is held narrower than its columns, which
        // on a desktop it never is.
        .overflow_x("auto")
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
        .selector("::placeholder", sx().color("text-dimmed"))
});

field_props! {
    pub(crate) struct CascaderCoreProps {
        /// The tree, **wrapped fresh on every render on purpose**.
        /// `CascaderTree` compares by `Rc` pointer, so these props never
        /// compare equal - which is what stops this component memoizing and
        /// keeping a stale `node` or `filter`, both of which always compare
        /// equal ([[codebase/dioxus-memoization-traps]]). It also spares a
        /// deep comparison of the tree.
        options: CascaderTree,
        /// The committed option's index path. `None` is no selection.
        committed: Option<Vec<usize>>,
        /// The option to commit next, by index path - `None` to clear.
        onpick: EventHandler<Option<Vec<usize>>>,
        /// Draws one row's content. `None` draws the label.
        #[props(default)]
        node: Option<CascaderRender>,
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
        /// Shows an x in place of the chevron while a value is committed.
        #[props(default)]
        clearable: bool,
        /// A hidden input of that name carrying `form_value`, so the field
        /// posts with a native form. The trigger is a `div` and cannot carry
        /// a `name` itself.
        #[props(default)]
        name: Option<String>,
        /// What the hidden input posts - the value's `Options::value()`.
        /// `None` posts nothing.
        #[props(default)]
        form_value: Option<String>,
        /// What the skin's `validate` rules say.
        #[props(default)]
        rules: Option<crate::components::FieldStatus>,
        /// Narrows the paths while searching. `None` is a case-insensitive
        /// `contains` over the joined labels.
        #[props(default)]
        filter: Option<CascaderMatch>,
    }
}

/// The engine under `Cascader`. It never sees `T`: the skin hands it the tree
/// without its values, the committed option as an index path, and erased
/// callbacks, and takes an index path back.
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
    // Like `SelectCore`: the tree is the only editor, so read-only leaves the
    // trigger focusable and posting and refuses to open it.
    let readonly = props.readonly.unwrap_or(false);
    let required = props.required.unwrap_or(false);

    let state = use_combobox();
    let opened = state.is_open() && !disabled && !readonly;
    let searchable = props.searchable && !disabled;

    // One index per level. `[2, 0]` highlights the first child of the third
    // root, and - Mantine's `getCascaderColumns` rule - does *not* expand it:
    // the columns never run ahead of the cursor.
    let cursor = use_signal(Vec::<usize>::new);
    let query = use_signal(String::new);
    let search = use_element();
    let trigger_element = use_element();

    let nodes = props.options.0.clone();
    let any_level = props.any_level;

    let committed = props.committed.clone();
    let searching = searchable && opened && !query().is_empty();
    let layout = match searching {
        true => CascaderLayout::Paths,
        false => props.layout,
    };

    // Every path the `Paths` layout could draw, and which of them the query
    // leaves. `Columns` never reads it, and building it costs one walk of a
    // tree the component is holding anyway.
    let visible: Rc<Vec<FlatPath>> = Rc::new(visible_paths(
        &nodes,
        any_level,
        searching.then(|| query.read().clone()).as_deref(),
        &props.separator,
        props.filter.as_ref(),
    ));

    let cursor_now = cursor.read().clone();
    // The cursor as a row of `visible`, which is the `Paths` keyboard's index.
    let path_row = visible.iter().position(|path| path.indices == cursor_now);

    let id = state.id();
    let listbox_id = format!("{id}-listbox");

    // `Rc` rather than a bare closure: each is needed in two or more handlers,
    // and what they capture - a path - is not `Copy`.
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

    // The frame draws the ring, so the trigger must not draw a second one.
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
    };
    let max_height = theme.combobox.max_dropdown_height;
    // The strip of columns, which scrolls sideways once the viewport holds
    // the dropdown narrower than them. The cursor always sits in the last
    // column, so each new column is scrolled into view as it opens - by
    // pointer or by the arrows, which would otherwise move into a column no
    // one can see.
    let strip = use_element();
    let depth = cursor_now.len();
    use_effect(use_reactive!(|depth| {
        let _ = depth;
        // Read, not branched on: it subscribes the effect to every reopen,
        // which mounts a new strip at the same handle.
        if strip.mount_token().is_some() {
            // Past the end on purpose: the browser clamps it to the range.
            let _ = strip.scroll_to(f64::from(u32::MAX), 0.0);
        }
    }));
    let columns = use_box()
        .framework_sx(&CASCADER_COLUMNS_SX)
        .prepare()
        .element(&strip);
    let body = match layout {
        CascaderLayout::Columns => rows.columns(
            columns,
            &listbox_id,
            field.label_id(),
            &props.column_width,
            max_height,
        ),
        CascaderLayout::Paths => rows.paths(
            &visible,
            path_row,
            &listbox_id,
            field.label_id(),
            max_height,
        ),
    };

    // ---- the dropdown -------------------------------------------------
    // On the Escape stack exactly while the key handler would take
    // Escape, so a `HoverCard` around this field leaves the press to it.
    use_field_list_layer(opened);
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
        // Held inside the viewport by `use_popover`'s own cap: three columns
        // side by side are wider than a phone, and a fixed box that runs off
        // the right edge cannot be scrolled to - the columns scroll inside it
        // instead.
        .framework_sx(&COMBOBOX_DROPDOWN_SX)
        .states(&dropdown_states)
        .style(popover.style())
        .prepare();

    use_focus_search(opened, searchable, popover.placed(), search);

    let descendant = active_descendant(&id, layout, &cursor_now, path_row);
    let header = searchable.then(|| {
        search_header(
            search_box,
            CascaderSearch {
                element: search,
                query,
                cursor,
                state,
            },
            listbox_id.clone(),
            descendant.clone(),
            props.search_placeholder.clone().unwrap_or_default(),
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
    let mut attributes = trigger_aria(searchable, opened, &listbox_id, descendant);
    attributes.extend(props.attributes);

    let value_slot = value_slot(&props.display, props.placeholder.as_deref());

    let toggle = open.clone();
    let trigger = field
        .aria(control)
        .element(&trigger_element)
        .attr("aria-labelledby", field.label_id())
        .attr("aria-disabled", disabled.then_some("true"))
        .attr("aria-readonly", readonly.then_some("true"))
        .attr("tabindex", (!disabled).then_some("0"))
        .event("onclick", move |_: MouseEvent| {
            if !disabled && !readonly {
                toggle(!state.is_open());
            }
        })
        // While searchable the focus moves into the search box, so closing on
        // the trigger's blur would shut the list before a key could land.
        .event("onblur", move |_: FocusEvent| {
            if !searchable {
                state.close();
            }
        })
        // On the trigger, not on a wrapper around the frame: the frame also
        // holds the x, and a wrapper would take its Enter and Space to open
        // the list instead of letting the button clear.
        .event("onkeydown", move |event: KeyboardEvent| keys.handle(event))
        .render(
            HtmlTag::Div,
            attributes,
            rsx! {
                {value_slot}
                if clear.is_none() {
                    ChevronDownIcon {}
                }
            },
        );

    let hidden = hidden_input(props.name.clone(), props.form_value.clone(), disabled);

    field.render(rsx! {
        {
            wrapper
                .element(&anchor)
                .render(HtmlTag::Div, Vec::new(), frame.render(trigger))
        }
        {hidden}
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

/// Two elements cannot both be the combobox. While the search box is open it
/// owns the role, `aria-controls` and `aria-activedescendant`; the trigger
/// keeps only what says a list hangs off it.
fn trigger_aria(
    searchable: bool,
    opened: bool,
    listbox_id: &str,
    descendant: Option<String>,
) -> Vec<Attribute> {
    if searchable && opened {
        return vec![
            attr("aria-haspopup", "listbox"),
            attr("aria-expanded", "true"),
        ];
    }
    let mut trigger = vec![
        attr("role", "combobox"),
        attr("aria-haspopup", "listbox"),
        attr("aria-expanded", opened.to_string()),
    ];
    // The list is mounted only while open, and an id that names nothing is an
    // invalid reference.
    if opened {
        trigger.push(attr("aria-controls", listbox_id.to_string()));
    }
    if let Some(target) = descendant.filter(|_| opened) {
        trigger.push(attr("aria-activedescendant", target));
    }
    trigger
}

/// The keyboard: two tables on one handler, chosen by the layout that is on
/// screen. `Columns` walks the tree; `Paths` walks a flat list and leaves Left
/// and Right to the search box's caret. Held in an `Rc`, because the trigger
/// and the portaled dropdown both need it.
struct CascaderKeys {
    nodes: Rc<Vec<CascaderNode>>,
    visible: Rc<Vec<FlatPath>>,
    cursor: Signal<Vec<usize>>,
    state: ComboboxState,
    open: Rc<dyn Fn(bool)>,
    commit: Rc<dyn Fn(Vec<usize>, bool)>,
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
            Key::Tab => self.state.close(),
            // The page must not scroll under an open list. While `searchable`
            // the focus is in the search box, where a space is ordinary
            // typing.
            Key::Character(ref character) if character == " " && !self.searchable => {
                event.prevent_default();
            }
            Key::ArrowDown | Key::ArrowUp | Key::Home | Key::End | Key::Enter if paths_layout => {
                self.paths(event)
            }
            _ => self.columns(event),
        }
    }

    /// A closed list: the keys that open it.
    fn closed(&self, event: KeyboardEvent) {
        let mut cursor = self.cursor;
        let open = &self.open;
        match event.key() {
            Key::ArrowDown | Key::ArrowRight | Key::Enter => {
                event.prevent_default();
                open(true);
                if cursor.read().is_empty()
                    && let Some(index) = first_enabled(&self.nodes)
                {
                    cursor.set(vec![index]);
                }
            }
            Key::ArrowUp => {
                event.prevent_default();
                open(true);
                if cursor.read().is_empty()
                    && let Some(index) = last_enabled(&self.nodes)
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
    }

    /// The flat list: Up, Down, Home, End and Enter.
    fn paths(&self, event: KeyboardEvent) {
        let (mut cursor, visible) = (self.cursor, &self.visible);
        let here = cursor.read().clone();
        let row = visible.iter().position(|path| path.indices == here);
        let key = event.key();
        if key == Key::Enter {
            // Nothing highlighted means Enter is not ours: it bubbles, so a
            // form still submits.
            let Some(path) = row.and_then(|row| visible.get(row)) else {
                return;
            };
            event.prevent_default();
            if !path.disabled {
                // A `Paths` row is a whole path, so there is nothing left to
                // drill into - every pick here is the end.
                (self.commit)(path.indices.clone(), true);
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

    /// The tree. `Paths` has no levels to walk, so Left and Right are left
    /// alone there - which is what lets them move the search box's caret.
    fn columns(&self, event: KeyboardEvent) {
        let (mut cursor, nodes) = (self.cursor, &self.nodes);
        let paths_layout = self.layout == CascaderLayout::Paths;
        let here = cursor.read().clone();
        let key = event.key();
        match key {
            Key::ArrowDown | Key::ArrowUp | Key::Home | Key::End => {
                event.prevent_default();
                let (parents, from) = match here.split_last() {
                    Some((last, parents)) => (parents.to_vec(), Some(*last)),
                    None => (Vec::new(), None),
                };
                let column = children_at(nodes, &parents);
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
            Key::ArrowRight if !paths_layout => {
                event.prevent_default();
                let column = children_at(nodes, &here);
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
                if here.is_empty() || disabled_at(nodes, &here) {
                    return;
                }
                event.prevent_default();
                let children = children_at(nodes, &here);
                if children.is_empty() {
                    (self.commit)(here, true);
                    return;
                }
                // A branch expands. With `any_level` it is picked on the way,
                // and the list stays open so the walk can go on.
                if self.any_level {
                    (self.commit)(here.clone(), false);
                }
                if let Some(index) = first_enabled(children) {
                    let mut next = here;
                    next.push(index);
                    cursor.set(next);
                }
            }
            _ => {}
        }
    }
}

/// Draws the rows of both layouts, for one render.
struct CascaderRows {
    nodes: Rc<Vec<CascaderNode>>,
    node: Option<CascaderRender>,
    separator: String,
    /// The combobox's id, which every option id is built from.
    id: String,
    cursor: Signal<Vec<usize>>,
    /// The cursor as this render read it.
    cursor_now: Vec<usize>,
    /// The committed path, empty for none.
    committed: Vec<usize>,
    commit: Rc<dyn Fn(Vec<usize>, bool)>,
    any_level: bool,
    size: Size,
    radius: Size,
}

impl CascaderRows {
    /// One node, through the skin's renderer. A `Paths` row is several of these
    /// with the separator between them, so the caller's `node` still draws
    /// every level rather than being skipped for the flat layout.
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
        // On the cursor's own chain, which in each column is exactly one row -
        // so the row `aria-activedescendant` points at always carries
        // `aria-selected`, the APG contract a committed-only mark would break
        // the moment two columns are open.
        let on_cursor = self.cursor_now.starts_with(&indices);
        let is_cursor = self.cursor_now == indices;
        let is_committed = self.committed == indices;
        let content = match whole_path {
            false => self.node(&indices),
            // Every level, separated - a `Paths` row is the path, which is
            // what makes one row of it enough to pick by.
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
        // Clicking a branch puts the cursor on its first child, not on the
        // branch itself - the deepest highlighted node is never expanded, so
        // stopping on the branch would show no children at all.
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

    /// One column per level the cursor has reached, and the roots when it has
    /// reached none. The deepest highlighted node is *not* expanded, so the
    /// columns never run ahead of the cursor.
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
                // Named by the row it hangs off, so no column needs an English
                // literal to be announced by.
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
        strip
            .attr("id", listbox_id.to_string())
            .attr("role", "presentation")
            .render(
                HtmlTag::Div,
                Vec::new(),
                rsx! {
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

/// Opens or closes the list. The list opens on what is already committed,
/// like a native `<select>`; with nothing committed there is no highlight
/// until a key makes one.
fn open_handler(
    cursor: Signal<Vec<usize>>,
    state: ComboboxState,
    seed: Option<Vec<usize>>,
) -> Rc<dyn Fn(bool)> {
    Rc::new(move |next: bool| {
        // `Signal` is `Copy`, so a local copy is what lets an `Fn` closure
        // write one.
        let mut cursor = cursor;
        if next && !state.is_open() {
            cursor.set(seed.clone().unwrap_or_default());
        }
        state.set_open(next);
    })
}

/// The one place a value is committed, shared by the keyboard and the mouse.
/// `allow_deselect` turns a re-pick of `picked` into a clear, which is the
/// same edit the x makes. `close` is false for the one commit that is not the
/// end of the interaction: an `any_level` branch, which is picked *and*
/// drilled into.
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

/// What the search box reads and writes.
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
    listbox_id: String,
    descendant: Option<String>,
    placeholder: String,
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
        .attr("aria-controls", listbox_id)
        .attr("aria-activedescendant", descendant)
        .event("oninput", move |event: FormEvent| {
            query.set(event.value());
            // The list under the highlight just changed; nothing in the new
            // one is armed until an arrow says so.
            cursor.set(Vec::new());
        })
        // The trigger's blur no longer closes while searchable - this does,
        // and the rows and the list cancel `mousedown`, so a click inside
        // never reaches it.
        .event("onblur", move |_: FocusEvent| state.close())
        // **No `onkeydown` here.** The box is inside the portaled dropdown,
        // which carries the very same handler, so a second one would run the
        // whole table twice per key - and the second pass sees the state the
        // first left. An Enter that committed and closed was reopened by its
        // own second pass, measured in Chromium.
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

/// A hidden input is the only way a control that is not a form element can
/// post - the shape `Select` and `Slider` use. The value alone: the path is
/// derived from it, so the server needs nothing else. Nothing selected posts
/// nothing, as a `Select` does.
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

/// Focuses the search box once the open list is placed. Placed means
/// measured, which means visible - the first moment at which focusing anything
/// inside the box can take.
fn use_focus_search(opened: bool, searchable: bool, placed: bool, search: ElementHandle) {
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
        // Clicking the list's padding or its scrollbar must not move focus off
        // the trigger: a trigger that closes on blur would close under the
        // click. The rows cancel it for themselves already.
        .event("onmousedown", move |event: MouseEvent| {
            event.prevent_default()
        })
        // The same handler as the trigger's, because the dropdown is
        // portaled: it is no descendant of the trigger, so a key pressed
        // inside it would otherwise bubble to `PortalOutlet` and die.
        .event("onkeydown", move |event: KeyboardEvent| keys.handle(event))
        .render(HtmlTag::Div, Vec::new(), content)
}

/// Which row `aria-activedescendant` points at - the cursor's deepest node in
/// `Columns`, and its row in the flat list in `Paths`.
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

    /// Three columns are wider than a phone. The dropdown is held inside the
    /// viewport (by `use_popover`, see `tests/all/combobox.rs`) and the columns
    /// scroll sideways within it, rather than opening off the right edge where
    /// nothing can reach them.
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
