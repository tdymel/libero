use std::{cell::RefCell, collections::HashSet, rc::Rc};

use dioxus::prelude::*;

use crate::{
    components::{
        Input, List, States,
        common::{base_props, css_string, dom_api},
    },
    hooks::{use_root_id, use_theme},
    sx::Sx,
    theme::Size,
};

use super::{
    tree_node::{
        ErasedRenderNode, TreeLabel, TreeNode, TreeNodeErased, TreeNodeRenderArgs, erase_nodes,
    },
    tree_row::TreeRow,
};

/// Borrows from the `data` it walks: it is rebuilt on every render and every
/// keystroke, so owning the ids and labels meant two `String`s per node each
/// time.
struct VisibleNode<'a> {
    id: &'a str,
    parent_id: Option<&'a str>,
    has_children: bool,
    disabled: bool,
    label: &'a str,
}

fn push_visible_nodes<'a>(
    nodes: &'a [TreeNodeErased],
    expanded: &HashSet<String>,
    parent_id: Option<&'a str>,
    out: &mut Vec<VisibleNode<'a>>,
) {
    for node in nodes {
        out.push(VisibleNode {
            id: &node.id,
            parent_id,
            has_children: node.has_children(),
            disabled: node.disabled,
            label: &node.label,
        });
        if node.has_children() && expanded.contains(&node.id) {
            push_visible_nodes(&node.children, expanded, Some(node.id.as_str()), out);
        }
    }
}

fn visible_order<'a>(
    nodes: &'a [TreeNodeErased],
    expanded: &HashSet<String>,
) -> Vec<VisibleNode<'a>> {
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
    order.get(target as usize).map(|node| node.id.to_string())
}

fn typeahead_match(order: &[VisibleNode], current: &str, ch: char) -> Option<String> {
    let current_index = order.iter().position(|node| node.id == current)?;
    let ch = ch.to_ascii_lowercase();
    let len = order.len();
    (1..=len).find_map(|offset| {
        let index = (current_index + offset) % len;
        let node = &order[index];
        // First char only, so no lowercased copy of every label per keystroke.
        let first = node.label.chars().next().map(|c| c.to_ascii_lowercase());
        (!node.disabled && first == Some(ch)).then(|| node.id.to_string())
    })
}

// The one state mutation `Tree` makes on its own behalf, shared between a
// branch row's click and the keyboard handling. Never touches selection -
// that belongs to `render_node`.
pub(super) fn toggle_expanded(
    id: &str,
    mut expanded: Signal<HashSet<String>>,
    onexpandedchange: Option<EventHandler<HashSet<String>>>,
) {
    let mut next = expanded.read().clone();
    if !next.remove(id) {
        next.insert(id.to_string());
    }
    expanded.set(next.clone());
    if let Some(onexpandedchange) = onexpandedchange {
        onexpandedchange.call(next);
    }
}

// A leaf's real link/button is kept out of the tab order (see
// `TreeNodeRenderArgs::tabindex`), so it is never focused and Enter never
// reaches it natively. This triggers it the way a click would.
fn click_tree_item(root_id: &str, target_id: &str) {
    let Ok(root) = dom_api().query_selector(&format!("#{root_id}")) else {
        return;
    };
    let id = css_string(target_id);
    let selector = format!("[data-tree-id={id}] a, [data-tree-id={id}] button");
    let _ = root.query_selector(&selector).and_then(|el| el.click());
}

// Shift is part of ordinary typing; the rest mark a browser or OS shortcut
// that must not be mistaken for typeahead.
fn has_shortcut_modifier(event: &Event<KeyboardData>) -> bool {
    let modifiers = event.modifiers();
    modifiers.ctrl() || modifiers.alt() || modifiers.meta()
}

// Keyboard nav only applies while the active row itself holds focus. Anything
// else (an input or editable inside a row) owns its own keystrokes.
fn tree_item_focused(root_id: &str, target_id: &str) -> bool {
    let selector = format!("#{root_id} [data-tree-id={}]", css_string(target_id));
    dom_api()
        .query_selector(&selector)
        .is_ok_and(|el| el.is_focused())
}

// Scoped to `root_id`, so two `Tree`s can reuse node ids without colliding.
fn focus_tree_item(root_id: &str, target_id: &str) {
    let selector = format!("#{root_id} [data-tree-id={}]", css_string(target_id));
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
    /// Row gap and per-level indent together - `List`'s scale, since `Tree`
    /// renders through it. Off-scale, or the two apart, goes through `sx`:
    /// `sx().gap("4px").selector("& ul", sx().padding_left("24px"))`.
    #[props(default, into)]
    size: Input<Size>,
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
    onexpandedchange: Option<EventHandler<HashSet<String>>>,
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
        #[props(into)]
        aria_label: String,
        data: Vec<TreeNodeErased>,
        render_node: ErasedRenderNode,
        #[props(default)]
        default_expanded: HashSet<String>,
        #[props(default)]
        onexpandedchange: Option<EventHandler<HashSet<String>>>,
    }
}

/// The real `Tree`, non-generic and compiled once. See [`Tree`] for why.
#[component]
fn TreeCore(props: TreeCoreProps) -> Element {
    let theme = use_theme();
    let root_id = use_root_id(&props.attributes);
    let active_id = use_signal(|| None::<String>);
    let expanded = use_signal(|| props.default_expanded.clone());

    let size = props.size.copied_or(theme.tree.size);

    let order = visible_order(&props.data, &expanded.read());
    let resolved_active = active_id
        .read()
        .clone()
        .filter(|id| order.iter().any(|node| node.id == id))
        .or_else(|| order.first().map(|node| node.id.to_string()));

    let onexpandedchange = props.onexpandedchange;
    let data_for_keydown = props.data.clone();
    let mut active_id_for_keydown = active_id;
    let resolved_active_for_keydown = resolved_active.clone();

    let onkeydown = move |event: Event<KeyboardData>| {
        let Some(current) = resolved_active_for_keydown.clone() else {
            return;
        };
        if !tree_item_focused(&root_id(), &current) {
            return;
        }
        let order = visible_order(&data_for_keydown, &expanded.read());
        let Some(node) = order.iter().find(|node| node.id == current) else {
            return;
        };
        // A bool, not the set: `toggle_expanded` writes to `expanded` below.
        let is_expanded = expanded.read().contains(&current);

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
                go_to(order.first().map(|node| node.id.to_string()));
            }
            Key::End => {
                event.prevent_default();
                go_to(order.last().map(|node| node.id.to_string()));
            }
            Key::ArrowRight if node.has_children => {
                event.prevent_default();
                if is_expanded {
                    go_to(sibling_id(&order, &current, 1));
                } else {
                    toggle_expanded(&current, expanded, onexpandedchange);
                }
            }
            Key::ArrowLeft => {
                event.prevent_default();
                if node.has_children && is_expanded {
                    toggle_expanded(&current, expanded, onexpandedchange);
                } else {
                    go_to(node.parent_id.map(str::to_string));
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
            Key::Character(ref c) if c == " " && has_shortcut_modifier(&event) => {}
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
            Key::Character(ref c) if !has_shortcut_modifier(&event) => {
                if let Some(target) = c
                    .chars()
                    .next()
                    .and_then(|ch| typeahead_match(&order, &current, ch))
                {
                    event.prevent_default();
                    go_to(Some(target));
                }
            }
            _ => {}
        }
    };

    let root_sx = props.sx.into_option().unwrap_or_default();

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
