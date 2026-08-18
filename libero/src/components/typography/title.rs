use dioxus::prelude::*;

use crate::{
    components::{Box, HtmlTag, Input, States, common::base_props},
    sx::{StaticSx, Sx},
    theme::{Size, TitleDefaults},
};

static TITLE_BASE_SX: StaticSx = StaticSx::new(|| TitleDefaults::theme_vars().margin("0"));

/// The heading tag `size` implies by default - `Xxl` is h1 (the largest,
/// most prominent), down to `Xs` as h6. Override via `component` to keep a
/// size's visual weight while using a different semantic tag, e.g. to
/// preserve a page's h1->h2->h3 a11y heading order.
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
        /// Which element to render as - defaults to `size`'s own heading tag
        /// (see `default_component`). Override to keep `size`'s visual
        /// weight while using a different semantic tag, e.g. to preserve a
        /// page's h1->h2->h3 a11y heading order.
        #[props(default, into)]
        component: Input<HtmlTag>,
        children: Element,
    }
}

#[component]
pub fn Title(props: TitleProps) -> Element {
    let chosen_size = props.size.copied_or(Size::Xxl);

    let states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with(chosen_size.state_name(), true);

    let component = props
        .component
        .as_ref()
        .copied()
        .unwrap_or_else(|| default_component(chosen_size));

    rsx! {
        Box {
            class: props.class,
            sx: props.sx,
            states,
            component,
            framework_sx: &TITLE_BASE_SX,
            attributes: props.attributes,
            {props.children}
        }
    }
}
