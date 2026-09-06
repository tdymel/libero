use dioxus::prelude::*;

use crate::{
    components::{HtmlTag, Input, States, common::base_props, layout::use_box},
    hooks::use_theme,
    sx::StaticSx,
    theme::{Size, TitleDefaults},
};

static TITLE_BASE_SX: StaticSx = StaticSx::new(|| TitleDefaults::theme_vars().margin("0"));

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
        /// Defaults to `size`'s heading tag. Override to keep a size's visual
        /// weight under a different tag, preserving h1->h2->h3 order.
        #[props(default, into)]
        component: Input<HtmlTag>,
        children: Element,
    }
}

#[component]
pub fn Title(props: TitleProps) -> Element {
    let theme = use_theme();
    let chosen_size = props.size.copied_or(theme.title.size);

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(chosen_size.state_name(), true)
        .into();

    let component = props
        .component
        .as_ref()
        .copied()
        // The caller's size, never the theme's: a theme is a look, and must
        // not move a bare `Title` in the document outline.
        .unwrap_or_else(|| default_component(props.size.copied_or(Size::Xxl)));

    use_box()
        .framework_sx(&TITLE_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare()
        .render(component, props.attributes, props.children)
}
