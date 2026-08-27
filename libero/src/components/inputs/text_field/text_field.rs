use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States,
        common::{field_props, focus_ring_sx},
        form::use_field,
        layout::use_box,
    },
    hooks::use_theme,
    sx::{StaticSx, Sx, sx},
    theme::TextFieldDefaults,
};

static TEXT_FIELD_BASE_SX: StaticSx = StaticSx::new(|| {
    TextFieldDefaults::theme_vars()
        .display("block")
        .width("100%")
        // An `<input>` inherits neither, so both would fall back to the UA's.
        .font_family("inherit")
        .line_height("normal")
        .border_style("solid")
        .border_width("1px")
        .border_color("grey.5")
        .background("white")
        .color("black")
        .selector("::placeholder", sx().color("grey.6"))
        .focus(sx().border_color("primary.6"))
        .focus_visible(focus_ring_sx())
        .when("error", sx().border_color("error.7"))
        .when("warning", sx().border_color("warning.7"))
        .when(
            "disabled",
            sx().opacity("0.5")
                .cursor("not-allowed")
                .background("grey.1"),
        )
});

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
        #[props(default, into)]
        placeholder: Option<String>,
    }
}

/// A single-line text field, with a label, a description, helper text and a
/// validation message stacked around it.
///
/// Controlled through `value` + `oninput`. Omit `value` and the `<input>` owns
/// its own text.
#[component]
pub fn TextField(props: TextFieldProps) -> Element {
    let theme = use_theme();

    let size = props.size.copied_or(theme.text_field.size);
    let radius = props.radius.copied_or(theme.text_field.radius);
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

    let control = use_box()
        .framework_sx(&TEXT_FIELD_BASE_SX)
        .states(field.states())
        .prepare();

    let oninput = props.oninput;
    let input = field
        .aria(control)
        .attr_default("type", "text")
        .attr("value", props.value.clone())
        .attr("placeholder", props.placeholder)
        .attr("disabled", disabled)
        .attr("required", required)
        .event(
            "oninput",
            oninput.map(|handler| move |event: FormEvent| handler.call(event.value())),
        )
        // Void element - `()` costs no dynamic node.
        .render(HtmlTag::Input, props.attributes, ());

    field.render(input)
}
