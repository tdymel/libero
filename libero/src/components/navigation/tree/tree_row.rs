use dioxus::prelude::*;

use crate::{
    components::{
        common::states,
        common::{HtmlTag, focus_ring_sx},
        data_display::List,
        layout::use_box,
    },
    sx::{StaticSx, sx},
    theme::Size,
};

use super::{
    tree::Expansion,
    tree_node::{ErasedRenderNode, TreeNodeErased, TreeNodeRenderArgsErased},
};

/// The row's tabindex and `disabled`, for its content. A signal, as
/// `use_context_provider` runs once.
#[derive(Clone, Copy)]
pub(super) struct TreeRowContext(pub Signal<TreeRowState>);

#[derive(Clone, Copy, PartialEq)]
pub(super) struct TreeRowState {
    pub tabindex: &'static str,
    pub disabled: bool,
}

// No hover here: the `<li>` holds its descendants, so it would paint the chain.
// Only the subtree drops the pointer, so the `<li>` shows `not-allowed` (todo 596).
static TREE_ROW_SX: StaticSx = StaticSx::new(|| {
    sx().focus_visible(focus_ring_sx()).when(
        "disabled",
        sx().opacity("0.5")
            .cursor("not-allowed")
            .selector("& *", sx().pointer_events("none")),
    )
});

// A sibling of the children `List`, keeping hover row-local. No vertical padding,
// so a border in `render_node`'s content spans the full row.
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
    pub expansion: Expansion,
    /// The tab stop's path from this row: `Some(&[])` is this row, `None` off the
    /// path, so a move re-renders only the rows it leaves and enters.
    pub active: Option<Vec<usize>>,
    /// Where `Tree`'s `current` row is, relative to this row, as `active`.
    #[props(default)]
    pub current: Option<Vec<usize>>,
    pub active_id: Signal<Option<String>>,
    pub render_node: ErasedRenderNode,
    /// Under a disabled branch, so the row acts disabled too.
    #[props(default)]
    pub ancestor_disabled: bool,
}

#[component]
pub(super) fn TreeRow(props: TreeRowProps) -> Element {
    let node = &props.node;
    let has_children = node.has_children();
    let is_expanded = has_children.then(|| props.expansion.open.read().contains(&node.id));
    let disabled = node.disabled || props.ancestor_disabled;
    let is_roving_active = props.active.as_deref().is_some_and(<[usize]>::is_empty);
    // The `<li>` is the roving tab stop; `render_node`'s content is always "-1".
    let li_tabindex = if is_roving_active { "0" } else { "-1" };

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

    // No chevron column here: that's `default_tree_render`'s.
    let content = props.render_node.call(TreeNodeRenderArgsErased {
        id: node.id.clone(),
        data: node.data.clone(),
        expanded: is_expanded,
        disabled,
        tabindex: "-1",
        depth: props.depth,
    });

    let mut active_id = props.active_id;
    let expansion = props.expansion;
    let id = node.id.clone();

    let onclick = move |_: Event<MouseData>| {
        if disabled {
            return;
        }
        active_id.set(Some(id.clone()));
        if has_children {
            expansion.toggle(&id);
        }
    };

    // The node's own flag: the ancestor's opacity already covers this row.
    let row_states = states().with("disabled", node.disabled).into();
    let row = use_box()
        .framework_sx(&TREE_ROW_SX)
        .states(&row_states)
        .prepare();
    // On the inner div, a sibling of the children `List`, so a child's click
    // never toggles the parent.
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
                            expansion: props.expansion,
                            active: child_active(props.active.as_deref(), index),
                            current: child_active(props.current.as_deref(), index),
                            active_id: props.active_id,
                            render_node: props.render_node.clone(),
                            ancestor_disabled: disabled,
                        }
                    }
                }
            }
        },
        None => row_content,
    };

    let is_current = props.current.as_deref().is_some_and(<[usize]>::is_empty);
    row.attr("role", "treeitem")
        .attr("data-tree-id", node.id.to_string())
        // No selection model: without an explicit "false" Chrome announces the
        // tab-stop row as selected. `current` speaks through `aria-current`.
        .attr("aria-selected", "false")
        .attr("aria-current", is_current.then_some("true"))
        .attr("aria-expanded", is_expanded.map(|value| value.to_string()))
        .attr("aria-disabled", disabled.then_some("true"))
        .attr("tabindex", li_tabindex)
        // A press would focus a row the roving stop is not on, where no key works.
        .event("onmousedown", move |event: Event<MouseData>| {
            if disabled {
                event.prevent_default();
            }
        })
        .render(HtmlTag::Li, Vec::new(), children)
}

/// The part of a row's `active` path that its child `index` sees.
pub(super) fn child_active(active: Option<&[usize]>, index: usize) -> Option<Vec<usize>> {
    match active? {
        [head, rest @ ..] if *head == index => Some(rest.to_vec()),
        _ => None,
    }
}
