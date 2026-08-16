use std::{
    collections::HashSet,
    sync::atomic::{AtomicU64, Ordering},
};

use dioxus::{document, prelude::*};

use crate::{
    components::{Box, Icon, Input, List, common::focus_ring_sx},
    hooks::use_theme,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{Size, SizeCss},
};

use super::tree_node::{TreeLabel, TreeNode, TreeNodeRenderArgs, default_tree_render};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

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

// The row's own content (leading chevron/spacer + whatever
// `render_node`/the default label produces) - a flex row nested inside the
// block-level `<li>`, and *not* an ancestor of the nested children `List`
// (that's a sibling of this element, not inside it) - so hover only ever
// paints this row's own content, never a descendant's. No navigation
// affordance (border, active color, etc.) here on purpose - that's specific
// to *navigable* content, which is entirely `render_node`'s call (e.g. a
// `NavLink` in a nav sidebar already has its own active styling); `Tree`
// itself doesn't know or care whether a leaf is a link.
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
        .hover(sx().background("grey.1"))
});

// Reserves the same width the chevron would take, so a leaf row's content
// aligns with its sibling branch rows instead of starting further left.
static TREE_ROW_LEADING_SPACER_SX: StaticSx = StaticSx::new(|| {
    sx().flex_shrink("0")
        .width(SizeCss::ICON_SIZE.value(Size::Xs))
});

fn chevron_svg() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M9 18l6-6-6-6" }
        }
    }
}

struct VisibleNode {
    id: String,
    parent_id: Option<String>,
    has_children: bool,
    disabled: bool,
    label: String,
}

fn push_visible_nodes<T: TreeLabel + Clone>(
    nodes: &[TreeNode<T>],
    expanded: &HashSet<String>,
    parent_id: Option<&str>,
    out: &mut Vec<VisibleNode>,
) {
    for node in nodes {
        out.push(VisibleNode {
            id: node.id.clone(),
            parent_id: parent_id.map(str::to_string),
            has_children: node.has_children(),
            disabled: node.disabled,
            label: node.data.tree_label(),
        });
        if node.has_children() && expanded.contains(&node.id) {
            push_visible_nodes(&node.children, expanded, Some(node.id.as_str()), out);
        }
    }
}

fn visible_order<T: TreeLabel + Clone>(
    nodes: &[TreeNode<T>],
    expanded: &HashSet<String>,
) -> Vec<VisibleNode> {
    let mut out = Vec::new();
    push_visible_nodes(nodes, expanded, None, &mut out);
    out
}

fn sibling_id(order: &[VisibleNode], current: &str, offset: isize) -> Option<String> {
    let index = order.iter().position(|node| node.id == current)?;
    let target = index as isize + offset;
    if target < 0 {
        return None;
    }
    order.get(target as usize).map(|node| node.id.clone())
}

fn typeahead_match(order: &[VisibleNode], current: &str, ch: char) -> Option<String> {
    let current_index = order.iter().position(|node| node.id == current)?;
    let ch = ch.to_ascii_lowercase();
    let len = order.len();
    (1..=len).find_map(|offset| {
        let index = (current_index + offset) % len;
        let node = &order[index];
        (!node.disabled && node.label.to_ascii_lowercase().starts_with(ch)).then(|| node.id.clone())
    })
}

// Toggles `id` in/out of `expanded` - the one state mutation `Tree` performs
// on its own behalf (a disclosure toggle is structural, not content). Shared
// between a branch row's click and the keyboard Enter/Space/Left/Right
// handling. Never touches selection - that's entirely `render_node`'s (e.g.
// a `NavLink`'s) own business now.
fn toggle_expanded(
    id: &str,
    mut expanded: Signal<HashSet<String>>,
    onexpandedchange: EventHandler<HashSet<String>>,
) {
    let mut next = expanded.read().clone();
    if !next.remove(id) {
        next.insert(id.to_string());
    }
    expanded.set(next.clone());
    onexpandedchange.call(next);
}

fn focus_tree_item(root_id: &str, target_id: &str) {
    let selector = format!("[data-tree-id={target_id:?}]");
    document::eval(&format!(
        r#"await new Promise(function(r) {{ setTimeout(r, 0); }});
        var root = document.getElementById({root_id:?});
        if (!root) return;
        var target = root.querySelector({selector:?});
        if (target) target.focus();"#
    ));
}

