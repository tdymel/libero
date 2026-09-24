use dioxus::prelude::*;

use crate::components::form::{row_label, use_bound};

use crate::{
    components::{
        common::{ComboboxState, Input, OptionSource, Options, use_combobox},
        form::field_props,
    },
    hooks::{use_localization, use_theme},
    utils::warn,
};

use super::core::{Picked, SelectCore, SelectPart, SelectionDraw, SelectionRenderArgs, use_picked};

/// One row's content, handed to `option`. The row itself (highlight, `aria-selected`, click) is the component's.
#[derive(Clone, PartialEq)]
pub struct SelectOptionArgs<T> {
    pub value: T,
    pub index: usize,
    /// Part of the selection, for a checkmark.
    pub selected: bool,
    /// The list refuses this row. Greying and `aria-disabled` are drawn anyway.
    pub disabled: bool,
}

/// One selected value, handed to `MultiSelect`'s and `FileField`'s `selection` callback.
/// The chip is the caller's, `remove` included; the control keeps the keyboard.
#[derive(Clone, PartialEq)]
pub struct SelectionArgs<T> {
    pub value: T,
    /// Drops this value, the same edit as picking its row again.
    pub remove: Callback<()>,
}

/// One option under test, handed to `filter` while searching. The same pair as `AutocompleteFilterArgs`.
#[derive(Clone, PartialEq)]
pub struct SelectFilterArgs<T> {
    pub value: T,
    /// What is typed in the search box.
    pub query: String,
}

field_props! {
    parts(SelectPart);
    pub struct SelectProps<T: Options> {
        /// Strictly controlled: pair it with `onchange`. `None` shows `placeholder`.
        #[props(default)]
        value: Option<T>,
        /// The option to select next, or `None` when `clearable`'s x is clicked.
        #[props(default)]
        onchange: Option<EventHandler<Option<T>>>,
        /// Defaults to `Options::options()`, empty for runtime types like `String`. A `Vec<T>`,
        /// an [`OptionList`](crate::components::OptionList) (groups, `disabled`), or a [`Resource`].
        #[props(default, into)]
        options: OptionSource<T>,
        /// Draws one row's content. Defaults to `Options::label` in an ellipsising label slot.
        #[props(default)]
        option: Option<Callback<SelectOptionArgs<T>, Element>>,
        /// Draws the selection in the trigger. Defaults to the label.
        #[props(default)]
        selection: Option<Callback<T, Element>>,
        /// Shown while `value` is `None`.
        #[props(default)]
        placeholder: Option<String>,
        /// A hidden input posting `Options::value()`. A field path also binds to the `Form` without `onchange`.
        #[props(default, into)]
        name: crate::components::form::FieldName<Option<T>>,
        /// Rules over the selection, shown after blur or submit.
        #[props(default, into)]
        validate: crate::components::form::Validators<Option<T>>,
        /// Shows an x that clears the selection.
        #[props(default)]
        clearable: Option<bool>,
        /// Puts a search box at the top of the list.
        #[props(default)]
        searchable: Option<bool>,
        /// Narrows the options while searching. Defaults to a case-insensitive `contains` over the label.
        #[props(default)]
        filter: Option<Callback<SelectFilterArgs<T>, bool>>,
        /// What the search box says while empty.
        #[props(default)]
        search_placeholder: Option<String>,
    }
}

