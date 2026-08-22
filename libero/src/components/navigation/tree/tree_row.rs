use std::collections::HashSet;

use dioxus::prelude::*;

use crate::{
    components::{HtmlTag, List, common::focus_ring_sx, common::states, layout::use_box},
    sx::{StaticSx, sx},
    theme::Size,
};

use super::{
    tree::toggle_expanded,
    tree_node::{ErasedRenderNode, TreeNodeErased, TreeNodeRenderArgsErased},
};

/// What a row's content needs from the row: the tab stop it must not steal,
/// and whether the node is disabled. Signal-backed - `use_context_provider`
/// runs once, so plain values would freeze at the first render.
#[derive(Clone, Copy)]
pub(super) struct TreeRowContext(pub Signal<TreeRowState>);

#[derive(Clone, Copy, PartialEq)]
pub(super) struct TreeRowState {
    pub tabindex: &'static str,
    pub disabled: bool,
}

// The `<li role="treeitem">`. Deliberately no hover background: this element
// contains its descendants' `<li>`s, so `:hover` here would paint the whole
// ancestor chain. `disabled` does want that subtree-wide reach, so it stays.
static TREE_ROW_SX: StaticSx = StaticSx::new(|| {
    sx().focus_visible(focus_ring_sx()).when(
        "disabled",
        sx().opacity("0.5")
            .cursor("not-allowed")
            .pointer_events("none"),
    )
});

// The row's own content, a sibling of the nested children `List` - which is
// what keeps hover row-local. Horizontal padding only: a border drawn by
// `render_node`'s content must span the full row height to read as continuous,
// so vertical breathing room is that content's own job.
static TREE_ROW_CONTENT_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("center")
        .gap("6px")
        .padding_left("8px")
        .padding_right("8px")
        .border_radius("4px")
        .cursor("pointer")
        .hover(sx().background("grey.2"))
});

#[derive(Props, Clone, PartialEq)]
pub(super) struct TreeRowProps {
    pub node: TreeNodeErased,
    pub size: Size,
    pub depth: usize,
    pub expanded: Signal<HashSet<String>>,
    pub resolved_active: Option<String>,
    pub active_id: Signal<Option<String>>,
    pub render_node: ErasedRenderNode,
    pub onexpandedchange: Option<EventHandler<HashSet<String>>>,
}

#[component]
pub(super) fn TreeRow(props: TreeRowProps) -> Element {
    let node = &props.node;
    let has_children = node.has_children();
    let is_expanded = has_children.then(|| props.expanded.read().contains(&node.id));
    let disabled = node.disabled;
    let is_roving_active = props.resolved_active.as_deref() == Some(node.id.as_str());
    // The `<li>` carries the roving tabindex; `render_node`'s content is
    // always "-1", or the active row gets a second tab stop.
    let li_tabindex = if is_roving_active { "0" } else { "-1" };

    // `render_node`'s content is always "-1": the `<li>` is the roving tab
    // stop, and a second one would strand arrow-key navigation.
    let mut row_context = use_context_provider(|| {
        Signal::new(TreeRowState {
            tabindex: "-1",
            disabled,
        })
    });
    if row_context.peek().disabled != disabled {
        row_context.write().disabled = disabled;
    }
    use_context_provider(|| TreeRowContext(row_context));

    // `Tree` reserves no chevron column - that's `default_tree_render`'s
    // concern, so a custom `render_node` leaves no gap behind.
    let content = props.render_node.call(TreeNodeRenderArgsErased {
        id: node.id.clone(),
        data: node.data.clone(),
        expanded: is_expanded,
        disabled,
        tabindex: "-1",
        depth: props.depth,
    });

    let mut active_id = props.active_id;
    let expanded = props.expanded;
    let id = node.id.clone();
    let onexpandedchange = props.onexpandedchange;

    let onclick = move |_: Event<MouseData>| {
        if disabled {
            return;
        }
        active_id.set(Some(id.clone()));
        if has_children {
            toggle_expanded(&id, expanded, onexpandedchange);
        }
    };

    let row_states = states().with("disabled", disabled).into();
    let row = use_box()
        .framework_sx(&TREE_ROW_SX)
        .states(&row_states)
        .prepare();
    // `onclick` sits on this inner div, not the `<li>`: the children `List` is
    // its sibling, so a child's click can't bubble here and toggle the parent -
    // no `stop_propagation` needed.
    let row_content = use_box()
        .framework_sx(&TREE_ROW_CONTENT_SX)
        .prepare()
        .event("onclick", onclick)
        .render(HtmlTag::Div, Vec::new(), content);

    // Split on `is_expanded` instead of putting an `if` in one block: a
    // conditional node costs its slot on every render of every *leaf* too.
    let children = match is_expanded {
        Some(true) => rsx! {
            {row_content}
            List {
                "role": "group",
                size: props.size,
                for child in &node.children {
                    TreeRow {
                        key: "{child.id}",
                        node: child.clone(),
                        size: props.size,
                        depth: props.depth + 1,
                        expanded: props.expanded,
                        resolved_active: props.resolved_active.clone(),
                        active_id: props.active_id,
                        render_node: props.render_node.clone(),
                        onexpandedchange: props.onexpandedchange,
                    }
                }
            }
        },
        _ => row_content,
    };

    row.attr("role", "treeitem")
        .attr("data-tree-id", node.id.to_string())
        .attr("aria-expanded", is_expanded.map(|value| value.to_string()))
        .attr("aria-disabled", disabled.then_some("true"))
        .attr("tabindex", li_tabindex)
        .render(HtmlTag::Li, Vec::new(), children)
}
