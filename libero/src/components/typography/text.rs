use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            Cut, HtmlTag, Input, States, Truncate, base_color, base_props, clamp_height_sx,
            clamp_sx, gradient_text_sx, text_color, truncate_sx, variables, with_own,
        },
        layout::use_box,
    },
    hooks::{use_gradient_style, use_range_css, use_theme},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{CssVar, Gradient, Responsive, Size, TEXT_LINE_HEIGHT, TextDefaults},
};

static TEXT_BASE_SX: StaticSx = StaticSx::new(|| {
    TextDefaults::theme_vars()
        .and(truncate_sx(TEXT_LINE_HEIGHT))
        .margin("0")
        .padding("0")
        .text_decoration("none")
        // A long word overflowed a 320px column (WCAG 1.4.10), as on `Title`.
        .overflow_wrap("break-word")
        .when("colored", sx().color(TEXT_COLOR.value()))
        .when("gradient", gradient_text_sx())
});

const TEXT_COLOR: CssVar = CssVar::new("--lsx-text-color");

base_props! {
    pub struct TextProps {
        /// Per breakpoint too: `responsive(Size::Sm).md(Size::Lg)`.
        #[props(default, into)]
        size: Input<Responsive<Size>>,
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
        /// Cuts a text on one line with an ellipsis at its `end` or `start`. The element
        /// becomes a block, so a `span` truncates too. `line_clamp` wins when both are set.
        /// Blitz paints no ellipsis: it clips the text at the cut end.
        #[props(default, into)]
        truncate: Input<Truncate>,
        /// Cuts the text after this many lines with an ellipsis; over `truncate`, and `0`
        /// is off. Blitz has no line clamp: it cuts at the line height, without an ellipsis.
        /// Per breakpoint too, `responsive(2).md(0)`, where `0` lifts the clamp in that range.
        #[props(default, into)]
        line_clamp: Input<Responsive<u32>>,
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
    let sizes = props.size.copied_or(theme.text.size.into());
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
        .and_then(|color| text_color(&base_color("Text", Some(color))));
    let colored = plain.is_some();
    let style = gradient.or_else(|| colored.then(|| variables().with(TEXT_COLOR, plain).render()));

    let cut = Cut::of(&props.truncate, &props.line_clamp);
    let states: Input<States> = cut
        .states()
        .into_iter()
        .fold(
            props
                .states
                .unwrap_or_default()
                .responsive(&sizes, |size| size.state_name())
                .with("colored", colored)
                .with("gradient", props.gradient.is_some()),
            |states, (state, on)| states.with(state, on),
        )
        .into();
    let lines = cut.variables();
    let sx = with_own(clamp_sx(&props.line_clamp), &props.sx);
    use_range_css(
        &TEXT_BASE_SX,
        sizes.range_copies(
            |size| size.state_name(),
            |size| TextDefaults::size_sx(size).and(clamp_height_sx(TEXT_LINE_HEIGHT, size)),
        ),
    );
    let children = match cut.is_start() {
        true => rsx! { bdi { {props.children} } },
        false => props.children,
    };

    use_box()
        .framework_sx(&TEXT_BASE_SX)
        .class(&props.class)
        .sx(&sx)
        .states(&states)
        .variables(&lines)
        .style(style)
        .prepare()
        .render(
            props.component.copied_or(HtmlTag::P),
            props.attributes,
            children,
        )
}
