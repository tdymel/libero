use dioxus::prelude::*;

use crate::components::form::use_bound;

use crate::{
    components::{Input, Options, common::field_props},
    hooks::use_theme,
    utils::warn,
};

use super::core::{SelectCore, SelectionRenderArgs};

/// One row, handed to `Select`'s and `MultiSelect`'s `option` callback. It
/// draws the row's *content*: the row itself - its highlight, its
/// `aria-selected`, its click - is the component's.
#[derive(Clone, PartialEq)]
pub struct SelectOptionArgs<T> {
    pub value: T,
    pub index: usize,
    /// Whether this row is part of the selection, for a checkmark.
    pub selected: bool,
}

/// One selected value, handed to the `selection` callback of `MultiSelect`
/// and `FileField` alike. The chip's inner design is the caller's, `remove`
/// included - the control keeps only the keyboard.
///
/// Shared rather than duplicated: a second struct of `{ value, remove }`
/// differing by nothing would be the wart. There is no single-`Select`
/// counterpart because one selection is emptied by `clearable`.
#[derive(Clone, PartialEq)]
pub struct SelectionArgs<T> {
    pub value: T,
    /// Drops this value from the selection, which is the same edit as picking
    /// its row again.
    pub remove: Callback<()>,
}

/// One option under test, handed to `Select`'s and `MultiSelect`'s `filter`
/// callback while searching.
///
/// Deliberately the same pair `AutocompleteFilterArgs<T>` carries: two filter
/// args structs differing by nothing would be the wart.
#[derive(Clone, PartialEq)]
pub struct SelectFilterArgs<T> {
    pub value: T,
    /// What is currently typed in the search box.
    pub query: String,
}

field_props! {
    pub struct SelectProps<T: Options> {
        /// Strictly controlled - pair it with `onchange`. `None` shows
        /// `placeholder`.
        #[props(default)]
        value: Option<T>,
        /// Called with the option the caller should select next, or `None`
        /// when `clearable`'s x is clicked.
        #[props(default)]
        onchange: Option<EventHandler<Option<T>>>,
        /// The options to list. Defaults to every `Options::options()` - which
        /// `String` and any other runtime type leave empty, so those pass them
        /// here.
        #[props(default)]
        options: Option<Vec<T>>,
        /// Draws one row's content. Defaults to `Options::label`.
        #[props(default)]
        option: Option<Callback<SelectOptionArgs<T>, Element>>,
        /// Draws the selection inside the trigger. Defaults to the label as
        /// text.
        #[props(default)]
        selection: Option<Callback<T, Element>>,
        /// Shown while `value` is `None`.
        #[props(default)]
        placeholder: Option<String>,
        /// Emits a hidden input of that name carrying the selected option's
        /// `Options::value()`, so the select posts with a native form. The
        /// trigger is a `div`, so it cannot carry the name itself.
        /// A path - `Order::FIELDS.plan()` - also binds the selection to the
        /// surrounding `Form`'s value when there is no `onchange`.
        #[props(default, into)]
        name: crate::components::FieldName<Option<T>>,
        /// Rules over the selection, shown once the select loses focus or its
        /// form is submitted.
        #[props(default, into)]
        validate: crate::components::Validators<Option<T>>,
        /// Shows an x that clears the selection, which is what makes
        /// `onchange` fire `None`.
        #[props(default)]
        clearable: Option<bool>,
        /// Puts a search box at the top of the list.
        #[props(default)]
        searchable: Option<bool>,
        /// Narrows the options while searching. Defaults to a case-insensitive
        /// `contains` over `Options::label`.
        #[props(default)]
        filter: Option<Callback<SelectFilterArgs<T>, bool>>,
        /// What the search box says while empty.
        #[props(default)]
        search_placeholder: Option<String>,
    }
}

