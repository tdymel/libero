use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States,
        common::{base_props, focus_ring_sx},
        layout::use_box,
    },
    hooks::{use_root_id, use_theme},
    sx::{StaticSx, Sx, sx},
    theme::{Size, TextFieldDefaults},
    utils::warn,
};

static TEXT_FIELD_WRAPPER_SX: StaticSx =
    StaticSx::new(|| sx().display("flex").flex_direction("column").gap("4px"));

static TEXT_FIELD_LABEL_SX: StaticSx = StaticSx::new(TextFieldDefaults::label_theme_vars);

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
        .when(
            "disabled",
            sx().opacity("0.5")
                .cursor("not-allowed")
                .background("grey.1"),
        )
});

base_props! {
    extends(input);
    pub struct TextFieldProps {
        /// Strictly controlled - pair it with `onchange`. `None` is the empty
        /// field.
        #[props(default)]
        value: Option<String>,
        /// Called per keystroke with the text the field should hold next.
        #[props(default)]
        onchange: Option<EventHandler<String>>,
        #[props(default)]
        placeholder: Option<String>,
        #[props(default, into)]
        size: Input<Size>,
        /// Corner radius, independent of `size`.
        #[props(default, into)]
        radius: Input<Size>,
        #[props(default)]
        disabled: Option<bool>,
        /// The field's own caption, above the control.
        #[props(default)]
        label: Option<String>,
    }
}

/// A single-line text field with its own caption. Controlled: it renders
/// `value` and asks for a new one through `onchange`.
#[component]
pub fn TextField(props: TextFieldProps) -> Element {
    let theme = use_theme();

    let size = props.size.copied_or(theme.text_field.size);
    let radius = props.radius.copied_or(theme.text_field.radius);
    let disabled = props.disabled.unwrap_or(false);

    if props.onchange.is_none() {
        warn("TextField: without `onchange` the text can never change.");
    }

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with(radius.radius_state_name(), true)
        .with("disabled", disabled)
        .into();

    let id = use_root_id(&props.attributes);

    let wrapper = use_box()
        .framework_sx(&TEXT_FIELD_WRAPPER_SX)
        .class(&props.class)
        .sx(&props.sx)
        .prepare();
    let field = use_box()
        .framework_sx(&TEXT_FIELD_BASE_SX)
        .states(&states)
        .prepare();
    let label_box = use_box()
        .framework_sx(&TEXT_FIELD_LABEL_SX)
        .states(&states)
        .prepare();

    // Not a wrapping `<label>` like `Select`'s: it would hijack clicks on any
    // control the field grows later.
    let caption = match &props.label {
        Some(label) => {
            label_box
                .attr("for", id())
                .render(HtmlTag::Label, Vec::new(), rsx! { {label.clone()} })
        }
        None => rsx! {},
    };

    let onchange = props.onchange;
    let field = field
        .attr("id", id())
        .attr("type", "text")
        .attr("value", props.value.clone().unwrap_or_default())
        .attr("placeholder", props.placeholder)
        .attr("disabled", disabled)
        .event("oninput", move |event: FormEvent| {
            if let Some(onchange) = &onchange {
                onchange.call(event.value());
            }
        })
        // Void element - `()` costs no dynamic node.
        .render(HtmlTag::Input, props.attributes, ());

    wrapper.render(HtmlTag::Div, Vec::new(), vec![caption, field])
}
