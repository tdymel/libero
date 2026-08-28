use dioxus::prelude::*;

use crate::{
    components::{Input, Options, common::field_props},
    hooks::use_theme,
    utils::warn,
};

use super::core::SelectCore;

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
        /// Shows an x that clears the selection, which is what makes
        /// `onchange` fire `None`.
        #[props(default)]
        clearable: Option<bool>,
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

    if props.onchange.is_none() {
        warn("Select: without `onchange` the selection can never change.");
    }

    let values = props
        .options
        .clone()
        .unwrap_or_else(|| T::options().to_vec());
    if values.is_empty() {
        warn("Select: no options - a `T` without static `options()` needs `options`.");
    }
    let selected_index = props
        .value
        .as_ref()
        .and_then(|value| values.iter().position(|option| option == value));
    if props.value.is_some() && selected_index.is_none() && !values.is_empty() {
        warn("Select: `value` is not one of the options, so none is selected.");
    }

    let selected: Vec<bool> = (0..values.len())
        .map(|index| selected_index == Some(index))
        .collect();
    let rows = draw_rows(&values, &selected, props.option.as_ref());
    let selection = selected_index.map(|index| {
        let value = values[index].clone();
        match &props.selection {
            Some(selection) => selection.call(value),
            None => rsx! { "{value.label()}" },
        }
    });

    let onchange = props.onchange;
    rsx! {
        SelectCore {
            rows,
            selected,
            onpick: move |index: usize| {
                if let (Some(onchange), Some(value)) = (&onchange, values.get(index)) {
                    onchange.call(Some(value.clone()));
                }
            },
            selection,
            placeholder: props.placeholder,
            clearable: props.clearable.unwrap_or(false),
            onclear: move |_| {
                if let Some(onchange) = &onchange {
                    onchange.call(None);
                }
            },
            label: props.label,
            description: props.description,
            helper: props.helper,
            status: props.status,
            size: props.size.copied_or(theme.select.size),
            radius: props.radius.copied_or(theme.select.radius),
            disabled: props.disabled,
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