// A leaf's own real interactive element (an `<a href>`, a `<button>`) is
// deliberately kept out of the tab order (see `TreeNodeRenderArgs::tabindex`)
// so the roving `<li>` stays the only tab stop - which means activating it
// via keyboard (Enter/Space) can't rely on the browser's native "focused
// link/button responds to Enter" behavior, since it's never actually
// focused. This triggers it the same way a real click would instead.
fn click_tree_item(root_id: &str, target_id: &str) {
    let selector = format!("[data-tree-id={target_id:?}] a, [data-tree-id={target_id:?}] button");
    document::eval(&format!(
        r#"var root = document.getElementById({root_id:?});
        if (!root) return;
        var target = root.querySelector({selector:?});
        if (target) target.click();"#
    ));
}

#[derive(Props, Clone, PartialEq)]
pub struct TreeProps<T: TreeLabel + Clone + PartialEq + 'static> {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    size: Input<ThemeAwareValue>,
    /// Overrides the gap between rows at every nesting level (indent is
    /// untouched) - e.g. `"0"` so a border on each row reads as one
    /// continuous line down a section instead of separate dashes.
    #[props(default, into)]
    gap: Input<ThemeAwareValue>,
    /// Required - WAI-ARIA's tree pattern needs an accessible name on the root.
    #[props(into)]
    aria_label: String,
    data: Vec<TreeNode<T>>,
    /// Renders each visible row's content given its data and live state.
    /// Defaults to [`default_tree_render`] (plain `tree_label()` text) -
    /// call that yourself from a custom `render_node` to fall back to it
    /// selectively, e.g. default rendering for branches, something custom
    /// (like a `NavLink`) for leaves.
    #[props(default = Callback::new(default_tree_render))]
    render_node: Callback<TreeNodeRenderArgs<T>, Element>,
    /// Which nodes start expanded - seeds `Tree`'s own internal state once.
    /// After that, expanded/collapsed is `Tree`'s own business, not the
    /// caller's - not a controlled prop.
    #[props(default)]
    default_expanded: HashSet<String>,
    /// Fires whenever the internal expanded set changes, for callers that
    /// want to observe it (e.g. to force a section open from outside) -
    /// purely a notification, not what drives rendering.
    #[props(default)]
    onexpandedchange: EventHandler<HashSet<String>>,
}

