use dioxus::prelude::*;

use crate::components::form::use_bound;

use crate::{
    components::{Input, OptionSource, Options, common::field_props, form::removable_chip},
    hooks::use_theme,
    utils::warn,
};

use super::{
    core::{SelectCore, SelectionRenderArgs},
    select::{SelectFilterArgs, SelectOptionArgs, SelectionArgs, draw_rows},
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
        ///
        /// A `Vec<T>` converts, which is the flat list. An
        /// [`OptionList`](crate::components::OptionList) adds named groups and
        /// per-option `disabled`, and a [`Resource`] adds the fetch: the list
        /// then reads pending against ready itself, and derives the loader,
        /// `aria-busy` and the held-back empty state from it.
        #[props(default, into)]
        options: OptionSource<T>,
        /// Draws one row's content. Defaults to `Options::label`, in a
        /// `span { "data-slot": "label" }`, which is what ellipsises a long one.
        #[props(default)]
        option: Option<Callback<SelectOptionArgs<T>, Element>>,
        /// Draws one selected value beside the trigger. Defaults to the label
        /// in a `Chip` with an x. A caller who overrides it draws the whole
        /// chip, remove control included - `args.remove` is the wiring, and the
        /// keyboard stays the control's either way.
        #[props(default)]
        selection: Option<Callback<SelectionArgs<T>, Element>>,
        /// Shown while `value` is empty.
        #[props(default)]
        placeholder: Option<String>,
        /// Emits one hidden input of that name per selected option, carrying
        /// its `Options::value()`. The trigger is a `div`, so it cannot carry
        /// the name itself.
        /// A path - `Order::FIELDS.toppings()` - also binds the selection to
        /// the surrounding `Form`'s value when there is no `onchange`.
        #[props(default, into)]
        name: crate::components::FieldName<Vec<T>>,
        /// Rules over the selection, shown once the select loses focus or its
        /// form is submitted.
        #[props(default, into)]
        validate: crate::components::Validators<Vec<T>>,
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

    let bound = use_bound(&props.name, props.onchange.is_some());
    let held = bound.value().unwrap_or_else(|| props.value.clone());

    if props.onchange.is_none() && !bound.is_bound() {
        warn("MultiSelect: without `onchange` the selection can never change.");
    }

    let list = props.options.or_static();
    let values = list.values();
    // The same rule as `Select`: a request still in flight lists nothing, and
    // says nothing about being empty.
    let loading = props.options.is_pending();
    if values.is_empty() && !loading {
        warn("MultiSelect: no options - a `T` without static `options()` needs `options`.");
    }

    let selected: Vec<bool> = values.iter().map(|option| held.contains(option)).collect();
    let row_disabled = list.disabled();
    let rows = draw_rows(&values, &selected, &row_disabled, props.option.as_ref());
    // The chips, redrawn from the cursor the core owns. Each wrapper carries the
    // id `aria-activedescendant` points at; what is inside it is the skin's, or
    // the caller's.
    let onchange = bound.emit(props.onchange);
    let size = props.size.copied_or(theme.multi_select.size);
    let disabled = bound.disabled(props.disabled);
    let readonly = props.readonly.unwrap_or(false);
    let picked = held.clone();
    let draw_selection = props.selection;
    let chip_change = onchange.clone();
    let selection = (!picked.is_empty()).then(|| {
        Callback::new(move |args: SelectionRenderArgs| {
            let chips = picked.iter().cloned().enumerate().map(|(index, value)| {
                let removing = picked.clone();
                let onchange = chip_change.clone();
                let remove = Callback::new(move |_: ()| drop_at(&removing, index, &onchange));
                match &draw_selection {
                    Some(selection) => selection.call(SelectionArgs { value, remove }),
                    None => removable_chip(value.label(), remove, size, disabled || readonly),
                }
            });
            rsx! {
                for (index, chip) in chips.enumerate() {
                    span {
                        key: "{index}",
                        "data-slot": "chip",
                        id: "{args.id_prefix}-{index}",
                        "data-cursor": (args.cursor == Some(index)).then_some("true"),
                        {chip}
                    }
                }
            }
        })
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

    let current = held.clone();
    // Built before the pick closure takes `current`.
    let posted = current.iter().map(Options::value).collect::<Vec<_>>();
    let rules = props.validate.check(&current);
    let removable = held.clone();
    let value_labels = held.iter().map(Options::label).collect::<Vec<_>>();
    let (pick_change, remove_change, clear_change) = (onchange.clone(), onchange.clone(), onchange);
    rsx! {
        SelectCore {
            rows,
            selected,
            groups: list.group_labels(),
            row_disabled,
            row_labels: values.iter().map(Options::label).collect::<Vec<_>>(),
            loading: loading.then(|| theme.combobox.labels.loading.to_string()),
            multiple: true,
            onpick: move |index: usize| {
                let (Some(onchange), Some(value)) = (&pick_change, values.get(index)) else {
                    return;
                };
                let mut next = current.clone();
                match next.iter().position(|picked| picked == value) {
                    Some(at) => {
                        next.remove(at);
                    }
                    None => next.push(value.clone()),
                }
                onchange(next);
            },
            selection,
            value_labels,
            chip_count: removable.len(),
            onremove: move |index: usize| drop_at(&removable, index, &remove_change),
            placeholder: props.placeholder,
            name: bound.name().map(str::to_string),
            form_values: posted,
            rules,
            clearable: props.clearable.unwrap_or(false),
            searchable,
            search_placeholder: props.search_placeholder,
            matches,
            onclear: move |_| {
                if let Some(onchange) = &clear_change {
                    onchange(Vec::new());
                }
            },
            label: props.label,
            description: props.description,
            helper: props.helper,
            status: props.status,
            size,
            radius: props.radius.copied_or(theme.multi_select.radius),
            disabled: Some(disabled),
            readonly: props.readonly,
            required: props.required,
            class: props.class,
            sx: props.sx,
            states: props.states,
            attributes: props.attributes,
        }
    }
}

/// Drops one value from the selection - the same edit as picking its row again.
fn drop_at<T: Options>(values: &[T], index: usize, onchange: &Option<impl Fn(Vec<T>)>) {
    let Some(onchange) = onchange else {
        return;
    };
    if index >= values.len() {
        return;
    }
    let mut next = values.to_vec();
    next.remove(index);
    onchange(next);
}