/// A controlled single-choice listbox field with custom-drawn rows.
///
/// Unlike `NativeSelect` it has no OS picker on phones and needs wasm.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Select;
/// # fn app() -> Element {
/// let mut plan = use_signal(|| None::<String>);
/// rsx! {
///     Select {
///         label: "Plan",
///         options: vec!["Free".to_string(), "Pro".to_string()],
///         value: plan(),
///         onchange: move |next| plan.set(next),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/select>
#[component]
pub fn Select<T: Options>(props: SelectProps<T>) -> Element {
    let theme = use_theme();
    let common = use_localization().common;

    let bound = use_bound(&props.name, props.onchange.is_some());
    let current = bound.value().unwrap_or_else(|| props.value.clone());

    if props.onchange.is_none() && !bound.is_bound() {
        warn("Select: without `onchange` the selection can never change.");
    }

    let list = props.options.or_static();
    let values = list.values();
    // A pending list shows nothing and doesn't report empty.
    let loading = props.options.is_pending();
    if values.is_empty() && !loading {
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
    let row_disabled = list.disabled();
    let state = use_combobox();
    let rows = draw_open_rows(
        state,
        &values,
        &selected,
        &row_disabled,
        props.option.as_ref(),
    );
    // No chips, so the core's cursor is always `None` here.
    let draw_selection = props.selection;
    let shown = selected_index.map(|index| values[index].clone());
    let draw = use_callback(
        move |_: SelectionRenderArgs| match (&draw_selection, &shown) {
            (Some(selection), Some(value)) => selection.call(value.clone()),
            (None, Some(value)) => rsx! { "{value.label()}" },
            (_, None) => rsx! {},
        },
    );
    let selection = selected_index.map(|_| SelectionDraw::new(draw, props.selection.is_some()));
    let rendered = Picked {
        selected,
        form_values: vec![current.as_ref().map(Options::value).unwrap_or_default()],
    };
    let picked = use_picked(rendered.clone());
    // The props' selection as of the last render, for `onpick`'s refusal check.
    let mut latest = use_hook(|| CopyValue::new(rendered.clone()));
    latest.set(rendered);

    let searchable = props.searchable.unwrap_or(false);
    let matches = use_search_mask(searchable, values.clone(), props.filter);
    let row_labels = values.iter().map(Options::label).collect::<Vec<_>>();

    // Stable, so a closed select's `SelectCore` compares equal and skips.
    let onchange = bound.emit(props.onchange);
    let clear = onchange.clone();
    let onpick = use_callback(move |index: usize| {
        if let (Some(onchange), Some(value)) = (&onchange, values.get(index)) {
            // Written here too: `use_picked`'s write lands a pass later (todo 842).
            let mut picked = picked;
            picked.set(Picked {
                selected: (0..values.len()).map(|row| row == index).collect(),
                form_values: vec![value.value()],
            });
            onchange(Some(value.clone()));
            // After `onchange`'s renders: a caller that kept the old value wins back.
            spawn(async move {
                if *picked.peek() != *latest.peek() {
                    picked.set(latest.cloned());
                }
            });
        }
    });
    let onclear = use_callback(move |_: ()| {
        if let Some(clear) = &clear {
            clear(None);
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
            onpick,
            selection,
            placeholder: props.placeholder,
            name: bound.name().map(str::to_string),
            rules: props.validate.check(&current),
            clearable: props.clearable.unwrap_or(false),
            searchable,
            search_placeholder: props.search_placeholder,
            matches,
            onclear,
            label: props.label,
            description: props.description,
            helper: props.helper,
            status: props.status,
            size: props.size.copied_or(theme.select.size),
            radius: props.radius.copied_or(theme.select.radius),
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

/// Which rows a query keeps; `None` unless `searchable`. A stable callback running the newest captures.
pub(super) fn use_search_mask<T: Options>(
    searchable: bool,
    values: Vec<T>,
    filter: Option<Callback<SelectFilterArgs<T>, bool>>,
) -> Option<Callback<String, Vec<bool>>> {
    let mask = use_callback(move |query: String| {
        let needle = query.to_lowercase();
        values
            .iter()
            .map(|value| match &filter {
                Some(filter) => filter.call(SelectFilterArgs {
                    value: value.clone(),
                    query: query.clone(),
                }),
                None => value.label().to_lowercase().contains(&needle),
            })
            // Annotated: `SpawnIfAsync` leaves a bare `collect` ambiguous.
            .collect::<Vec<bool>>()
    });
    searchable.then_some(mask)
}

/// [`draw_rows`] while the list is open, nothing while it is closed.
pub(super) fn draw_open_rows<T: Options>(
    state: ComboboxState,
    values: &[T],
    selected: &[bool],
    disabled: &[bool],
    option: Option<&Callback<SelectOptionArgs<T>, Element>>,
) -> Vec<Element> {
    match state.is_open() {
        true => draw_rows(values, selected, disabled, option),
        false => Vec::new(),
    }
}

/// Each row's content: the caller's `option`, or the label.
pub(super) fn draw_rows<T: Options>(
    values: &[T],
    selected: &[bool],
    disabled: &[bool],
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
                disabled: disabled.get(index).copied().unwrap_or(false),
            }),
            None => row_label(value.label()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Default rows put their text in the label slot, the one that can ellipsise.
    #[test]
    fn a_default_row_puts_its_text_in_the_label_slot() {
        let values = vec!["Apple".to_string(), "Banana".to_string()];
        let rows = draw_rows(&values, &[false, true], &[false, false], None);
        let html: Vec<String> = rows.into_iter().map(dioxus_ssr::render_element).collect();
        assert_eq!(
            html,
            [
                r#"<span data-slot="label">Apple</span>"#,
                r#"<span data-slot="label">Banana</span>"#
            ]
        );
    }
}
