use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, OptionSource, Options, Parts, ScaleOrCss, use_combobox},
        form::{
            CaretKeys, ComboboxCore, ComboboxOption, DropdownPart, FIELD_CONTROL_SX, clear_button,
            field_props, row_label, use_bound, use_field, use_field_frame, use_row_cache,
        },
        layout::use_box,
    },
    hooks::{PopoverWidth, use_element, use_localization, use_theme},
    utils::warn,
};

/// One suggestion, handed to `Autocomplete`'s `option` callback to draw the
/// row's content. No `selected`: a suggestion is not a selection.
#[derive(Clone, PartialEq)]
pub struct AutocompleteOptionArgs<T> {
    pub value: T,
    /// The row's position in the *narrowed* list.
    pub index: usize,
}

/// One suggestion under test, handed to `Autocomplete`'s `filter` callback.
#[derive(Clone, PartialEq)]
pub struct AutocompleteFilterArgs<T> {
    pub value: T,
    /// The text currently in the field.
    pub query: String,
}

field_props! {
    extends(input);
    pub struct AutocompleteProps<T: Options> {
        /// The text. Strictly controlled - pair it with `oninput`.
        #[props(default, into)]
        value: String,
        /// Fires per keystroke, and again with the label when a suggestion is
        /// picked or the field is cleared.
        #[props(default)]
        oninput: Option<EventHandler<String>>,
        /// Rules over the text, shown on blur or submit.
        #[props(default, into)]
        validate: crate::components::form::Validators<String>,
        /// The suggestions to offer. A `Vec<T>`, or a [`Resource`] or `Option` (`None` while
        /// fetching): a pending list says `loading_label` rather than `nothing_found`.
        #[props(default, into)]
        options: OptionSource<T>,
        /// Draws one row's content. Defaults to `Options::label`.
        #[props(default)]
        option: Option<Callback<AutocompleteOptionArgs<T>, Element>>,
        /// A suggestion was accepted, with the whole `T`. Fires after `oninput`.
        #[props(default)]
        onpick: Option<EventHandler<T>>,
        /// Narrows `options`. Defaults to a case-insensitive `contains` of the trimmed text on the label.
        #[props(default)]
        filter: Option<Callback<AutocompleteFilterArgs<T>, bool>>,
        /// `options` arrives already narrowed; `filter` is skipped.
        #[props(default)]
        prefiltered: Option<bool>,
        /// What the field posts as. A path also binds it to the surrounding `Form`.
        #[props(default, into)]
        name: crate::components::form::FieldName<String>,
        #[props(default, into)]
        placeholder: Option<String>,
        /// Shows an x that empties the field while it holds text.
        #[props(default)]
        clearable: Option<bool>,
        /// Shown in place of the list when typed text matches nothing. Announced as
        /// `combobox.nothing_found` either way.
        #[props(default)]
        empty: Option<Element>,
        /// Said while fetching. Defaults to [`CommonLabels::loading`](crate::localization::CommonLabels::loading).
        #[props(default, into)]
        loading_label: Option<String>,
        /// Inside the frame, before the control.
        #[props(default, into)]
        leading: Option<Element>,
        /// Inside the frame, after the control.
        #[props(default, into)]
        trailing: Option<Element>,
        /// `leading` is text that describes the value (`aria-describedby`).
        #[props(default)]
        describe_leading: bool,
        /// `trailing` is text that describes the value, e.g. a unit.
        #[props(default)]
        describe_trailing: bool,
        /// Styles the portaled dropdown and its inner parts.
        #[props(default, into)]
        dropdown_parts: Input<Parts<DropdownPart>>,
    }
}

