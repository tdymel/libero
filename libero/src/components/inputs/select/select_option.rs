use dioxus::prelude::*;

use crate::{
    components::{HtmlTag, Input, States, common::base_props, layout::use_box},
    sx::Sx,
};

base_props! {
    extends(option);
    pub struct OptionProps {
        #[props(into)]
        value: String,
        children: Element,
    }
}

#[component]
pub fn Option(props: OptionProps) -> Element {
    use_box()
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        .attr("value", props.value)
        .render(HtmlTag::Option, props.attributes, props.children)
}
