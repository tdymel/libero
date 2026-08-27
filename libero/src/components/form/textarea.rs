use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input,
        common::field_props,
        form::{field_control_sx, use_field, use_field_frame},
        layout::use_box,
    },
    hooks::use_theme,
    sx::StaticSx,
};

/// The control's own additions to [`field_control_sx`]: a `<textarea>` is not
/// inline, and it keeps the drag handle the browser gives it for free.
static TEXTAREA_CONTROL_SX: StaticSx = StaticSx::new(|| {
    field_control_sx()
        .display("block")
        .resize("vertical")
        // The frame's `min-height` is a single line's floor; `rows` is what
        // decides the box, so the control must not inherit that floor.
        .min_height("0")
});

field_props! {
    extends(textarea);
    pub struct TextareaProps {
        /// The text to render. `None` leaves the `<textarea>` uncontrolled -
        /// it keeps its own text and needs no handler.
        #[props(default, into)]
        value: Option<String>,
        /// Fires per keystroke with the text the field should hold next.
        /// Native name, native timing.
        #[props(default)]
        oninput: Option<EventHandler<String>>,
        #[props(default, into)]
        placeholder: Option<String>,
        /// Visible lines, which is what sets the starting height. The user can
        /// still drag it taller.
        #[props(default = 3)]
        rows: u32,
    }
}

/// A multi-line text field, with a label, a description, helper text and a
/// validation message stacked around it.
///
/// `rows` sets the starting height and the browser's own drag handle takes it
/// from there. Controlled through `value` + `oninput`; omit `value` and the
/// `<textarea>` owns its own text.
#[component]
pub fn Textarea(props: TextareaProps) -> Element {
    let theme = use_theme();

    let size = props.size.copied_or(theme.textarea.size);
    let radius = props.radius.copied_or(theme.textarea.radius);
    let disabled = props.disabled.unwrap_or(false);
    let required = props.required.unwrap_or(false);

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
        .framework_sx(&TEXTAREA_CONTROL_SX)
        .focus_ring(false)
        .prepare();

    let oninput = props.oninput;
    let textarea = field
        .aria(control)
        .attr("value", props.value.clone())
        .attr("rows", props.rows.to_string())
        .attr("placeholder", props.placeholder)
        .attr("disabled", disabled)
        .attr("required", required)
        .event(
            "oninput",
            oninput.map(|handler| move |event: FormEvent| handler.call(event.value())),
        )
        .render(HtmlTag::Textarea, props.attributes, ());

    field.render(frame.render(textarea))
}