#[component]
pub fn Tree<T: TreeLabel + Clone + PartialEq + 'static>(props: TreeProps<T>) -> Element {
    let theme = use_theme();
    let root_id = use_hook(|| format!("lsx-tree-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed)));
    let active_id = use_signal(|| None::<String>);
    let expanded = use_signal(|| props.default_expanded.clone());

    let size = match props.size.as_ref() {
        Some(ThemeAwareValue::Size(size)) => *size,
        _ => theme.tree.size,
    };

    let expanded_snapshot = expanded.read().clone();
    let order = visible_order(&props.data, &expanded_snapshot);
    let resolved_active = active_id
        .read()
        .clone()
        .filter(|id| order.iter().any(|node| &node.id == id))
        .or_else(|| order.first().map(|node| node.id.clone()));

    let onexpandedchange = props.onexpandedchange;
    let data_for_keydown = props.data.clone();
    let root_id_for_keydown = root_id.clone();
    let mut active_id_for_keydown = active_id;
    let resolved_active_for_keydown = resolved_active.clone();

    let onkeydown = move |event: Event<KeyboardData>| {
        let Some(current) = resolved_active_for_keydown.clone() else {
            return;
        };
        let expanded_snapshot = expanded.read().clone();
        let order = visible_order(&data_for_keydown, &expanded_snapshot);
        let Some(node) = order.iter().find(|node| node.id == current) else {
            return;
        };

        let mut go_to = |target: Option<String>| {
            if let Some(target) = target {
                active_id_for_keydown.set(Some(target.clone()));
                focus_tree_item(&root_id_for_keydown, &target);
            }
        };

        match event.key() {
            Key::ArrowDown => {
                event.prevent_default();
                go_to(sibling_id(&order, &current, 1));
            }
            Key::ArrowUp => {
                event.prevent_default();
                go_to(sibling_id(&order, &current, -1));
            }
            Key::Home => {
                event.prevent_default();
                go_to(order.first().map(|node| node.id.clone()));
            }
            Key::End => {
                event.prevent_default();
                go_to(order.last().map(|node| node.id.clone()));
            }
            Key::ArrowRight if node.has_children => {
                event.prevent_default();
                if expanded_snapshot.contains(&current) {
                    go_to(sibling_id(&order, &current, 1));
                } else {
                    toggle_expanded(&current, expanded, onexpandedchange);
                }
            }
            Key::ArrowLeft => {
                event.prevent_default();
                if node.has_children && expanded_snapshot.contains(&current) {
                    toggle_expanded(&current, expanded, onexpandedchange);
                } else {
                    go_to(node.parent_id.clone());
                }
            }
            Key::Enter => {
                if !node.disabled {
                    event.prevent_default();
                    if node.has_children {
                        toggle_expanded(&current, expanded, onexpandedchange);
                    } else {
                        click_tree_item(&root_id_for_keydown, &current);
                    }
                }
            }
            Key::Character(ref c) if c == " " => {
                if !node.disabled {
                    event.prevent_default();
                    if node.has_children {
                        toggle_expanded(&current, expanded, onexpandedchange);
                    } else {
                        click_tree_item(&root_id_for_keydown, &current);
                    }
                }
            }
            Key::Character(ref c) => {
                if let Some(ch) = c.chars().next() {
                    event.prevent_default();
                    go_to(typeahead_match(&order, &current, ch));
                }
            }
            _ => {}
        }
    };

    let gap = props.gap.into_option();
    let root_sx = props
        .sx
        .into_option()
        .unwrap_or_default()
        .apply_if(gap.clone(), |sx, gap| sx.gap(gap));

    rsx! {
        List {
            id: "{root_id}",
            class: props.class,
            sx: root_sx,
            size: props.size.into_option(),
            "role": "tree",
            "aria-label": props.aria_label,
            onkeydown,
            attributes: props.attributes,
            for node in &props.data {
                TreeRow {
                    key: "{node.id}",
                    node: node.clone(),
                    size,
                    gap: gap.clone(),
                    expanded,
                    resolved_active: resolved_active.clone(),
                    active_id,
                    render_node: props.render_node,
                    onexpandedchange: props.onexpandedchange,
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct TreeRowProps<T: TreeLabel + Clone + PartialEq + 'static> {
    node: TreeNode<T>,
    size: Size,
    gap: Option<ThemeAwareValue>,
    expanded: Signal<HashSet<String>>,
    resolved_active: Option<String>,
    active_id: Signal<Option<String>>,
    render_node: Callback<TreeNodeRenderArgs<T>, Element>,
    onexpandedchange: EventHandler<HashSet<String>>,
}

#[component]
fn TreeRow<T: TreeLabel + Clone + PartialEq + 'static>(props: TreeRowProps<T>) -> Element {
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

    let content = props.render_node.call(TreeNodeRenderArgs {
        id: node.id.clone(),
        data: node.data.clone(),
        expanded: is_expanded,
        disabled,
        tabindex: "-1",
    });

    // Every row reserves the same leading width (chevron for a branch, a
    // matching spacer for a leaf) so labels line up across siblings instead
    // of a leaf's content starting further left than its branch siblings'.
    let leading = match is_expanded {
        Some(expanded_flag) => rsx! {
            Icon {
                variant: "transparent",
                size: "xs",
                color: "grey.6",
                sx: sx().flex_shrink("0")
                    .transition("transform 120ms ease")
                    .transform(if expanded_flag { "rotate(90deg)" } else { "rotate(0deg)" }),
                {chevron_svg()}
            }
        },
        None => rsx! {
            Box { framework_sx: &TREE_ROW_LEADING_SPACER_SX }
        },
    };

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
                {leading}
                {content}
            }
            if is_expanded == Some(true) {
                List {
                    "role": "group",
                    sx: sx().apply_if(props.gap.clone(), |sx, gap| sx.gap(gap)),
                    size: ThemeAwareValue::Size(props.size),
                    for child in &node.children {
                        TreeRow {
                            key: "{child.id}",
                            node: child.clone(),
                            size: props.size,
                            gap: props.gap.clone(),
                            expanded: props.expanded,
                            resolved_active: props.resolved_active.clone(),
                            active_id: props.active_id,
                            render_node: props.render_node,
                            onexpandedchange: props.onexpandedchange,
                        }
                    }
                }
            }
        }
    }
}