/// A listbox over an enum, with a label, a description, helper text and a
/// validation message stacked around it. Controlled: it renders `value` and
/// asks for a new one through `onchange`.
///
/// Unlike `NativeSelect`, the rows are libero's own, so they can be drawn with
/// anything through `option`. What that costs is the OS picker on phones and
/// working without wasm - reach for `NativeSelect` when those matter.
#[component]
pub fn Select<T: Options>(props: SelectProps<T>) -> Element {
    let theme = use_theme();

    let bound = use_bound(&props.name, props.onchange.is_some());
    let current = bound.value().unwrap_or_else(|| props.value.clone());

    if props.onchange.is_none() && !bound.is_bound() {
        warn("Select: without `onchange` the selection can never change.");
    }

    let values = props
        .options
        .clone()
        .unwrap_or_else(|| T::options().to_vec());
    if values.is_empty() {
        warn("Select: no options - a `T` without static `options()` needs `options`.");
    }
    let selected_index = current
        .as_ref()
        .and_then(|value| values.iter().position(|option| option == value));
    if current.is_some() && selected_index.is_none() && !values.is_empty() {
        warn("Select: `value` is not one of the options, so none is selected.");
    }

    let selected: Vec<bool> = (0..values.len())
        .map(|index| selected_index == Some(index))
        .collect();
    let rows = draw_rows(&values, &selected, props.option.as_ref());
    // A single selection has no chips, so the cursor the core hands down is
    // always `None` here.
    let draw_selection = props.selection;
    let selection = selected_index.map(|index| {
        let value = values[index].clone();
        Callback::new(move |_: SelectionRenderArgs| match &draw_selection {
            Some(selection) => selection.call(value.clone()),
            None => rsx! { "{value.label()}" },
        })
    });

    // Closes over this skin's own `values` and the caller's filter, so `T`
    // never reaches the core - it takes a `Vec<bool>` mask and nothing else.
    //
    // Built per render, which `Callback`'s `PartialEq` normally makes a trap:
    // two callbacks from one scope compare equal, so a stale one can survive.
    // It cannot here, because `SelectCore` also takes `rows: Vec<Element>`,
    // which never compares equal - its props therefore never do either, and the
    // fresh callback is always the one called. Anything that later drops `rows`
    // from those props has to revisit this.
    let searchable = props.searchable.unwrap_or(false);
    let filter = props.filter;
    let filtered = values.clone();
    let matches = searchable.then(|| {
        Callback::new(move |query: String| {
            let needle = query.to_lowercase();
            filtered
                .iter()
                .map(|value| match &filter {
                    Some(filter) => filter.call(SelectFilterArgs {
                        value: value.clone(),
                        query: query.clone(),
                    }),
                    None => value.label().to_lowercase().contains(&needle),
                })
                // Annotated: a `Callback`'s return type is inferred through
                // `SpawnIfAsync`, which leaves a bare `collect` ambiguous.
                .collect::<Vec<bool>>()
        })
    });

    let onchange = bound.emit(props.onchange);
    let clear = onchange.clone();
    rsx! {
        SelectCore {
            rows,
            selected,
            onpick: move |index: usize| {
                if let (Some(onchange), Some(value)) = (&onchange, values.get(index)) {
                    onchange(Some(value.clone()));
                }
            },
            selection,
            placeholder: props.placeholder,
            name: bound.name().map(str::to_string),
            form_values: vec![current.as_ref().map(Options::value).unwrap_or_default()],
            rules: props.validate.check(&current),
            clearable: props.clearable.unwrap_or(false),
            searchable,
            search_placeholder: props.search_placeholder,
            matches,
            onclear: move |_| {
                if let Some(clear) = &clear {
                    clear(None);
                }
            },
            label: props.label,
            description: props.description,
            helper: props.helper,
            status: props.status,
            size: props.size.copied_or(theme.select.size),
            radius: props.radius.copied_or(theme.select.radius),
            disabled: Some(bound.disabled(props.disabled)),
            required: props.required,
            class: props.class,
            sx: props.sx,
            states: props.states,
            attributes: props.attributes,
        }
    }
}

/// Each row's content - the caller's `option`, or the label.
pub(super) fn draw_rows<T: Options>(
    values: &[T],
    selected: &[bool],
    option: Option<&Callback<SelectOptionArgs<T>, Element>>,
) -> Vec<Element> {
    values
        .iter()
        .zip(selected)
        .enumerate()
        .map(|(index, (value, selected))| match option {
            Some(option) => option.call(SelectOptionArgs {
                value: value.clone(),
                index,
                selected: *selected,
            }),
            None => rsx! { "{value.label()}" },
        })
        .collect()
}
