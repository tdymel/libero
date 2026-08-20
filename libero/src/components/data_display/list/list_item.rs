use dioxus::prelude::*;

use crate::{
    components::{HtmlTag, Input, States, common::base_props, layout::use_box},
    sx::{StaticSx, Sx, sx},
};

// Block, not flex, so a nested `List` stacks below this item's content rather
// than beside it. Row alignment is the content's own job.
static LIST_ITEM_BASE_SX: StaticSx = StaticSx::new(|| sx().padding("0"));

base_props! {
    pub struct ListItemProps {
        children: Element,
    }
}

#[component]
pub fn ListItem(props: ListItemProps) -> Element {
    use_box()
        .framework_sx(&LIST_ITEM_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        .render(HtmlTag::Li, props.attributes, props.children)
}
