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
        // A long unbreakable label wraps instead of widening the page (1.4.10).
        .with("overflow-wrap", "anywhere")
        .hover(sx().background("muted.2"))
});

#[derive(Props, Clone, PartialEq)]
pub(super) struct TreeRowProps {
    pub node: TreeNodeErased,
    pub size: Size,
    pub depth: usize,
    pub expanded: Signal<HashSet<String>>,
    /// Where the roving tab stop is, relative to this row: `Some(&[])` is this
    /// row, `Some([i, ..])` is inside its child `i`. A path and not an id, so
    /// that a repeated id still makes exactly one tab stop. It is `None` on
    /// every row off that path, so moving the stop re-renders only the rows it
    /// leaves and enters.
    pub active: Option<Vec<usize>>,
    pub active_id: Signal<Option<String>>,
    pub render_node: ErasedRenderNode,
    pub onexpandedchange: Option<EventHandler<HashSet<String>>>,
    /// Under a disabled branch: the row acts disabled, as `TREE_ROW_SX`'s
    /// subtree-wide `pointer-events` already makes it for the mouse.
    #[props(default)]
    pub ancestor_disabled: bool,
}

#[component]
pub(super) fn TreeRow(props: TreeRowProps) -> Element {
    let node = &props.node;
    let has_children = node.has_children();
    let is_expanded = has_children.then(|| props.expanded.read().contains(&node.id));
    let disabled = node.disabled || props.ancestor_disabled;
    let is_roving_active = props.active.as_deref().is_some_and(<[usize]>::is_empty);
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

    // The node's own flag: the ancestor's opacity already covers this row.
    let row_states = states().with("disabled", node.disabled).into();
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

    // A leaf skips the conditional slot. A branch keeps one template either
    // way, so a toggle does not remount its content.
    let children = match is_expanded {
        Some(open) => rsx! {
            {row_content}
            if open {
                List {
                    "role": "group",
                    size: props.size,
                    for (index , child) in node.children.iter().enumerate() {
                        TreeRow {
                            key: "{child.id}",
                            node: child.clone(),
                            size: props.size,
                            depth: props.depth + 1,
                            expanded: props.expanded,
                            active: child_active(props.active.as_deref(), index),
                            active_id: props.active_id,
                            render_node: props.render_node.clone(),
                            onexpandedchange: props.onexpandedchange,
                            ancestor_disabled: disabled,
                        }
                    }
                }
            }
        },
        None => row_content,
    };

    row.attr("role", "treeitem")
        .attr("data-tree-id", node.id.to_string())
        .attr("aria-expanded", is_expanded.map(|value| value.to_string()))
        .attr("aria-disabled", disabled.then_some("true"))
        .attr("tabindex", li_tabindex)
        .render(HtmlTag::Li, Vec::new(), children)
}

/// The part of a row's `active` path that its child `index` sees.
pub(super) fn child_active(active: Option<&[usize]>, index: usize) -> Option<Vec<usize>> {
    match active? {
        [head, rest @ ..] if *head == index => Some(rest.to_vec()),
        _ => None,
    }
}
