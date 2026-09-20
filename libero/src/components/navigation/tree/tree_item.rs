use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, LogicalTextAlign, base_props},
        layout::use_box,
    },
    sx::{StaticSx, sx},
};

use super::tree_row::TreeRowContext;

// The page's type and the row's vertical padding are this button's job.
static TREE_ITEM_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("center")
        // Blitz's UA sheet centres a button's content.
        .justify_content("flex-start")
        .gap("6px")
        .width("100%")
        .padding("6px 0")
        .border("none")
        .background("none")
        .color("inherit")
        .font("inherit")
        .text_align_start()
        .cursor("pointer")
        // Disabled by its row or a `Fieldset` (todo 514); only the Fieldset's case dims here.
        .selector("&:disabled", sx().cursor("not-allowed"))
        .selector(
            "&:disabled:not([aria-disabled=\"true\"] *)",
            sx().opacity("0.5"),
        )
});

base_props! {
    pub struct TreeItemProps {
        #[props(default)]
        onclick: Option<EventHandler<MouseEvent>>,
        children: Element,
    }
}

/// A button row for a custom `render_node`, taking the row's tab stop and
/// `disabled`. A link row is a `NavLink` with `TreeNodeRenderArgs::tabindex`.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::{Tree, TreeItem, TreeNode, TreeNodeRenderArgs};
/// # fn app() -> Element {
/// rsx! {
///     Tree {
///         aria_label: "Actions",
///         data: vec![TreeNode::new("open", "Open")],
///         render_node: |args: TreeNodeRenderArgs<&'static str>| rsx! {
///             TreeItem { onclick: |_| {}, "{args.data}" }
///         },
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/navigation/tree>
#[component]
pub fn TreeItem(props: TreeItemProps) -> Element {
    // Outside a `Tree`, an ordinary button.
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
