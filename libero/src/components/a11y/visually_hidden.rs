use dioxus::prelude::*;

use crate::{
    components::{HtmlTag, Input, common::base_props, layout::use_box},
    sx::{StaticSx, sx},
};

pub(crate) static VISUALLY_HIDDEN_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .width("1px")
        .height("1px")
        .padding("0")
        .margin("-1px")
        .overflow("hidden")
        .clip("rect(0, 0, 0, 0)")
        .white_space("nowrap")
        .border_width("0")
});

base_props! {
    pub struct VisuallyHiddenProps {
        children: Element,
    }
}

#[component]
pub fn VisuallyHidden(props: VisuallyHiddenProps) -> Element {
    use_box()
        .framework_sx(&VISUALLY_HIDDEN_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        .render(HtmlTag::Span, props.attributes, props.children)
}