/// A text field that offers completions; the value stays a `String`.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Autocomplete;
/// # fn app() -> Element {
/// let mut city = use_signal(String::new);
/// rsx! {
///     Autocomplete {
///         label: "City",
///         value: city(),
///         oninput: move |text| city.set(text),
///         options: vec!["Berlin".to_string(), "Paris".to_string()],
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/autocomplete>
#[component]
pub fn Autocomplete<T: Options>(props: AutocompleteProps<T>) -> Element {
    let theme = use_theme();
    let localization = use_localization();
    let nothing_found = localization.combobox.nothing_found;
    let size = props.size.copied_or(theme.autocomplete.size);
    let radius = ScaleOrCss::new(props.radius.as_ref(), theme.autocomplete.radius);
    let required = props.required.unwrap_or(false);
    let prefiltered = props.prefiltered.unwrap_or(false);

    let bound = use_bound(&props.name, props.oninput.is_some());
    let disabled = bound.disabled(props.disabled);
    // Native `readonly` covers typing; a read-only field also never opens the list.
    let readonly = props.readonly.unwrap_or(false);
    let text = bound.value().unwrap_or_else(|| props.value.clone());

    if props.oninput.is_none() && !bound.is_bound() {
        warn("Autocomplete: without `oninput` the text can never change.");
    }
    if prefiltered && props.filter.is_some() {
        warn("Autocomplete: `prefiltered` skips filtering, so `filter` never runs.");
    }

    let state = use_combobox();
    let input_element = use_element();
    let row_cache = use_row_cache();

    // Trimmed: a phone keyboard's trailing space must not hide "Paris" (todo 2415).
    let query = text.trim().to_lowercase();
    // A pending list draws the loader, never its stale rows or "nothing found".
    let loading = props.options.is_pending();
    let options = props.options.list().values();
    // With their place in `options`, the rows' stable key.
    let matches: Vec<(usize, T)> = options
        .iter()
        .enumerate()
        .filter(|(_, value)| match (prefiltered, &props.filter) {
            (true, _) => true,
            (false, Some(filter)) => filter.call(AutocompleteFilterArgs {
                value: (*value).clone(),
                query: text.clone(),
            }),
            (false, None) => value.label().to_lowercase().contains(&query),
        })
        .map(|(key, value)| (key, value.clone()))
        .collect();

    // The label also names the listbox, which `for` cannot reach.
    let field = use_field()
        .label_with_id()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(props.validate.check(&text))
        .bound(&bound)
        .required(required)
        .empty(text.is_empty())
        .readonly(readonly)
        .disabled(disabled)
        .size(size)
        .radius(radius.clone())
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&props.states)
        .attributes(&props.attributes)
        .text_slots(
            props.describe_leading && props.leading.is_some(),
            props.describe_trailing && props.trailing.is_some(),
        )
        .prepare();

    let oninput = bound.emit(props.oninput);
    let onpick = props.onpick;

    let pick_input = oninput.clone();
    let pick = use_callback(move |value: T| {
        if let Some(oninput) = &pick_input {
            oninput(value.label());
        }
        if let Some(onpick) = &onpick {
            onpick.call(value);
        }
        state.close();
    });
    // A caller's `option` content is drawn eagerly and handed down as a value, so a narrowed
    // list never leaves a stale row on screen.
    let option = props.option;
    let row_keys: Vec<usize> = matches.iter().map(|(key, _)| *key).collect();
    let rows = row_cache.rows(
        matches
            .into_iter()
            .enumerate()
            .map(|(index, (key, value))| {
                let content = option.as_ref().map(|option| {
                    option.call(AutocompleteOptionArgs {
                        value: value.clone(),
                        index,
                    })
                });
                (
                    key,
                    AutocompleteRowInputs {
                        value,
                        pick,
                        content,
                    },
                )
            }),
        autocomplete_row::<T>,
    );
    let has_rows = !rows.is_empty();
    // Typed text against a list that has options to match, a fetched one, or a caller's `empty`.
    let nothing_found = (!text.is_empty()
        && (prefiltered || !options.is_empty() || props.empty.is_some()))
    .then(|| nothing_found.to_string());
    // Drawn only while said, so never under an emptied field (todo 2417).
    let empty = props.empty.filter(|_| nothing_found.is_some());
    let loading = loading.then(|| {
        props
            .loading_label
            .unwrap_or_else(|| localization.common.loading.to_string())
    });

    let clear_input = oninput.clone();
    let clear = clear_button(
        props.clearable.unwrap_or(false) && !text.is_empty() && !disabled && !readonly,
        size,
        input_element,
        Some(&field),
        move |_| {
            if let Some(oninput) = &clear_input {
                oninput(String::new());
            }
            state.set_active(None);
        },
    );
    // The x sits last. The id goes on the caller's part alone, so the x's name
    // does not describe the input.
    let [leading_id, trailing_id] = field.slot_ids();
    let trailing = (props.trailing.is_some() || clear.is_some()).then(|| {
        let caller = match (props.trailing.clone(), trailing_id) {
            (Some(caller), Some(id)) => rsx! { span { id, {caller} } },
            (caller, _) => caller.unwrap_or_else(|| rsx! {}),
        };
        rsx! {
            {caller}
            {clear.clone()}
        }
    });

    let frame = use_field_frame()
        .leading(&props.leading)
        .trailing(&trailing)
        .states(field.states())
        .ids([leading_id, None])
        .placeholder(props.placeholder.as_deref())
        .prepare();

    // The frame draws the ring, so the control must not draw a second one.
    let control = use_box()
        .framework_sx(&FIELD_CONTROL_SX)
        .focus_ring(false)
        .prepare();

    // The count `ComboboxCore` tells the state: none while fetching.
    let mut attributes = state.a11y_attributes_for(match loading {
        Some(_) => &[],
        None => &row_keys,
    });
    attributes.extend(props.attributes);
    let input = field
        .aria(control)
        .attr_default("type", "text")
        // `list`, not `both`: the field never completes the text inline, it
        // only offers rows underneath.
        .attr("aria-autocomplete", "list")
        .attr("name", bound.name().map(str::to_string))
        .attr("value", text)
        .attr("data-controlled", true)
        .attr("placeholder", props.placeholder)
        .attr("disabled", disabled)
        .attr("readonly", readonly)
        .attr("required", required)
        // The browser's saved entries would cover the list; a caller's token (WCAG 1.3.5) takes that trade.
        .attr_default("autocomplete", "off")
        .event("oninput", move |event: FormEvent| {
            if let Some(oninput) = &oninput {
                oninput(event.value());
            }
            // Typing disarms the highlight: the next Enter belongs to the typed
            // text, not to a row that moved under it.
            state.set_active(None);
            state.open();
        })
        .event("onblur", move |_: FocusEvent| state.close())
        .element(&input_element)
        .render(HtmlTag::Input, attributes, ());

    let listbox = rsx! {
        ComboboxCore {
            rows,
            row_keys,
            active: state.active(),
            onactive: move |row| state.set_active(Some(row)),
            // Nothing to show is not open: `aria-expanded` must not claim a
            // popup that draws nothing.
            opened: state.is_open()
                && !disabled
                && !readonly
                && (has_rows || loading.is_some() || nothing_found.is_some()),
            onopened: move |opened| state.set_open(opened),
            state,
            caret_keys: CaretKeys::Unhighlighted,
            empty,
            nothing_found,
            loading,
            size,
            radius: radius.value(),
            disabled: disabled || readonly,
            width: PopoverWidth::Match,
            labelled_by: field.label_id(),
            parts: props.dropdown_parts,
            {frame.render(input)}
        }
    };

    field.render(listbox)
}

/// Everything a row draws from. A caller's `content` never compares equal, so such a row redraws per key.
#[derive(Clone, PartialEq)]
struct AutocompleteRowInputs<T: 'static> {
    value: T,
    pick: Callback<T>,
    content: Option<Element>,
}

fn autocomplete_row<T: Options>(_key: usize, inputs: &AutocompleteRowInputs<T>) -> Element {
    let AutocompleteRowInputs {
        value,
        pick,
        content,
    } = inputs.clone();
    let label = content.unwrap_or_else(|| row_label(value.label()));
    rsx! {
        ComboboxOption { onpick: move |_| pick.call(value.clone()), {label} }
    }
}
