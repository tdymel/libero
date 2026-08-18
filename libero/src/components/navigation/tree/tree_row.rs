use std::collections::HashSet;

use dioxus::prelude::*;

use crate::{
    components::{Box, List, common::focus_ring_sx},
    sx::{StaticSx, sx},
    theme::{LIST_GAP, LIST_INDENT, Size},
};

use super::{
    tree::toggle_expanded,
    tree_node::{ErasedRenderNode, TreeNodeErased, TreeNodeRenderArgsErased},
};

// The `<li role="treeitem">` itself - block-level, not flex, so a nested
// `List` (a node's children) stacks below its own row instead of beside it,
// same reasoning as `ListItem`'s own base. No hover background here on
// purpose: this element structurally contains its own children's `<li>`s,
// and `:hover` matches an ancestor whenever the pointer is over *any*
// descendant - putting the background here would make hovering a child also
// paint its whole ancestor chain. `disabled`'s opacity/pointer-events stay
// here instead, since dimming/blocking the entire subtree when a parent is
// disabled is the behavior we actually want, and `onclick`/`tabindex` live
// here too (`pointer-events: none` needs to block clicks reaching this
// element to have any effect).
static TREE_ROW_SX: StaticSx = StaticSx::new(|| {
    sx().focus_visible(focus_ring_sx()).when(
        "disabled",
        sx().opacity("0.5")
            .cursor("not-allowed")
            .pointer_events("none"),
    )
});

// The row's own content (whatever `render_node`/the default label produces)
// - a flex row nested inside the block-level `<li>`, and *not* an ancestor
// of the nested children `List` (that's a sibling of this element, not
// inside it) - so hover only ever paints this row's own content, never a
// descendant's. No navigation affordance (border, active color, etc.) here
// on purpose - that's specific to *navigable* content, which is entirely
// `render_node`'s call (e.g. a `NavLink` in a nav sidebar already has its
// own active styling); `Tree` itself doesn't know or care whether a leaf is
// a link.
//
// No *vertical* padding here on purpose, only horizontal - a border drawn
// by `render_node`'s own content (e.g. `NavLink`, via `align-self: stretch`)
// needs to span this element's *full* height to read as continuous between
// adjacent rows; vertical padding here would leave a gap at the top/bottom
// of every row that no border reaches. Content that wants vertical breathing
// room (like `default_tree_render`'s plain text) provides its own.
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
    pub gap: Option<Size>,
    pub indent: Option<Size>,
    pub depth: usize,
    pub expanded: Signal<HashSet<String>>,
    pub resolved_active: Option<String>,
    pub active_id: Signal<Option<String>>,
    pub render_node: ErasedRenderNode,
    pub onexpandedchange: EventHandler<HashSet<String>>,
}

#[component]
pub(super) fn TreeRow(props: TreeRowProps) -> Element {
    let node = &props.node;
    let has_children = node.has_children();
    let is_expanded = has_children.then(|| props.expanded.read().contains(&node.id));
    let disabled = node.disabled;
    let is_roving_active = props.resolved_active.as_deref() == Some(node.id.as_str());
    // The `<li>` carries the *real* roving tabindex ("0" only for the
    // active row). Whatever `render_node` renders always gets "-1",
    // unconditionally - it must never be independently tabbable, even on
    // the active row, or that row alone gets a second tab stop again.
    let li_tabindex = if is_roving_active { "0" } else { "-1" };

    // Whatever `render_node` draws is the row's entire content - `Tree`
    // itself doesn't reserve a leading chevron column. That's
    // `default_tree_render`'s own concern (see its doc comment); a custom
    // `render_node` that skips the chevron gets no leftover gap to explain.
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

    let onclick = move |_| {
        if disabled {
            return;
        }
        active_id.set(Some(id.clone()));
        if has_children {
            toggle_expanded(&id, expanded, onexpandedchange);
        }
    };

    rsx! {
        Box {
            component: "li",
            framework_sx: &TREE_ROW_SX,
            states: crate::components::common::states().with("disabled", disabled),
            "role": "treeitem",
            "data-tree-id": "{node.id}",
            "aria-expanded": is_expanded.map(|value| value.to_string()),
            "aria-disabled": disabled.then_some("true"),
            tabindex: li_tabindex,
            // `onclick` lives on this inner div, not the outer `<li>` - the
            // nested children `List` below is this div's *sibling*, not its
            // descendant, so a click bubbling up from a child row's own
            // content div physically cannot reach this one. That's what
            // keeps a leaf click from also toggling its parent's expanded
            // state, without needing `stop_propagation` (which would also
            // block a click from ever reaching whatever's outside the tree
            // entirely - not what we want here).
            Box {
                framework_sx: &TREE_ROW_CONTENT_SX,
                onclick,
                {content}
            }
            if is_expanded == Some(true) {
                List {
                    "role": "group",
                    sx: sx()
                        .apply_if(props.gap, |sx, gap| sx.gap(LIST_GAP.value(gap)))
                        .apply_if(props.indent, |sx, indent| {
                            sx.padding_left(LIST_INDENT.value(indent))
                        }),
                    size: props.size,
                    for child in &node.children {
                        TreeRow {
                            key: "{child.id}",
                            node: child.clone(),
                            size: props.size,
                            gap: props.gap,
                            indent: props.indent,
                            depth: props.depth + 1,
                            expanded: props.expanded,
                            resolved_active: props.resolved_active.clone(),
                            active_id: props.active_id,
                            render_node: props.render_node.clone(),
                            onexpandedchange: props.onexpandedchange,
                        }
                    }
                }
            }
        }
    }
}
