use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        common::{Input, Options},
        form::{field_props, use_bound},
    },
    hooks::use_theme,
    utils::warn,
};

use super::{
    core::{
        CascaderCore, CascaderLayout, CascaderMatch, CascaderPart, CascaderRender, CascaderRowArgs,
        CascaderTree,
    },
    option::{CascaderOption, erase, indices_for_value, options_at},
};

/// One row's content, handed to `node`. The row itself (highlight, chevron, click) stays the component's.
///
/// `value` and `label`, not the option: a root option would clone its whole subtree per row.
#[derive(Clone, PartialEq)]
pub struct CascaderNodeArgs<T> {
    pub value: T,
    pub label: String,
    /// 0 for a root.
    pub level: usize,
    /// Whether the column to the right is this option's children.
    pub expanded: bool,
    /// Whether this option holds the committed value.
    pub selected: bool,
}

/// One path under test, handed to `filter` while searching.
#[derive(Clone, PartialEq)]
pub struct CascaderFilterArgs<T> {
    /// What is currently typed in the search box.
    pub query: String,
    /// The path's labels joined by `separator`, what the default filter matches.
    pub label: String,
    /// The values root to option; the last one is what this path commits.
    pub path: Vec<T>,
}

field_props! {
    parts(CascaderPart);
    pub struct CascaderProps<T: Options> {
        /// The tree. Values must be unique across the whole tree, not just among siblings.
        data: Vec<CascaderOption<T>>,
        /// The selected option's value. Strictly controlled: pair it with `onchange`, or bind it with `name`.
        #[props(default)]
        value: Option<T>,
        /// The value to select next, or `None` when cleared (the x, or `allow_deselect`).
        #[props(default)]
        onchange: Option<EventHandler<Option<T>>>,
        /// A branch commits its own value as well as expanding. Off, only a leaf can be picked.
        #[props(default)]
        any_level: Option<bool>,
        /// Picking the committed option again clears it. Off by default, as in APG.
        #[props(default)]
        allow_deselect: Option<bool>,
        /// `"columns"` (default) or `"paths"`. A search always renders `"paths"`.
        #[props(default, into)]
        layout: Input<CascaderLayout>,
        /// Puts a search box at the top of the list.
        #[props(default)]
        searchable: Option<bool>,
        /// Narrows the paths while searching. Defaults to a case-insensitive `contains` over the joined path.
        #[props(default)]
        filter: Option<Callback<CascaderFilterArgs<T>, bool>>,
        /// Between labels, in the trigger and in a `"paths"` row. Defaults to `" / "`.
        #[props(default)]
        separator: Option<String>,
        /// The trigger's text from the labels, root first. A `String`, so it can clip with an ellipsis.
        #[props(default)]
        format_value: Option<Callback<Vec<String>, String>>,
        /// Draws one row's content. Defaults to the label.
        #[props(default)]
        node: Option<Callback<CascaderNodeArgs<T>, Element>>,
        /// One column's minimum width; `"max-content"` fits the longest row. Defaults to the theme's.
        #[props(default)]
        column_width: Option<String>,
        /// Shows an x that clears the selection.
        #[props(default)]
        clearable: Option<bool>,
        /// Shown while nothing is selected.
        #[props(default)]
        placeholder: Option<String>,
        /// What the search box says while empty.
        #[props(default)]
        search_placeholder: Option<String>,
        /// A hidden input posting `Options::value()`. A field path also binds to the `Form` without `onchange`.
        #[props(default, into)]
        name: crate::components::form::FieldName<Option<T>>,
        /// Rules over the selected value, shown after blur or submit.
        #[props(default, into)]
        validate: crate::components::form::Validators<Option<T>>,
    }
}

