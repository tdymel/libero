use dioxus::prelude::*;

use crate::{
    components::{HtmlTag, Input, States, common::base_props, layout::use_box},
    sx::{StaticSx, Sx, sx},
};

use super::tree_row::TreeRowContext;

// A `<button>` inherits none of the page's type, and `Tree` puts no vertical
// padding on the row - both are this element's job.
static TREE_ITEM_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("center")
        .gap("6px")
        .width("100%")
        .padding("6px 0")
        .border("none")
        .background("none")
        .color("inherit")
        .font("inherit")
        .text_align("left")
        .cursor("pointer")
});

base_props! {
    pub struct TreeItemProps {
        #[props(default)]
        onclick: Option<EventHandler<MouseEvent>>,
        children: Element,
    }
}

/// A row's interactive content, for a custom `render_node`. Renders the
/// `<button>` `Tree`'s Enter/Space handling looks for, and takes the row's
/// tab stop and `disabled` from the row itself - so neither can be forgotten.
///
/// A row that is a *link* is `NavLink`, not this: pass it
/// `TreeNodeRenderArgs::tabindex` yourself.
#[component]
pub fn TreeItem(props: TreeItemProps) -> Element {
    // Outside a `Tree` there is no roving tab stop to stay out of the way of,
    // so the button is an ordinary one.
    let row = use_hook(try_consume_context::<TreeRowContext>);
    let (tabindex, disabled) = match row {
        Some(row) => {
            let state = row.0.read();
            (state.tabindex, state.disabled)
        }
        None => ("0", false),
    };

    use_box()
        .framework_sx(&TREE_ITEM_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        .attr("type", "button")
        .attr("tabindex", tabindex)
        .attr("disabled", disabled)
        .event("onclick", move |event: MouseEvent| {
            if let Some(onclick) = &props.onclick {
                onclick.call(event);
            }
        })
        .render(HtmlTag::Button, props.attributes, props.children)
}
