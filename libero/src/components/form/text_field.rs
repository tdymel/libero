use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input,
        common::field_props,
        form::{FIELD_CONTROL_SX, use_bound, use_field, use_field_frame},
        layout::use_box,
    },
    hooks::use_theme,
};

field_props! {
    extends(input);
    pub struct TextFieldProps {
        /// The text to render. `None` leaves the `<input>` uncontrolled - it
        /// keeps its own text and needs no handler.
        #[props(default, into)]
        value: Option<String>,
        /// Fires per keystroke with the text the field should hold next.
        /// Native name, native timing.
        #[props(default)]
        oninput: Option<EventHandler<String>>,
        /// Rules over the text, shown once the field loses focus or its form
        /// is submitted.
        #[props(default, into)]
        validate: crate::components::Validators<String>,
        /// What the field posts as. A path - `Signup::FIELDS.email()` - also
        /// binds the text to the surrounding `Form`'s value when the field has
        /// no `oninput`.
        #[props(default, into)]
        name: crate::components::FieldName<String>,
        #[props(default, into)]
        placeholder: Option<String>,
        /// Inside the frame, before the control - a search icon, a currency
        /// prefix.
        #[props(default, into)]
        leading: Option<Element>,
        /// Inside the frame, after the control - a clear button, a unit.
        #[props(default, into)]
        trailing: Option<Element>,
    }
}

/// A single-line text field, with a label, a description, helper text and a
/// validation message stacked around it, and room either side of the control
/// inside the frame.
///
/// Controlled through `value` + `oninput`. Omit `value` and the `<input>` owns
/// its own text.
#[component]
pub fn TextField(props: TextFieldProps) -> Element {
    let theme = use_theme();

    let size = props.size.copied_or(theme.text_field.size);
    let radius = props.radius.copied_or(theme.text_field.radius);
    let required = props.required.unwrap_or(false);

    let bound = use_bound(&props.name, props.oninput.is_some());
    let disabled = bound.disabled(props.disabled);
    let value = bound.value().or_else(|| props.value.clone());

    let field = use_field()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(props.validate.check(&value.clone().unwrap_or_default()))
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

    let frame = use_field_frame()
        .leading(&props.leading)
        .trailing(&props.trailing)
        .states(field.states())
        .prepare();

    // The frame draws the ring, so the control must not draw a second one.
    let control = use_box()
        .framework_sx(&FIELD_CONTROL_SX)
        .focus_ring(false)
        .prepare();

    let oninput = bound.emit(props.oninput);
    let input = field
        .aria(control)
        .attr_default("type", "text")
        .attr("name", bound.name().map(str::to_string))
        .attr("value", value)
        .attr("placeholder", props.placeholder)
        .attr("disabled", disabled)
        .attr("required", required)
        .event(
            "oninput",
            oninput.map(|emit| move |event: FormEvent| emit(event.value())),
        )
        // Void element - `()` costs no dynamic node.
        .render(HtmlTag::Input, props.attributes, ());

    field.render(frame.render(input))
}
