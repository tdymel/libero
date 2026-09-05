use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, Options,
        common::field_props,
        form::{field_control_sx, use_bound, use_field, use_field_frame},
        layout::use_box,
    },
    hooks::use_theme,
    sx::StaticSx,
    utils::warn,
};

/// The control keeps the UA's own chevron - drawing our own would mean
/// `appearance: none`, and with it the native picker's arrow on every
/// platform.
static NATIVE_SELECT_CONTROL_SX: StaticSx = StaticSx::new(|| field_control_sx().cursor("pointer"));

field_props! {
    pub struct NativeSelectProps<T: Options> {
        /// Strictly controlled - pair it with `onchange`. `None` shows
        /// `placeholder` and selects nothing.
        #[props(default)]
        value: Option<T>,
        /// Called with the option the caller should select next. Never fires
        /// for the placeholder, which cannot be picked.
        #[props(default)]
        onchange: Option<EventHandler<T>>,
        /// What the select posts as. A path - `Order::FIELDS.size()` - also
        /// binds it to the surrounding `Form`'s value when it has no
        /// `onchange`.
        #[props(default, into)]
        name: crate::components::FieldName<Option<T>>,
        /// Rules over the selection, shown once the select loses focus or its
        /// form is submitted.
        #[props(default, into)]
        validate: crate::components::Validators<Option<T>>,
        /// The options to show. Defaults to every `Options::options()` - which
        /// `String` and any other runtime type leave empty, so those pass them
        /// here.
        #[props(default)]
        options: Option<Vec<T>>,
        /// Overrides `Options::label`. Runs during render, so it can read a
        /// locale from context.
        ///
        /// Returns a `String`, not an `OptionLabel`: `<option>` holds text and
        /// nothing else, so there is no rich form to offer.
        #[props(default)]
        option_label: Option<Callback<T, String>>,
        /// Shown while `value` is `None`, as an unpickable first entry.
        #[props(default)]
        placeholder: Option<String>,
    }
}

/// A styled native `<select>` over an enum, with a label, a description,
/// helper text and a validation message stacked around it. Controlled: it
/// renders `value` and asks for a new one through `onchange`.
///
/// The options are `T::options()` unless `options` narrows them - so a
/// misspelled option is a compile error, and `onchange` hands back the value
/// itself rather than a string the caller has to look up again.
///
/// The real `<select>`, not a listbox: it keeps the OS picker on phones, works
/// without wasm, and renders its selection correctly under SSR. Reach for it
/// when those matter; reach for `Select` when the rows have to be styled.
#[component]
pub fn NativeSelect<T: Options>(props: NativeSelectProps<T>) -> Element {
    let theme = use_theme();

    let size = props.size.copied_or(theme.native_select.size);
    let radius = props.radius.copied_or(theme.native_select.radius);
    let required = props.required.unwrap_or(false);

    let bound = use_bound(&props.name, props.onchange.is_some());
    let disabled = bound.disabled(props.disabled);
    let current = bound.value().unwrap_or_else(|| props.value.clone());

    if props.onchange.is_none() && !bound.is_bound() {
        warn("NativeSelect: without `onchange` the selection can never change.");
    }
    // `field_props!` gives every field `readonly`, and this one cannot keep
    // the promise yet: HTML has no `readonly` for a `<select>`, and the only
    // ways to stop a native picker either take the field out of the tab order
    // or out of the form post. Warned rather than ignored - todo pending.
    if props.readonly.unwrap_or(false) {
        warn(
            "NativeSelect: `readonly` is not honoured yet - a native `<select>` has no read-only state.",
        );
    }

    let values = props
        .options
        .clone()
        .unwrap_or_else(|| T::options().to_vec());
    if values.is_empty() {
        warn("NativeSelect: no options - a `T` without static `options()` needs `options`.");
    }
    let selected = current
        .as_ref()
        .and_then(|value| values.iter().position(|option| option == value));
    if current.is_some() && selected.is_none() && !values.is_empty() {
        warn("NativeSelect: `value` is not one of the options, so none is selected.");
    }

    let labels: Vec<String> = values
        .iter()
        .map(|value| match &props.option_label {
            Some(label) => label.call(value.clone()),
            None => value.label(),
        })
        .collect();

    let field = use_field()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(props.validate.check(&current))
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

    let frame = use_field_frame().states(field.states()).prepare();

    // The frame draws the ring, so the control must not draw a second one.
    let control = use_box()
        .framework_sx(&NATIVE_SELECT_CONTROL_SX)
        .focus_ring(false)
        .prepare();

    let onchange = props.onchange;
    let setter = bound.setter();
    let pick = use_callback(move |index: usize| {
        let Some(value) = values.get(index) else {
            return;
        };
        match (&onchange, &setter) {
            (Some(onchange), _) => onchange.call(value.clone()),
            (None, Some(setter)) => setter.set(Some(value.clone())),
            (None, None) => {}
        }
    });

    let placeholder = props.placeholder.clone().unwrap_or_default();

    // `selected` on each `<option>`, not `value` on the `<select>`: the
    // property is written to the option itself, so it does not depend on the
    // parent's children already existing - which is what made the old
    // `value` path miss on the creating render, and what left SSR with no
    // selection at all.
    let select = field
        .aria(control)
        .attr("name", bound.name().map(str::to_string))
        .attr("disabled", disabled)
        .attr("required", required)
        .attr("data-controlled", true)
        .event("onchange", move |event: FormEvent| {
            if let Ok(index) = event.value().parse::<usize>() {
                pick.call(index);
            }
        })
        .render(
            HtmlTag::Select,
            props.attributes,
            rsx! {
                if selected.is_none() {
                    option {
                        value: "",
                        selected: true,
                        disabled: true,
                        hidden: true,
                        "{placeholder}"
                    }
                }
                for (index, label) in labels.iter().enumerate() {
                    option {
                        key: "{index}",
                        value: "{index}",
                        selected: selected == Some(index),
                        "{label}"
                    }
                }
            },
        );

    field.render(frame.render(select))
}
