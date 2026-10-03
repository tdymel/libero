use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, States, base_props},
        layout::use_box,
    },
    hooks::{use_gradient_style, use_theme},
    sx::StaticSx,
    theme::{Gradient, Size, TitleDefaults},
};

use super::text::gradient_text_sx;

static TITLE_BASE_SX: StaticSx = StaticSx::new(|| {
    TitleDefaults::theme_vars()
        .margin("0")
        .when("gradient", gradient_text_sx())
});

/// `Xxl` is h1, down to `Xs` as h6.
fn default_component(size: Size) -> HtmlTag {
    match size {
        Size::Xxl => HtmlTag::H1,
        Size::Xl => HtmlTag::H2,
        Size::Lg => HtmlTag::H3,
        Size::Md => HtmlTag::H4,
        Size::Sm => HtmlTag::H5,
        Size::Xs => HtmlTag::H6,
    }
}

base_props! {
    pub struct TitleProps {
        #[props(default, into)]
        size: Input<Size>,
        /// Unset, `size`'s heading tag. Set it to keep the outline order under another size.
        #[props(default, into)]
        component: Input<HtmlTag>,
        /// Paints the glyphs in a gradient from the theme's first stop: `("secondary", 45)` or
        /// a [`Gradient`]. A literal stop's contrast is the caller's to check.
        #[props(default, into)]
        gradient: Option<Gradient>,
        children: Element,
    }
}

/// A heading, `h1` to `h6` by `size`.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Title;
/// # fn app() -> Element {
/// rsx! {
///     Title { size: "xl", "Getting started" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/typography/title>
#[component]
pub fn Title(props: TitleProps) -> Element {
    let theme = use_theme();
    let chosen_size = props.size.copied_or(theme.title.size);
    let style = use_gradient_style(
        props.gradient.as_ref(),
        None,
        props.gradient.is_some(),
        true,
    );

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(chosen_size.state_name(), true)
        .with("gradient", props.gradient.is_some())
        .into();

    let component = props
        .component
        .as_ref()
        .copied()
        // The caller's size, never the theme's: a theme must not move the outline.
        .unwrap_or_else(|| default_component(props.size.copied_or(Size::Xxl)));

    use_box()
        .framework_sx(&TITLE_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .style(style)
        .prepare()
        .render(component, props.attributes, props.children)
}
