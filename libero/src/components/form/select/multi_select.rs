use dioxus::prelude::*;

use crate::{
    components::{Chip, Input, Options, common::field_props},
    hooks::use_theme,
    theme::Size,
    utils::warn,
};

use super::{
    core::SelectCore,
    select::{SelectFilterArgs, SelectOptionArgs, draw_rows},
};

field_props! {
    pub struct MultiSelectProps<T: Options> {
        /// Strictly controlled - pair it with `onchange`. Empty shows
        /// `placeholder`.
        #[props(default)]
        value: Vec<T>,
        /// Called with the whole selection the caller should hold next.
        #[props(default)]
        onchange: Option<EventHandler<Vec<T>>>,
        /// The options to list. Defaults to every `Options::options()` - which
        /// `String` and any other runtime type leave empty, so those pass them
        /// here.
        #[props(default)]
        options: Option<Vec<T>>,
        /// Draws one row's content. Defaults to `Options::label`.
        #[props(default)]
        option: Option<Callback<SelectOptionArgs<T>, Element>>,
        /// Draws one selected value inside the trigger. Defaults to the label
        /// in a `Chip`.
        #[props(default)]
        selection: Option<Callback<T, Element>>,
        /// Shown while `value` is empty.
        #[props(default)]
        placeholder: Option<String>,
        /// Shows an x that empties the selection.
        #[props(default)]
        clearable: Option<bool>,
        /// Puts a search box at the top of the list. The query survives a pick,
        /// so several matches of one search can be ticked without retyping it.
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

/// A listbox over an enum that holds any number of its options.
///
/// Stays open on a pick, and a pick toggles the row: Escape, clicking
/// elsewhere and the trigger close it. The selection is drawn in the trigger,
/// as chips unless `selection` says otherwise - in the order it was picked.
#[component]
pub fn MultiSelect<T: Options>(props: MultiSelectProps<T>) -> Element {
    let theme = use_theme();

    if props.onchange.is_none() {
        warn("MultiSelect: without `onchange` the selection can never change.");
    }

    let values = props
        .options
        .clone()
        .unwrap_or_else(|| T::options().to_vec());
    if values.is_empty() {
        warn("MultiSelect: no options - a `T` without static `options()` needs `options`.");
    }

    let selected: Vec<bool> = values
        .iter()
        .map(|option| props.value.contains(option))
        .collect();
    let rows = draw_rows(&values, &selected, props.option.as_ref());
    let selection = (!props.value.is_empty()).then(|| {
        let chips = props.value.iter().map(|value| match &props.selection {
            Some(selection) => selection.call(value.clone()),
            None => rsx! {
                Chip { size: Size::Xs, "{value.label()}" }
            },
        });
        rsx! {
            for (index, chip) in chips.enumerate() {
                Fragment { key: "{index}", {chip} }
            }
        }
    });

    // The same mask `Select` builds, and the same memoization reasoning - see
    // the comment there.
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
                // Annotated for the same reason as in `select.rs`.
                .collect::<Vec<bool>>()
        })
    });

    let onchange = props.onchange;
    let current = props.value.clone();
    rsx! {
        SelectCore {
            rows,
            selected,
            multiple: true,
            onpick: move |index: usize| {
                let (Some(onchange), Some(value)) = (&onchange, values.get(index)) else {
                    return;
                };
                let mut next = current.clone();
                match next.iter().position(|picked| picked == value) {
                    Some(at) => {
                        next.remove(at);
                    }
                    None => next.push(value.clone()),
                }
                onchange.call(next);
            },
            selection,
            placeholder: props.placeholder,
            clearable: props.clearable.unwrap_or(false),
            searchable,
            search_placeholder: props.search_placeholder,
            matches,
            onclear: move |_| {
                if let Some(onchange) = &onchange {
                    onchange.call(Vec::new());
                }
            },
            label: props.label,
            description: props.description,
            helper: props.helper,
            status: props.status,
            size: props.size.copied_or(theme.multi_select.size),
            radius: props.radius.copied_or(theme.multi_select.radius),
            disabled: props.disabled,
            required: props.required,
            class: props.class,
            sx: props.sx,
            states: props.states,
            attributes: props.attributes,
        }
    }
}
