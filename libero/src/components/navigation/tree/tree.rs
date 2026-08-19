use std::{cell::RefCell, collections::HashSet, rc::Rc};

use dioxus::prelude::*;

use crate::{
    components::{
        Input, List, States,
        common::{base_props, dom_api},
    },
    hooks::{use_id, use_theme},
    sx::Sx,
    theme::{LIST_GAP, Size},
};

use super::{
    tree_node::{
        ErasedRenderNode, TreeLabel, TreeNode, TreeNodeErased, TreeNodeRenderArgs, erase_nodes,
    },
    tree_row::TreeRow,
};

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

// The one state mutation `Tree` makes on its own behalf, shared between a
// branch row's click and the keyboard handling. Never touches selection -
// that belongs to `render_node`.
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

// A leaf's real link/button is kept out of the tab order (see
// `TreeNodeRenderArgs::tabindex`), so it is never focused and Enter never
// reaches it natively. This triggers it the way a click would.
fn click_tree_item(root_id: &str, target_id: &str) {
    let Ok(root) = dom_api().query_selector(&format!("#{root_id}")) else {
        return;
    };
    let selector = format!("[data-tree-id={target_id:?}] a, [data-tree-id={target_id:?}] button");
    let _ = root.query_selector(&selector).and_then(|el| el.click());
}

// Scoped to `root_id`, so two `Tree`s can reuse node ids without colliding.
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
    /// Row gap at every level, independent of `size`. Off-scale values go
    /// through `sx` - see the `& ul` note on `indent`.
    #[props(default, into)]
    gap: Input<Size>,
    /// Per-level indent, independent of `size`. To offset from `render_node`'s
    /// `depth` yourself, zero it on both the root and its nested groups:
    /// `sx().padding_left("0").selector("& ul", sx().padding_left("0"))`.
    #[props(default, into)]
    indent: Input<Size>,
    /// Required by WAI-ARIA's tree pattern.
    #[props(into)]
    aria_label: String,
    data: Vec<TreeNode<T>>,
    /// Each visible row's content. Defaults to [`default_tree_render`], which
    /// a custom `render_node` can also call to fall back selectively - say,
    /// default branches and `NavLink` leaves.
    #[props(default = Callback::new(super::tree_node::default_tree_render))]
    render_node: Callback<TreeNodeRenderArgs<T>, Element>,
    /// Seeds `Tree`'s internal state once. Not a controlled prop.
    #[props(default)]
    default_expanded: HashSet<String>,
    /// Notification only - it doesn't drive rendering.
    #[props(default)]
    onexpandedchange: EventHandler<HashSet<String>>,
}

/// Generic shim: erases `props.data`/`render_node` once, then hands off to the
/// non-generic `TreeCore`. Only this conversion monomorphizes per `T`; the
/// tree machinery compiles once.
///
/// Panics if a `TreeRow` hands back data that isn't a `T` - the erasure trades
/// that compile-time guarantee for a runtime check.
#[component]
pub fn Tree<T: TreeLabel + Clone + PartialEq + 'static>(props: TreeProps<T>) -> Element {
    // Cached so the `Rc<dyn Any>` pointers stay stable across renders that
    // don't change `props.data`, which is what lets `TreeNodeErased`'s
    // pointer equality skip an untouched subtree. Not a signal: it derives
    // from `props.data`, and writing one here forces a second render pass.
    let cache = use_hook(|| {
        Rc::new(RefCell::new((
            props.data.clone(),
            erase_nodes::<T>(&props.data),
        )))
    });
    let erased_data = {
        let mut cache = cache.borrow_mut();
        if cache.0 != props.data {
            *cache = (props.data.clone(), erase_nodes::<T>(&props.data));
        }
        cache.1.clone()
    };

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

/// The real `Tree`, non-generic and compiled once. See [`Tree`] for why.
#[component]
fn TreeCore(props: TreeCoreProps) -> Element {
    let theme = use_theme();
    let root_id = use_id();
    let active_id = use_signal(|| None::<String>);
    let expanded = use_signal(|| props.default_expanded.clone());

    let size = props.size.copied_or(theme.tree.size);

    let expanded_snapshot = expanded.read().clone();
    let order = visible_order(&props.data, &expanded_snapshot);
    let resolved_active = active_id
        .read()
        .clone()
        .filter(|id| order.iter().any(|node| &node.id == id))
        .or_else(|| order.first().map(|node| node.id.clone()));

    let onexpandedchange = props.onexpandedchange;
    let data_for_keydown = props.data.clone();
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
                focus_tree_item(&root_id(), &target);
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
                        click_tree_item(&root_id(), &current);
                    }
                }
            }
            Key::Character(ref c) if c == " " => {
                if !node.disabled {
                    event.prevent_default();
                    if node.has_children {
                        toggle_expanded(&current, expanded, onexpandedchange);
                    } else {
                        click_tree_item(&root_id(), &current);
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
