use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, States, base_props},
        layout::use_box,
    },
    hooks::{use_gradient_style, use_theme},
    platform::clips_background_to_text,
    sx::{StaticSx, Sx, sx},
    theme::{GRADIENT_FROM, Gradient, Size, TextDefaults, gradient_image},
};

static TEXT_BASE_SX: StaticSx = StaticSx::new(|| {
    TextDefaults::theme_vars()
        .margin("0")
        .padding("0")
        .text_decoration("none")
        .when("gradient", gradient_text_sx())
});

/// The gradient through the glyphs. Forced colours drop the image and force
/// the colour, so the text stays readable. Blitz paints the whole box, so
/// there it is the first stop, solid.
fn gradient_text_sx() -> Sx {
    if !clips_background_to_text() {
        return sx().color(GRADIENT_FROM.value());
    }
    sx().color("transparent")
        .background_image(gradient_image(None))
        .background_clip("text")
        .with("-webkit-background-clip", "text")
}

base_props! {
    pub struct TextProps {
        #[props(default, into)]
        size: Input<Size>,
        /// Which element to render as - `p` by default.
        #[props(default, into)]
        component: Input<HtmlTag>,
        /// Paints the glyphs in a linear gradient; `Gradient::default()` is the
        /// theme's. Palette stops take their text role, which reads on the
        /// page; a literal stop's contrast is the caller's to check. A debug
        /// build warns when a hex stop falls under 4.5:1 on the page background.
        #[props(default)]
        gradient: Option<Gradient>,
        children: Element,
    }
}

#[component]
pub fn Text(props: TextProps) -> Element {
    let theme = use_theme();
    let chosen_size = props.size.copied_or(theme.text.size);
    let gradient = use_gradient_style(props.gradient.as_ref(), true, true);

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(chosen_size.state_name(), true)
        .with("gradient", gradient.is_some())
        .into();

    use_box()
        .framework_sx(&TEXT_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .style(gradient)
        .prepare()
        .render(
            props.component.copied_or(HtmlTag::P),
            props.attributes,
            props.children,
        )
}
