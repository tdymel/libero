use std::collections::HashSet;

use dioxus::prelude::*;

use crate::{
    components::{ClassList, Input, OptionLabel, Options, States},
    sx::{Sx, ThemeAwareValue},
    theme::Size,
    utils::warn,
};

use super::{
    core::ComboboxCore,
    filter::{ComboboxFilterArgs, contains_ignoring_case},
    target::ComboboxTarget,
};

// Hand-written rather than `base_props!`, which is not generic.
#[derive(Props, Clone, PartialEq)]
pub struct ComboboxProps<T: Options> {
    /// Strictly controlled - pair it with `onchange`.
    #[props(default)]
    value: Option<T>,
    /// Called with what should be selected next. `None` is "nothing" - which
    /// a clear button in the target sends through `onclear`.
    #[props(default)]
    onchange: Option<EventHandler<Option<T>>>,
    /// The options to show. Defaults to every `Options::options()`.
    #[props(default)]
    options: Option<Vec<T>>,
    /// Overrides `Options::label`. Runs during render, so it can read a
    /// locale from context.
    ///
    /// Returns an `OptionLabel`, not a `String`: a row is a div, so it can
    /// hold an icon or a badge - and so can the target.
    #[props(default)]
    option_label: Option<Callback<T, OptionLabel>>,
    /// The whole control. `Combobox` renders no field of its own, so the
    /// target owns its look entirely - see [`ComboboxTarget`].
    target: Callback<ComboboxTarget, Element>,
    /// A search field above the options. `false` leaves a plain listbox.
    #[props(default = true)]
    searchable: bool,
    #[props(default)]
    search_placeholder: Option<String>,
    /// Whether an option survives the query. Defaults to a case-insensitive
    /// contains on the *resolved* label, so a translated option is searched
    /// by what it reads as.
    #[props(default)]
    filter: Option<Callback<ComboboxFilterArgs<T>, bool>>,
    /// Shown in place of the list when nothing matches.
    #[props(default)]
    empty: Option<Element>,
    /// Rows, the search field, and the row height virtualization assumes.
    #[props(default, into)]
    size: Input<Size>,
    /// The dropdown's corner radius.
    #[props(default, into)]
    radius: Input<Size>,
    /// A custom row's real height in px. Rows are virtualized against the
    /// themed row height, which a taller `option_label` outgrows.
    #[props(default)]
    option_height: Option<f64>,
    #[props(default, into)]
    max_dropdown_height: Input<ThemeAwareValue>,
    #[props(default)]
    disabled: Option<bool>,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default, into)]
    class: Input<ClassList>,
    /// Styles the dropdown - the wrapper it hangs off is scaffolding, not a
    /// user-facing element.
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
}

/// A listbox over an enum, with its own search field inside the dropdown.
/// Controlled: it renders `value` and asks for a new one through `onchange`.
///
/// The control itself is the caller's, through `target` - so the selection is
/// displayed exactly as the caller draws it, and `Combobox` never has to own a
/// field's styling.
///
/// Generic only at this boundary: the options are erased to indices here, and
/// everything below compiles once.
#[component]
pub fn Combobox<T: Options>(props: ComboboxProps<T>) -> Element {
    if props.onchange.is_none() {
        warn("Combobox: without `onchange` the selection can never change.");
    }
    if !props.searchable && props.filter.is_some() {
        warn("Combobox: `filter` does nothing while `searchable` is false.");
    }

    let values = props
        .options
        .clone()
        .unwrap_or_else(|| T::options().to_vec());
    if values.is_empty() {
        warn("Combobox: no options - a `T` without static `options()` needs `options`.");
    }

    let labels: Vec<OptionLabel> = values
        .iter()
        .map(|value| match &props.option_label {
            Some(label) => label.call(value.clone()),
            None => OptionLabel::from(value.label()),
        })
        .collect();

    let selected: HashSet<usize> = props
        .value
        .as_ref()
        .and_then(|value| values.iter().position(|option| option == value))
        .into_iter()
        .collect();
    if props.value.is_some() && selected.is_empty() && !values.is_empty() {
        warn("Combobox: `value` is not one of the options, so none is selected.");
    }

    let filter = props.filter;
    let filter_values = values.clone();
    let filter_labels = labels.clone();
    let matches = use_callback(move |(query, index): (String, usize)| {
        let Some(name) = filter_labels.get(index).map(OptionLabel::name) else {
            return false;
        };
        match &filter {
            Some(filter) => match filter_values.get(index) {
                Some(value) => filter.call(ComboboxFilterArgs {
                    query,
                    value: value.clone(),
                    name: name.to_string(),
                }),
                None => false,
            },
            None => contains_ignoring_case(&query, name),
        }
    });

    let onchange = props.onchange;
    let pick_values = values;
    let onpick = use_callback(move |index: usize| {
        if let (Some(onchange), Some(value)) = (&onchange, pick_values.get(index)) {
            onchange.call(Some(value.clone()));
        }
    });
    let onclear = use_callback(move |()| {
        if let Some(onchange) = &onchange {
            onchange.call(None);
        }
    });

    rsx! {
        ComboboxCore {
            labels,
            selected,
            onpick: move |index| onpick.call(index),
            onclear: move |()| onclear.call(()),
            matches,
            target: props.target,
            searchable: props.searchable,
            search_placeholder: props.search_placeholder,
            empty: props.empty,
            size: props.size,
            radius: props.radius,
            option_height: props.option_height,
            max_dropdown_height: props.max_dropdown_height,
            disabled: props.disabled.unwrap_or(false),
            attributes: props.attributes,
            class: props.class,
            sx: props.sx,
            states: props.states,
        }
    }
}
