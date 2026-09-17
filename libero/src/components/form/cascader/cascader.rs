use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{Input, Options, common::field_props, form::use_bound},
    hooks::use_theme,
    utils::warn,
};

use super::{
    core::{
        CascaderCore, CascaderLayout, CascaderMatch, CascaderRender, CascaderRowArgs, CascaderTree,
    },
    option::{CascaderOption, erase, indices_for_value, options_at},
};

/// One row under the `node` callback's brush. The row itself - its highlight,
/// its `aria-selected`, its chevron, its click - is the component's.
///
/// The option's `value` and `label` rather than the option itself: a root's
/// option carries its whole subtree, which every visible row would clone.
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
    /// The path's labels, already joined by `separator` - what the default
    /// filter matches against.
    pub label: String,
    /// The values root to option; the last one is what this path commits.
    pub path: Vec<T>,
}

field_props! {
    pub struct CascaderProps<T: Options> {
        /// The tree to walk. Values are unique across the whole tree, not
        /// just among siblings - the cascader finds its value by searching.
        data: Vec<CascaderOption<T>>,
        /// The selected option's value. Strictly controlled - pair it with
        /// `onchange`, or bind it with `name`. The path to it is found in
        /// `data`, for the trigger's text and for where the list opens.
        #[props(default)]
        value: Option<T>,
        /// Called with the value to select next, or `None` when the
        /// selection was cleared - by the x, or by `allow_deselect`.
        #[props(default)]
        onchange: Option<EventHandler<Option<T>>>,
        /// A branch commits its own value as well as expanding. Off, only a leaf can be picked.
        #[props(default)]
        any_level: Option<bool>,
        /// Picking the committed option again clears it. Off by default:
        /// APG keeps the value on a re-pick, so Enter only confirms.
        #[props(default)]
        allow_deselect: Option<bool>,
        /// `"columns"` (default) or `"paths"`. A search renders `"paths"`
        /// whatever this says.
        #[props(default, into)]
        layout: Input<CascaderLayout>,
        /// Puts a search box at the top of the list.
        #[props(default)]
        searchable: Option<bool>,
        /// Narrows the paths while searching. Defaults to a case-insensitive
        /// `contains` over the joined path.
        #[props(default)]
        filter: Option<Callback<CascaderFilterArgs<T>, bool>>,
        /// Between labels, in the trigger and in a `"paths"` row. `" / "` by
        /// default.
        #[props(default)]
        separator: Option<String>,
        /// Overrides the joined labels in the trigger; takes the labels root
        /// to option. A `String`, not a node: the value slot clips for
        /// `text-overflow: ellipsis`.
        #[props(default)]
        format_value: Option<Callback<Vec<String>, String>>,
        /// Draws one row's content. Defaults to the label.
        #[props(default)]
        node: Option<Callback<CascaderNodeArgs<T>, Element>>,
        /// One column's width, and its minimum: when the trigger is wider
        /// than the open columns, they share the rest. Defaults to the
        /// theme's; `"max-content"` is how a column takes the width of its
        /// longest row.
        #[props(default)]
        column_width: Option<String>,
        /// Shows an x that clears the selection, which is what makes
        /// `onchange` fire `None`.
        #[props(default)]
        clearable: Option<bool>,
        /// Shown while nothing is selected.
        #[props(default)]
        placeholder: Option<String>,
        /// What the search box says while empty.
        #[props(default)]
        search_placeholder: Option<String>,
        /// Emits a hidden input of that name carrying the selected value's
        /// `Options::value()`, so the field posts with a native form. A path -
        /// `Listing::FIELDS.category()` - also binds the selection to the
        /// surrounding `Form`'s value when there is no `onchange`.
        #[props(default, into)]
        name: crate::components::FieldName<Option<T>>,
        /// Rules over the selected value, shown once the cascader loses focus
        /// or its form is submitted.
        #[props(default, into)]
        validate: crate::components::Validators<Option<T>>,
    }
}

/// A listbox over a tree, walked level by level. Its value is one option's
/// `value` - any `T` a `Select` could hold; what it adds is the path to that
/// option, which it finds in `data` itself and shows in the trigger.
///
/// It takes its own `CascaderOption<T>`. Only this shell is
/// generic: the engine under it walks the tree by index path and never sees a
/// `T`, so a second `T` costs a few small functions, not a second engine. Unlike `Tree` it is a
/// field: it sits in a frame with a label, it posts, it validates, and focus
/// never leaves its trigger - the columns are listboxes the trigger points at
/// with `aria-activedescendant`.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{Cascader, CascaderOption};
/// # fn app() -> Element {
/// # type Category = String;
/// # let categories = use_signal(Vec::<CascaderOption<Category>>::new);
/// # let mut chosen = use_signal(|| None::<Category>);
/// # rsx! {
/// Cascader {
///     label: "Category",
///     data: categories(),
///     value: chosen(),
///     onchange: move |next: Option<Category>| chosen.set(next),
///     searchable: true,
/// }
/// # } }
/// ```
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

    // The trigger's text. A `String` and not a node, so the value slot can
    // clip it for `text-overflow: ellipsis`.
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
            states: props.states,
            attributes: props.attributes,
        }
    }
}
