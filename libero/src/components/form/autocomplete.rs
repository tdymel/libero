use dioxus::prelude::*;

use crate::{
    components::{
        ComboboxCore, ComboboxOption, HtmlTag, Input, Options,
        common::field_props,
        form::{FIELD_CONTROL_SX, clear_button, use_bound, use_field, use_field_frame},
        layout::use_box,
        use_combobox,
    },
    hooks::{PopoverWidth, use_element, use_theme},
    utils::warn,
};

/// One suggestion, handed to `Autocomplete`'s `option` callback. It draws the
/// row's *content*: the row itself - its highlight, its click - is the
/// component's.
///
/// No `selected`: a suggestion is not a selection, which is the call
/// `Combobox` already made for `aria-selected`.
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
        /// Rules over the text, shown once the field loses focus or its form
        /// is submitted.
        #[props(default, into)]
        validate: crate::components::Validators<String>,
        /// The suggestions to offer. `T` infers from it, so no call site ever
        /// annotates one.
        #[props(default)]
        options: Vec<T>,
        /// Draws one row's content. Defaults to `Options::label`.
        #[props(default)]
        option: Option<Callback<AutocompleteOptionArgs<T>, Element>>,
        /// A suggestion was accepted, with the whole `T` behind the text - the
        /// record's id, not just its label. Fires after `oninput`.
        #[props(default)]
        onpick: Option<EventHandler<T>>,
        /// Narrows `options`. Defaults to a case-insensitive `contains` over
        /// `Options::label`.
        #[props(default)]
        filter: Option<Callback<AutocompleteFilterArgs<T>, bool>>,
        /// `options` arrives already narrowed - a list fetched per keystroke.
        /// Skips filtering entirely, so `filter` is dead alongside it.
        #[props(default)]
        prefiltered: Option<bool>,
        /// What the field posts as. A path - `Signup::FIELDS.city()` - also
        /// binds it to the surrounding `Form`'s value when the field has no
        /// `oninput`.
        #[props(default, into)]
        name: crate::components::FieldName<String>,
        #[props(default, into)]
        placeholder: Option<String>,
        /// Shows an x that empties the field while it holds text.
        #[props(default)]
        clearable: Option<bool>,
        /// Shown in place of the list when nothing matches. Without it a list
        /// with no rows draws nothing at all.
        #[props(default)]
        empty: Option<Element>,
        /// Inside the frame, before the control - a search icon.
        #[props(default, into)]
        leading: Option<Element>,
        /// Inside the frame, after the control, before the clear x.
        #[props(default, into)]
        trailing: Option<Element>,
    }
}

/// A text field that offers completions.
///
/// The value is a `String` at all times - picking a suggestion inserts its
/// label, it does not make the field hold a `T`. `T` is only what the rows are
/// drawn from, and `onpick` hands the whole one back for the caller that needs
/// the record behind the text. To *choose* out of a fixed set instead, reach
/// for `Select`.
///
/// Typing opens the list; ArrowDown opens it too. Nothing is highlighted until
/// the user arrows onto a row, so Enter on text that matches nothing is not
/// swallowed and a form still submits.
#[component]
pub fn Autocomplete<T: Options>(props: AutocompleteProps<T>) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.autocomplete.size);
    let radius = props.radius.copied_or(theme.autocomplete.radius);
    let required = props.required.unwrap_or(false);
    let prefiltered = props.prefiltered.unwrap_or(false);

    let bound = use_bound(&props.name, props.oninput.is_some());
    let disabled = bound.disabled(props.disabled);
    // The text is the value, so the native `readonly` covers typing; the
    // suggestion list is the other way to change it, and a read-only field
    // does not open it.
    let readonly = props.readonly.unwrap_or(false);
    let text = bound.value().unwrap_or_else(|| props.value.clone());

    if props.oninput.is_none() && !bound.is_bound() {
        warn("Autocomplete: without `oninput` the text can never change.");
    }
    if prefiltered && props.filter.is_some() {
        warn("Autocomplete: `prefiltered` skips filtering, so `filter` never runs.");
    }

    let state = use_combobox();
    // What the x hands the focus to once it has cleared the field.
    let input_element = use_element();

    let query = text.to_lowercase();
    let matches: Vec<T> = props
        .options
        .iter()
        .filter(|value| match (prefiltered, &props.filter) {
            (true, _) => true,
            (false, Some(filter)) => filter.call(AutocompleteFilterArgs {
                value: (*value).clone(),
                query: text.clone(),
            }),
            (false, None) => value.label().to_lowercase().contains(&query),
        })
        .cloned()
        .collect();

    let field = use_field()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(props.validate.check(&text))
        .bound(&bound)
        .required(required)
        .disabled(disabled)
        .size(size)
        .radius(radius)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .attributes(&props.attributes)
        .prepare();

    let oninput = bound.emit(props.oninput);
    let onpick = props.onpick;

    // Drawn eagerly and handed down as values: a `Callback` would let the rows
    // memoize and a narrowed list would leave stale ones on screen.
    let option = props.option;
    let rows: Vec<Element> = matches
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, value)| {
            let oninput = oninput.clone();
            let content = match &option {
                Some(option) => option.call(AutocompleteOptionArgs {
                    value: value.clone(),
                    index,
                }),
                None => rsx! { "{value.label()}" },
            };
            rsx! {
                ComboboxOption {
                    onpick: move |_| {
                        if let Some(oninput) = &oninput {
                            oninput(value.label());
                        }
                        if let Some(onpick) = &onpick {
                            onpick.call(value.clone());
                        }
                        state.close();
                    },
                    {content}
                }
            }
        })
        .collect();

    let clear_input = oninput.clone();
    let clear = clear_button(
        props.clearable.unwrap_or(false) && !text.is_empty() && !disabled,
        size,
        input_element,
        move |_| {
            if let Some(oninput) = &clear_input {
                oninput(String::new());
            }
            state.set_active(None);
        },
    );
    // The caller's own trailing content keeps its place; the x sits at the end,
    // nearest the frame's edge.
    let trailing = (props.trailing.is_some() || clear.is_some()).then(|| {
        let caller = props.trailing.clone();
        rsx! {
            {caller}
            {clear.clone()}
        }
    });

    let frame = use_field_frame()
        .leading(&props.leading)
        .trailing(&trailing)
        .states(field.states())
        .prepare();

    // The frame draws the ring, so the control must not draw a second one.
    let control = use_box()
        .framework_sx(&FIELD_CONTROL_SX)
        .focus_ring(false)
        .prepare();

    let mut attributes = state.a11y_attributes();
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
        .attr("autocomplete", "off")
        .event("oninput", move |event: FormEvent| {
            if let Some(oninput) = &oninput {
                oninput(event.value());
            }
            // The list changes under the highlight, so typing disarms it: the
            // next Enter belongs to whatever was typed, not to a row that
            // happens to sit where the old one did.
            state.set_active(None);
            state.open();
        })
        .event("onblur", move |_: FocusEvent| state.close())
        .element(&input_element)
        .render(HtmlTag::Input, attributes, ());

    let listbox = rsx! {
        ComboboxCore {
            rows,
            active: state.active(),
            onactive: move |row| state.set_active(Some(row)),
            opened: state.is_open() && !disabled && !readonly,
            onopened: move |opened| state.set_open(opened),
            state,
            empty: props.empty,
            size,
            radius,
            disabled,
            width: PopoverWidth::Match,
            {frame.render(input)}
        }
    };

    field.render(listbox)
}
