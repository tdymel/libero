use std::{
    collections::HashSet,
    sync::atomic::{AtomicU64, Ordering},
};

use dioxus::prelude::*;

use crate::{
    components::{
        Input, List, States,
        common::{base_props, dom_api},
    },
    hooks::use_theme,
    sx::Sx,
    theme::{LIST_GAP, Size},
};

use super::{
    tree_node::{
        ErasedRenderNode, TreeLabel, TreeNode, TreeNodeErased, TreeNodeRenderArgs, erase_nodes,
    },
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

fn push_visible_nodes(
    nodes: &[TreeNodeErased],
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
            label: node.label.clone(),
        });
        if node.has_children() && expanded.contains(&node.id) {
            push_visible_nodes(&node.children, expanded, Some(node.id.as_str()), out);
        }
    }
}

fn visible_order(nodes: &[TreeNodeErased], expanded: &HashSet<String>) -> Vec<VisibleNode> {
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
    let Ok(root) = dom_api().query_selector(&format!("#{root_id}")) else {
        return;
    };
    let selector = format!("[data-tree-id={target_id:?}] a, [data-tree-id={target_id:?}] button");
    let _ = root.query_selector(&selector).and_then(|el| el.click());
}

// Scoped to `root_id` the same way `click_tree_item` is, so multiple `Tree`
// instances on one page can reuse the same node ids without colliding.
fn focus_tree_item(root_id: &str, target_id: &str) {
    let selector = format!("#{root_id} [data-tree-id={target_id:?}]");
    let _ = dom_api()
        .query_selector(&selector)
        .and_then(|el| el.focus());
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
    size: Input<Size>,
    /// Overrides the gap between rows at every nesting level, independently
    /// of `size` (indent is untouched). Off-scale values (like a flat `0`)
    /// go through `sx` instead - see the `& ul` note on `indent`.
    #[props(default, into)]
    gap: Input<Size>,
    /// Overrides the per-level indent at every nesting level, independently
    /// of `size`. To disable it entirely and compute your own offset from
    /// `render_node`'s `depth`, zero it through `sx` on both the root and
    /// its nested groups: `sx().padding_left("0").selector("& ul",
    /// sx().padding_left("0"))`.
    #[props(default, into)]
    indent: Input<Size>,
    /// Required - WAI-ARIA's tree pattern needs an accessible name on the root.
    #[props(into)]
    aria_label: String,
    data: Vec<TreeNode<T>>,
    /// Renders each visible row's content given its data and live state.
    /// Defaults to [`default_tree_render`] (plain `tree_label()` text) -
    /// call that yourself from a custom `render_node` to fall back to it
    /// selectively, e.g. default rendering for branches, something custom
    /// (like a `NavLink`) for leaves.
    #[props(default = Callback::new(super::tree_node::default_tree_render))]
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

/// Thin generic shim: converts `props.data`/`render_node` to their
/// type-erased form once, then hands off to the non-generic `TreeCore`. Only
/// this small conversion monomorphizes per `T` - the actual tree machinery
/// (`TreeCore`, `TreeRow`, keyboard nav) is compiled once regardless of how
/// many different `T`s callers use (see [[project_wasm_bundle_size_findings]]).
#[component]
pub fn Tree<T: TreeLabel + Clone + PartialEq + 'static>(props: TreeProps<T>) -> Element {
    // Cached rather than re-erased every render: keeps the `Rc<dyn Any>`
    // pointers inside `TreeNodeErased` stable across renders that don't
    // change `props.data` (e.g. an expand/collapse), which is what lets
    // `TreeNodeErased`'s `PartialEq` (pointer-based) actually skip
    // re-rendering an untouched `TreeRow` subtree.
    let mut erased_cache = use_signal(|| (props.data.clone(), erase_nodes::<T>(&props.data)));
    if erased_cache.read().0 != props.data {
        erased_cache.set((props.data.clone(), erase_nodes::<T>(&props.data)));
    }
    let erased_data = erased_cache.read().1.clone();

    let render_node = props.render_node;
    let erased_render_node = ErasedRenderNode::new(move |args| {
        let data = args
            .data
            .downcast::<T>()
            .expect("Tree: erased node data type mismatch");
        render_node.call(TreeNodeRenderArgs {
            id: args.id,
            data: (*data).clone(),
            expanded: args.expanded,
            disabled: args.disabled,
            tabindex: args.tabindex,
            depth: args.depth,
        })
    });

    rsx! {
        TreeCore {
            attributes: props.attributes,
            class: props.class,
            sx: props.sx,
            states: props.states,
            size: props.size,
            gap: props.gap,
            indent: props.indent,
            aria_label: props.aria_label,
            data: erased_data,
            render_node: erased_render_node,
            default_expanded: props.default_expanded,
            onexpandedchange: props.onexpandedchange,
        }
    }
}

base_props! {
    struct TreeCoreProps {
        #[props(default, into)]
        size: Input<Size>,
        #[props(default, into)]
        gap: Input<Size>,
        #[props(default, into)]
        indent: Input<Size>,
        #[props(into)]
        aria_label: String,
        data: Vec<TreeNodeErased>,
        render_node: ErasedRenderNode,
        #[props(default)]
        default_expanded: HashSet<String>,
        #[props(default)]
        onexpandedchange: EventHandler<HashSet<String>>,
    }
}

/// The real `Tree` - non-generic, compiled once. See [`Tree`] for why the
/// public generic component is split out from this.
#[component]
fn TreeCore(props: TreeCoreProps) -> Element {
    let theme = use_theme();
    let root_id = use_hook(|| format!("lsx-tree-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed)));
    let active_id = use_signal(|| None::<String>);
    let expanded = use_signal(|| props.default_expanded.clone());

    let size = props.size.as_ref().copied().unwrap_or(theme.tree.size);

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

    let gap = props.gap.as_ref().copied();
    let indent = props.indent.as_ref().copied();
    let root_sx = props
        .sx
        .into_option()
        .unwrap_or_default()
        .apply_if(gap, |sx, gap| sx.gap(LIST_GAP.value(gap)));

    rsx! {
        List {
            id: "{root_id}",
            class: props.class,
            sx: root_sx,
            states: props.states,
            size,
            "role": "tree",
            "aria-label": props.aria_label,
            onkeydown,
            attributes: props.attributes,
            for node in &props.data {
                TreeRow {
                    key: "{node.id}",
                    node: node.clone(),
                    size,
                    gap,
                    indent,
                    depth: 0,
                    expanded,
                    resolved_active: resolved_active.clone(),
                    active_id,
                    render_node: props.render_node.clone(),
                    onexpandedchange: props.onexpandedchange,
                }
            }
        }
    }
}
