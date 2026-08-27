use dioxus::prelude::*;

use crate::{
    components::{
        Caption, ClassList, HtmlTag, Input, Options, States,
        form::{FieldStatus, field_control_sx, use_field, use_field_frame},
        layout::use_box,
    },
    hooks::use_theme,
    sx::{StaticSx, Sx},
    theme::Size,
    utils::warn,
};

/// The control keeps the UA's own chevron - drawing our own would mean
/// `appearance: none`, and with it the native picker's arrow on every
/// platform.
static SELECT_CONTROL_SX: StaticSx = StaticSx::new(|| field_control_sx().cursor("pointer"));

// Hand-written rather than `field_props!`, which is not generic - as
// `TabsProps` is.
#[derive(Props, Clone, PartialEq)]
pub struct SelectProps<T: Options> {
    /// Strictly controlled - pair it with `onchange`. `None` shows
    /// `placeholder` and selects nothing.
    #[props(default)]
    value: Option<T>,
    /// Called with the option the caller should select next. Never fires for
    /// the placeholder, which cannot be picked.
    #[props(default)]
    onchange: Option<EventHandler<T>>,
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
    /// The field's caption, above the control.
    #[props(default, into)]
    label: Caption,
    /// Between the label and the control. What to pick.
    #[props(default, into)]
    description: Caption,
    /// Under the control. Constraints, consequences of the choice.
    #[props(default, into)]
    helper: Caption,
    /// Validation state, under the helper. A bare `&str` is an error.
    #[props(default, into)]
    status: Input<FieldStatus>,
    #[props(default, into)]
    size: Input<Size>,
    /// Corner radius, independent of `size`.
    #[props(default, into)]
    radius: Input<Size>,
    /// `None` is "not stated" - what a `Fieldset` will cascade into later.
    #[props(default)]
    disabled: Option<bool>,
    #[props(default)]
    required: Option<bool>,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default, into)]
    class: Input<ClassList>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
}

/// A styled native `<select>` over an enum, with a label, a description,
/// helper text and a validation message stacked around it. Controlled: it
/// renders `value` and asks for a new one through `onchange`.
///
/// The options are `T::options()` unless `options` narrows them - so a
/// misspelled option is a compile error, and `onchange` hands back the value
/// itself rather than a string the caller has to look up again.
#[component]
pub fn Select<T: Options>(props: SelectProps<T>) -> Element {
    let theme = use_theme();

    let size = props.size.copied_or(theme.select.size);
    let radius = props.radius.copied_or(theme.select.radius);
    let disabled = props.disabled.unwrap_or(false);
    let required = props.required.unwrap_or(false);

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
    let selected = props
        .value
        .as_ref()
        .and_then(|value| values.iter().position(|option| option == value));
    if props.value.is_some() && selected.is_none() && !values.is_empty() {
        warn("Select: `value` is not one of the options, so none is selected.");
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
        .framework_sx(&SELECT_CONTROL_SX)
        .focus_ring(false)
        .prepare();

    let onchange = props.onchange;
    let pick = use_callback(move |index: usize| {
        if let Some(onchange) = &onchange
            && let Some(value) = values.get(index)
        {
            onchange.call(value.clone());
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
        .attr("disabled", disabled)
        .attr("required", required)
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