/// A select over a tree, walked level by level; the trigger shows the path to the picked option.
///
/// Only this shell is generic: the engine walks the tree by index path and never sees a `T`.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Cascader, CascaderOption};
/// # fn app() -> Element {
/// let mut chosen = use_signal(|| None::<String>);
/// rsx! {
///     Cascader {
///         label: "Category",
///         data: vec![CascaderOption::new("fruit", "Fruit").children(vec![
///             CascaderOption::new("apple", "Apple"),
///         ])],
///         value: chosen(),
///         onchange: move |next| chosen.set(next),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/cascader>
#[component]
pub fn Cascader<T: Options>(props: CascaderProps<T>) -> Element {
    let theme = use_theme();

    let bound = use_bound(&props.name, props.onchange.is_some());
    let current = bound.value().unwrap_or_else(|| props.value.clone());

    if props.onchange.is_none() && !bound.is_bound() {
        warn("Cascader: without `onchange` the selection can never change.");
    }
    if props.data.is_empty() {
        warn("Cascader: `data` is empty, so there is nothing to pick.");
    }

    let separator = props
        .separator
        .clone()
        .unwrap_or_else(|| String::from(" / "));
    let committed = current
        .as_ref()
        .and_then(|value| indices_for_value(&props.data, value));
    if current.is_some() && committed.is_none() {
        warn("Cascader: no option in `data` holds `value`, so nothing is selected.");
    }

    let display = match &committed {
        None => String::new(),
        Some(indices) => {
            let labels: Vec<String> = options_at(&props.data, indices)
                .iter()
                .map(|option| option.label.clone())
                .collect();
            match &props.format_value {
                Some(format) => format.call(labels),
                None => labels.join(&separator),
            }
        }
    };

    let rules = props.validate.check(&current);
    let form_value = current.as_ref().map(Options::value);
    let emit = bound.emit(props.onchange);

    // Erased fresh each render on purpose - see `CascaderCoreProps::options`.
    let options = CascaderTree(Rc::new(erase(&props.data)));
    // Shared by the three closures that map an index path back to a `T`.
    let data = Rc::new(props.data);

    let onpick = {
        let data = data.clone();
        move |next: Option<Vec<usize>>| {
            let Some(emit) = &emit else {
                return;
            };
            let value = next.and_then(|indices| {
                options_at(&data, &indices)
                    .last()
                    .map(|option| option.value.clone())
            });
            emit(value);
        }
    };

    let node = props.node.map(|draw| {
        let data = data.clone();
        CascaderRender(Rc::new(move |row: CascaderRowArgs| {
            match options_at(&data, &row.indices).last() {
                Some(option) => draw.call(CascaderNodeArgs {
                    value: option.value.clone(),
                    label: option.label.clone(),
                    level: row.indices.len().saturating_sub(1),
                    expanded: row.expanded,
                    selected: row.selected,
                }),
                None => rsx! {},
            }
        }))
    });

    let filter = props.filter.map(|filter| {
        let data = data.clone();
        CascaderMatch(Rc::new(
            move |query: &str, indices: &[usize], label: String| {
                filter.call(CascaderFilterArgs {
                    query: query.to_string(),
                    label,
                    path: options_at(&data, indices)
                        .iter()
                        .map(|option| option.value.clone())
                        .collect(),
                })
            },
        ))
    });

    rsx! {
        CascaderCore {
            options,
            committed,
            onpick,
            node,
            filter,
            form_value,
            display,
            separator,
            any_level: props.any_level.unwrap_or(false),
            allow_deselect: props.allow_deselect.unwrap_or(false),
            layout: props.layout.copied_or_default(),
            searchable: props.searchable.unwrap_or(false),
            column_width: props
                .column_width
                .clone()
                .unwrap_or_else(|| theme.cascader.column_width.to_string()),
            placeholder: props.placeholder,
            search_placeholder: props.search_placeholder,
            clearable: props.clearable.unwrap_or(false),
            name: bound.name().map(str::to_string),
            rules,
            label: props.label,
            description: props.description,
            helper: props.helper,
            status: props.status,
            size: props.size.copied_or(theme.cascader.size),
            radius: props.radius.copied_or(theme.cascader.radius),
            disabled: Some(bound.disabled(props.disabled)),
            readonly: props.readonly,
            required: props.required,
            class: props.class,
            sx: props.sx,
            parts: props.parts,
            states: props.states,
            attributes: props.attributes,
        }
    }
}
