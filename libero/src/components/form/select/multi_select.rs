use dioxus::prelude::*;

use crate::components::form::use_bound;

use crate::{
    components::{
        common::{Input, OptionSource, Options, Part, Parts, use_combobox},
        form::{DropdownPart, field_props, removable_chip},
    },
    hooks::{use_localization, use_theme},
    utils::warn,
};

use super::{
    core::{Picked, SelectCore, SelectPart, SelectionDraw, SelectionRenderArgs, use_picked},
    select::{SelectFilterArgs, SelectOptionArgs, SelectionArgs, draw_open_rows, use_search_mask},
};

field_props! {
    parts(SelectPart);
    pub struct MultiSelectProps<T: Options> {
        /// Strictly controlled: pair it with `onchange`. Empty shows `placeholder`.
        #[props(default)]
        value: Vec<T>,
        /// The whole selection to hold next.
        #[props(default)]
        onchange: Option<EventHandler<Vec<T>>>,
        /// Defaults to `Options::options()`, empty for runtime types like `String`. A `Vec<T>`,
        /// an [`OptionList`](crate::components::OptionList) (groups, `disabled`), or a [`Resource`].
        #[props(default, into)]
        options: OptionSource<T>,
        /// Draws one row's content. Defaults to `Options::label` in an ellipsising label slot.
        #[props(default)]
        option: Option<Callback<SelectOptionArgs<T>, Element>>,
        /// Draws one selected value. Defaults to a `Chip` with an x; an override draws the whole chip.
        #[props(default)]
        selection: Option<Callback<SelectionArgs<T>, Element>>,
        /// Shown while `value` is empty.
        #[props(default)]
        placeholder: Option<String>,
        /// One hidden input per selected option. A field path also binds to the `Form` without `onchange`.
        #[props(default, into)]
        name: crate::components::form::FieldName<Vec<T>>,
        /// Rules over the selection, shown after blur or submit.
        #[props(default, into)]
        validate: crate::components::form::Validators<Vec<T>>,
        /// Shows an x that empties the selection.
        #[props(default)]
        clearable: Option<bool>,
        /// Puts a search box at the top of the list. The query survives a pick.
        #[props(default)]
        searchable: Option<bool>,
        /// Narrows the options while searching. Defaults to a case-insensitive `contains` over the label.
        #[props(default)]
        filter: Option<Callback<SelectFilterArgs<T>, bool>>,
        /// What the search box says while empty.
        #[props(default)]
        search_placeholder: Option<String>,
        /// Styles the portaled dropdown and its inner parts.
        #[props(default, into)]
        dropdown_parts: Input<Parts<DropdownPart>>,
    }
}

