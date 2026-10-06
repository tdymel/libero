use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, States, base_color, base_props, text_color, variables},
        layout::use_box,
    },
    hooks::{use_gradient_style, use_theme},
    platform::clips_background_to_text,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{CssVar, GRADIENT_FROM, Gradient, Size, TextDefaults, gradient_image},
};

static TEXT_BASE_SX: StaticSx = StaticSx::new(|| {
    TextDefaults::theme_vars()
        .margin("0")
        .padding("0")
        .text_decoration("none")
        .when("colored", sx().color(TEXT_COLOR.value()))
        .when("gradient", gradient_text_sx())
});

const TEXT_COLOR: CssVar = CssVar::new("--lsx-text-color");

/// The gradient through the glyphs. Blitz paints the whole box, so there it is the first stop.
pub(super) fn gradient_text_sx() -> Sx {
    if !clips_background_to_text() {
        return sx().color(GRADIENT_FROM.value());
    }
    sx().color("transparent")
        .background_image(gradient_image(None))
        .background_clip("text")
        .with("-webkit-background-clip", "text")
        // Print drops backgrounds by default, so the text would print blank; `exact` left a
        // hairline round the clip in Chromium's PDF, hence solid in the first stop.
        .media(
            "print",
            sx().color(GRADIENT_FROM.value()).background_image("none"),
        )
}

base_props! {
    pub struct TextProps {
        #[props(default, into)]
        size: Input<Size>,
        /// The element to render, `p` by default.
        #[props(default, into)]
        component: Input<HtmlTag>,
        /// The glyphs' colour: a palette colour in its text shade, a literal as given.
        /// The first stop under a `gradient`.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Paints the glyphs in a gradient from `color`: `("secondary", 45)` or a
        /// [`Gradient`]. A literal stop's contrast is the caller's to check.
        #[props(default, into)]
        gradient: Option<Gradient>,
        children: Element,
    }
}

/// Themed body text, a `p` unless `component` says otherwise.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Text;
/// # fn app() -> Element {
/// rsx! {
///     Text { size: "sm", "Fine print" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/typography/text>
#[component]
pub fn Text(props: TextProps) -> Element {
    let theme = use_theme();
    let chosen_size = props.size.copied_or(theme.text.size);
    let color = props.color.as_ref();
    let gradient = use_gradient_style(
        props.gradient.as_ref(),
        color,
        props.gradient.is_some(),
        true,
    );
    // Under a gradient the colour is its first stop instead.
    let plain = color
        .filter(|_| gradient.is_none())
        .and_then(|color| text_color(&base_color(Some(color))));
    let colored = plain.is_some();
    let style = gradient.or_else(|| colored.then(|| variables().with(TEXT_COLOR, plain).render()));

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(chosen_size.state_name(), true)
        .with("colored", colored)
        .with("gradient", props.gradient.is_some())
        .into();

    use_box()
        .framework_sx(&TEXT_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .style(style)
        .prepare()
        .render(
            props.component.copied_or(HtmlTag::P),
            props.attributes,
            props.children,
        )
}
