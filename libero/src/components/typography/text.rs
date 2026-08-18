use dioxus::prelude::*;

use crate::{
    components::{Box, HtmlTag, Input, States, common::base_props},
    sx::{StaticSx, Sx},
    theme::{Size, TextDefaults},
};

static TEXT_BASE_SX: StaticSx = StaticSx::new(|| {
    TextDefaults::theme_vars()
        .margin("0")
        .padding("0")
        .text_decoration("none")
});

base_props! {
    pub struct TextProps {
        #[props(default, into)]
        size: Input<Size>,
        /// Which element to render as - `p` by default.
        #[props(default, into)]
        component: Input<HtmlTag>,
        children: Element,
    }
}

#[component]
pub fn Text(props: TextProps) -> Element {
    let chosen_size = props.size.copied_or(Size::Md);

    let states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with(chosen_size.state_name(), true);

    let component = props.component.copied_or(HtmlTag::P);

    rsx! {
        Box {
            class: props.class,
            sx: props.sx,
            states,
            component,
            framework_sx: &TEXT_BASE_SX,
            attributes: props.attributes,
            {props.children}
        }
    }
}
