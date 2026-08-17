use std::{
    collections::HashSet,
    sync::atomic::{AtomicU64, Ordering},
};

use dioxus::{document, prelude::*};

use crate::{
    components::{Input, List, States},
    hooks::{FocusRegistry, use_theme},
    sx::{Sx, ThemeAwareValue},
};

use super::{
    tree_node::{TreeLabel, TreeNode, TreeNodeRenderArgs, default_tree_render},
    tree_row::TreeRow,
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

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
// between a branch row's click (in `tree_row`) and the keyboard
// Enter/Space/Left/Right handling below. Never touches selection - that's
// entirely `render_node`'s (e.g. a `NavLink`'s) own business now.
pub(super) fn toggle_expanded(
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
    states: Input<States>,
    #[props(default, into)]
    size: Input<ThemeAwareValue>,
    /// Overrides the gap between rows at every nesting level (indent is
    /// untouched) - e.g. `"0"` so a border on each row reads as one
    /// continuous line down a section instead of separate dashes.
    #[props(default, into)]
    gap: Input<ThemeAwareValue>,
    /// Overrides the per-level indent at every nesting level - e.g. `"0"`
    /// to disable it entirely and compute your own from `render_node`'s
    /// `depth` instead, when the indent needs to interact with the content
    /// itself (like a border lining up with an ancestor's chevron column).
    #[props(default, into)]
    indent: Input<ThemeAwareValue>,
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
    let focus_registry = use_context_provider(FocusRegistry::<String>::new);

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
                focus_registry.focus(&target);
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
    let indent = props.indent.into_option();
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
            states: props.states,
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
                    indent: indent.clone(),
                    depth: 0,
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
