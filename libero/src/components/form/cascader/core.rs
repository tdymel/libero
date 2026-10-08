use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        accessibility::VisuallyHidden,
        common::{HtmlTag, Input, Part, Parts, input_from_str},
        form::{
            Asks, ComboboxState, DropdownPart, clear_button, combobox::nothing_found_row,
            field_parts_enum, field_props, use_combobox, use_field, use_field_frame,
            use_refocus_on_close, with_drawn_placeholder,
        },
        layout::use_box,
    },
    hooks::{TYPEAHEAD_RESET, use_element, use_localization, use_theme, use_typeahead},
    str_enum::str_enum,
    theme::Size,
};

use super::{
    body::{Body, use_cascader_body},
    dropdown::{Dropdown, DropdownSetup, dropdown_box, use_cascader_dropdown},
    keys::CascaderKeys,
    nodes::{FlatPath, flatten_paths, join_labels},
    option::CascaderNode,
    rows::CascaderRows,
    search::{CascaderSearch, SEARCH_INSET, search_header},
    trigger::{CASCADER_TRIGGER_SX, Trigger, cascader_trigger},
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

/// Below the theme's `sm`, where two columns no longer fit a phone.
pub(super) fn narrow_query() -> String {
    format!("not all and (min-width: {})", Size::Sm.breakpoint_value())
}

field_parts_enum! {
    /// [`Cascader`](super::Cascader)'s inner parts, for its `parts` prop: a
    /// field's, and the value.
    pub enum CascaderPart framed {
        /// The joined path or the placeholder, in the trigger.
        Value = "value" => "& > * > [data-slot='frame'] > [data-slot='control'] > [data-slot='value']",
    }
}

field_props! {
    parts(CascaderPart);
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
        #[props(default, into)]
        dropdown_parts: Input<Parts<DropdownPart>>,
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

    let blurred = use_refocus_on_close(opened, searchable, trigger_element, query);

    let field = use_field()
        .labelled_by()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(props.rules.clone())
        .name(props.name.as_deref())
        .required(required)
        .empty(committed.is_none())
        .asks(Asks::Select)
        .disabled(disabled)
        .size(size)
        .radius(radius)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&props.states)
        .attributes(&props.attributes)
        .prepare();

    let clear = clear_button(
        props.clearable && committed.is_some() && !disabled && !readonly,
        size,
        trigger_element,
        Some(&field),
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
        trigger: trigger_element,
        blurred,
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
        state,
        field: field.root(),
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
        parts: &props.dropdown_parts,
    });

    // Drawn in the list's place and said by the status region below.
    let nothing_found = (searching && visible.is_empty()).then_some(nothing_found);
    // What a query left, said but not shown (WCAG 4.1.3, todo 2035).
    let results =
        (searching && !visible.is_empty()).then(|| (words.combobox.results)(visible.len()));
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
                    blurred,
                },
                // No listbox while nothing matches; a dangling id is invalid (todo 2291).
                nothing_found.is_none().then(|| controlled_id.clone()),
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
        true => control
            .attr("data-slot", CascaderPart::Control.slot())
            .attr("id", field.id().to_string()),
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
            if let Some(text) = results {
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
    // Trimmed, as the other search boxes: a trailing space must not hide a match.
    let needle = query.trim().to_lowercase();
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

pub(super) fn option_id(id: &str, level: usize, index: usize) -> String {
    format!("{id}-option-{level}-{index}")
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

/// The listbox `aria-controls` names: in `Columns` the cursor's column, since the strip is only `presentation`.
pub(super) fn controlled_id(listbox_id: &str, layout: CascaderLayout, cursor: &[usize]) -> String {
    match layout {
        CascaderLayout::Paths => listbox_id.to_string(),
        CascaderLayout::Columns => format!("{listbox_id}-{}", cursor.len().saturating_sub(1)),
    }
}

/// The row `aria-activedescendant` names: the cursor's deepest node, or its `Paths` row.
pub(super) fn active_descendant(
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
