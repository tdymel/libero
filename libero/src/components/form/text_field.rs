use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input,
        common::field_props,
        form::{
            FIELD_CONTROL_SX, LiveControl, LiveSlot, use_bound, use_field, use_field_frame,
            use_live_slot,
        },
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
        /// `leading` is text that belongs to the value - `"https://"`, `"@"` -
        /// so the input's `aria-describedby` reads it. Not for an icon or a button.
        #[props(default)]
        describe_leading: bool,
        /// `trailing` is text that belongs to the value - `"kg"`, `"12/20"`.
        #[props(default)]
        describe_trailing: bool,
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
    let mut props = props;
    let value = props.value.take();
    let mut live = use_signal(|| value.clone());
    if *live.peek() != value {
        live.set(value);
    }
    let leading = use_live_slot(props.leading.take());
    let trailing = use_live_slot(props.trailing.take());
    rsx! { TextFieldShell { live, leading, trailing, field: props } }
}

/// Everything but the text and the slots' content, so a keystroke or a
/// caller's new slot `Element` skips it.
#[component]
fn TextFieldShell(
    live: Signal<Option<String>>,
    leading: Option<Signal<Option<Element>>>,
    trailing: Option<Signal<Option<Element>>>,
    field: TextFieldProps,
) -> Element {
    let props = field;
    let leading = leading.map(|content| rsx! { LiveSlot { content } });
    let trailing = trailing.map(|content| rsx! { LiveSlot { content } });
    let theme = use_theme();

    let size = props.size.copied_or(theme.text_field.size);
    let radius = props.radius.copied_or(theme.text_field.radius);
    let required = props.required.unwrap_or(false);

    let bound = use_bound(&props.name, props.oninput.is_some());
    let disabled = bound.disabled(props.disabled);
    let readonly = props.readonly.unwrap_or(false);
    let bound_value = bound.value();
    // Rules read the text here, so only a validated field redraws per keystroke.
    let rules = (!props.validate.is_empty())
        .then(|| {
            bound.check(
                &props.validate,
                bound_value.clone().or_else(|| live.cloned()),
            )
        })
        .flatten();

    let field = use_field()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(rules)
        .bound(&bound)
        .required(required)
        .disabled(disabled)
        .size(size)
        .radius(radius)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .attributes(&props.attributes)
        .text_slots(
            props.describe_leading && leading.is_some(),
            props.describe_trailing && trailing.is_some(),
        )
        .prepare();

    let frame = use_field_frame()
        .leading(&leading)
        .trailing(&trailing)
        .states(field.states())
        .ids(field.slot_ids())
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
        .attr("placeholder", props.placeholder)
        .attr("disabled", disabled)
        .attr("readonly", readonly)
        .attr("required", required)
        .event(
            "oninput",
            oninput.map(|emit| move |event: FormEvent| emit(event.value())),
        );
    let attributes = props.attributes;
    let draw = Rc::new(move || {
        let value = bound_value.clone().or_else(|| live.cloned());
        input
            .clone()
            // A form reset leaves the text alone when this component sets it.
            .attr("data-controlled", value.is_some())
            .attr("value", value)
            // Void element - `()` costs no dynamic node.
            .render(HtmlTag::Input, attributes.clone(), ())
    });

    field.render(frame.render(rsx! { LiveControl { draw } }))
}