/// A listbox field holding any number of options, shown as chips in pick order.
///
/// A pick toggles its row and keeps the list open.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::MultiSelect;
/// # fn app() -> Element {
/// let mut toppings = use_signal(Vec::<String>::new);
/// rsx! {
///     MultiSelect {
///         label: "Toppings",
///         options: vec!["Cheese".to_string(), "Olives".to_string()],
///         value: toppings(),
///         onchange: move |next| toppings.set(next),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/multi-select>
#[component]
pub fn MultiSelect<T: Options>(props: MultiSelectProps<T>) -> Element {
    let theme = use_theme();
    let common = use_localization().common;

    let bound = use_bound(&props.name, props.onchange.is_some());
    let held = bound.value().unwrap_or_else(|| props.value.clone());

    if props.onchange.is_none() && !bound.is_bound() {
        warn("MultiSelect: without `onchange` the selection can never change.");
    }

    let list = props.options.or_static();
    let values = list.values();
    // As `Select`: a pending list shows nothing and doesn't report empty.
    let loading = props.options.is_pending();
    if values.is_empty() && !loading {
        warn("MultiSelect: no options - a `T` without static `options()` needs `options`.");
    }

    let selected: Vec<bool> = values.iter().map(|option| held.contains(option)).collect();
    let row_disabled = list.disabled();
    let state = use_combobox();
    let rows = draw_open_rows(
        state,
        &values,
        &selected,
        &row_disabled,
        props.option.as_ref(),
    );
    let onchange = bound.emit(props.onchange);
    let size = props.size.copied_or(theme.multi_select.size);
    let disabled = bound.disabled(props.disabled);
    let readonly = props.readonly.unwrap_or(false);

    // Stable callbacks, so a closed select's `SelectCore` compares equal and skips.
    let removable = held.clone();
    let remove_change = onchange.clone();
    let onremove = use_callback(move |index: usize| drop_at(&removable, index, &remove_change));
    // Each chip wrapper carries the id `aria-activedescendant` points at.
    let picked = held.clone();
    let draw_selection = props.selection;
    let draw = use_callback(move |args: SelectionRenderArgs| {
        let chips = picked.iter().cloned().enumerate().map(|(index, value)| {
            // Guarded as Backspace is, since a caller's `selection` gets it too.
            let remove = Callback::new(move |_: ()| {
                if !disabled && !readonly {
                    onremove.call(index);
                }
            });
            match &draw_selection {
                Some(selection) => selection.call(SelectionArgs {
                    value,
                    remove,
                    disabled,
                    readonly,
                }),
                None => removable_chip(value.label(), remove, size, disabled || readonly),
            }
        });
        rsx! {
            for (index, chip) in chips.enumerate() {
                span {
                    key: "{index}",
                    "data-slot": SelectPart::Chip.slot(),
                    id: "{args.id_prefix}-{index}",
                    "data-cursor": (args.cursor == Some(index)).then_some("true"),
                    {chip}
                }
            }
        }
    });
    let selection = (!held.is_empty()).then(|| SelectionDraw::new(draw, props.selection.is_some()));

    let searchable = props.searchable.unwrap_or(false);
    let matches = use_search_mask(searchable, values.clone(), props.filter);
    let row_labels = values.iter().map(Options::label).collect::<Vec<_>>();

    let current = held.clone();
    // Built before the pick closure takes `current`.
    let posted = current.iter().map(Options::value).collect::<Vec<_>>();
    let rules = props.validate.check(&current);
    let value_labels = held.iter().map(Options::label).collect::<Vec<_>>();
    let rendered = Picked {
        selected,
        form_values: posted,
    };
    let picked = use_picked(rendered.clone());
    // The props' selection as of the last render, for `onpick`'s refusal check.
    let mut latest = use_hook(|| CopyValue::new(rendered.clone()));
    latest.set(rendered);
    let (pick_change, clear_change) = (onchange.clone(), onchange);
    let onpick = use_callback(move |index: usize| {
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
        // As `Select`: `use_picked`'s write lands a pass later (todo 876).
        let mut picked = picked;
        picked.set(Picked {
            selected: values.iter().map(|row| next.contains(row)).collect(),
            form_values: next.iter().map(Options::value).collect(),
        });
        onchange(next);
        // A caller that kept the old selection left `latest` behind: the props win.
        spawn(async move {
            if *picked.peek() != *latest.peek() {
                picked.set(latest.cloned());
            }
        });
    });
    let onclear = use_callback(move |_: ()| {
        if let Some(onchange) = &clear_change {
            onchange(Vec::new());
        }
    });
    rsx! {
        SelectCore {
            rows,
            picked,
            state,
            groups: list.group_labels(),
            row_disabled,
            row_labels,
            loading: loading.then(|| common.loading.to_string()),
            multiple: true,
            onpick,
            selection,
            value_labels,
            chip_count: held.len(),
            onremove,
            placeholder: props.placeholder,
            name: bound.name().map(str::to_string),
            rules,
            clearable: props.clearable.unwrap_or(false),
            searchable,
            search_placeholder: props.search_placeholder,
            matches,
            onclear,
            label: props.label,
            description: props.description,
            helper: props.helper,
            status: props.status,
            size,
            radius: Input::Value(
                props
                    .radius
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(|| theme.multi_select.radius.into()),
            ),
            disabled: Some(disabled),
            readonly: props.readonly,
            required: props.required,
            class: props.class,
            sx: props.sx,
            parts: props.parts,
            dropdown_parts: props.dropdown_parts,
            states: props.states,
            attributes: props.attributes,
        }
    }
}

/// Drops one value, the same edit as picking its row again.
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
