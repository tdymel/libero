use dioxus::prelude::*;

use crate::{
    components::{HtmlTag, Input, States, common::base_props, layout::use_box},
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

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(chosen_size.state_name(), true)
        .into();

    use_box()
        .framework_sx(&TEXT_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare()
        .render(
            props.component.copied_or(HtmlTag::P),
            props.attributes,
            props.children,
        )
}
