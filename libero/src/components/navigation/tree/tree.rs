use std::{cell::RefCell, collections::HashSet, rc::Rc};

use dioxus::prelude::*;

use crate::{
    components::{
        Input, List,
        common::{base_props, css_string, has_shortcut_modifier},
    },
    hooks::{
        ElementHandle, TYPEAHEAD_RESET, typeahead_match, use_element, use_theme, use_typeahead,
    },
    platform::ElementApi,
    theme::Size,
    utils::warn,
};

use super::{
    tree_node::{
        ErasedRenderNode, TreeNode, TreeNodeErased, TreeNodeRenderArgs, TreeValue, erase_nodes,
    },
    tree_row::{TreeRow, child_active},
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

/// Where the first visible row with `id` sits, as child indices from the
/// roots down. By position, not by id: a repeated id would otherwise make every
/// row that carries it the tab stop.
fn visible_path(
    nodes: &[TreeNodeErased],
    expanded: &HashSet<String>,
    id: &str,
) -> Option<Vec<usize>> {
    nodes.iter().enumerate().find_map(|(index, node)| {
        if node.id == id {
            return Some(vec![index]);
        }
        if !(node.has_children() && expanded.contains(&node.id)) {
            return None;
        }
        let mut path = visible_path(&node.children, expanded, id)?;
        path.insert(0, index);
        Some(path)
    })
}

/// The first id that appears twice anywhere in the tree.
fn repeated_id(nodes: &[TreeNodeErased]) -> Option<&str> {
    fn walk<'a>(nodes: &'a [TreeNodeErased], seen: &mut HashSet<&'a str>) -> Option<&'a str> {
        nodes
            .iter()
            .find_map(|node| match seen.insert(node.id.as_str()) {
                true => walk(&node.children, seen),
                false => Some(node.id.as_str()),
            })
    }
    walk(nodes, &mut HashSet::new())
}

/// The ids from a root down to `id`, both ends included.
fn path_to<'a>(nodes: &'a [TreeNodeErased], id: &str) -> Option<Vec<&'a str>> {
    nodes.iter().find_map(|node| {
        if node.id == id {
            return Some(vec![node.id.as_str()]);
        }
        let mut path = path_to(&node.children, id)?;
        path.insert(0, node.id.as_str());
        Some(path)
    })
}

fn sibling_id(order: &[VisibleNode], current: &str, offset: isize) -> Option<String> {
    let index = order.iter().position(|node| node.id == current)?;
    let target = index as isize + offset;
    if target < 0 {
        return None;
    }
    order.get(target as usize).map(|node| node.id.to_string())
}

/// What ArrowRight or ArrowLeft does on `node`.
#[derive(Debug, PartialEq)]
enum Horizontal {
    Toggle,
    Go(Option<String>),
    /// Nothing moves, but the key is still the tree's.
    Stay,
}

/// APG's tree arrows, with the rule every navigation widget shares: a disabled
/// node is walked through like any other, but no key opens or closes it.
/// `None` for ArrowRight on a leaf, which the tree leaves to the page.
fn horizontal(
    order: &[VisibleNode],
    node: &VisibleNode,
    is_expanded: bool,
    forward: bool,
) -> Option<Horizontal> {
    let toggles = node.has_children && !node.disabled;
    Some(match forward {
        true if !node.has_children => return None,
        true if is_expanded => Horizontal::Go(sibling_id(order, node.id, 1)),
        true if toggles => Horizontal::Toggle,
        true => Horizontal::Stay,
        false if is_expanded && toggles => Horizontal::Toggle,
        false => Horizontal::Go(node.parent_id.map(str::to_string)),
    })
}

/// The row a typed `query` lands on, through the library's one typeahead
/// ([`typeahead_match`]): one character cycles from the row after `current`,
/// a longer query narrows and stays on a row that still matches. Disabled rows
/// are skipped.
fn typeahead_target(order: &[VisibleNode], current: &str, query: &str) -> Option<String> {
    let current = order.iter().position(|node| node.id == current)?;
    typeahead_match(order.len(), Some(current), query, |index| {
        let node = &order[index];
        (!node.disabled).then_some(node.label)
    })
    .map(|index| order[index].id.to_string())
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
fn click_tree_item(root: &ElementHandle, target_id: &str) {
    let id = css_string(target_id);
    let selector = format!("[data-tree-id={id}] a, [data-tree-id={id}] button");
    let _ = root.query_selector(&selector).and_then(|el| el.click());
}

// Keyboard nav only applies while the active row itself holds focus. Anything
// else (an input or editable inside a row) owns its own keystrokes.
fn tree_item_focused(root: &ElementHandle, target_id: &str) -> bool {
    let selector = format!("[data-tree-id={}]", css_string(target_id));
    root.query_selector(&selector)
        .is_ok_and(|el| el.is_focused())
}

// Scoped to this tree's own root, so two `Tree`s can reuse node ids without
// colliding.
fn focus_tree_item(root: &ElementHandle, target_id: &str) {
    let selector = format!("[data-tree-id={}]", css_string(target_id));
    let _ = root.query_selector(&selector).and_then(|el| el.focus());
}

base_props! {
    pub struct TreeProps<T: TreeValue> {
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
        /// The id of the node where the user is - a nav's current page. Tab
        /// into the tree lands on it rather than on the first row, until the
        /// arrow keys move on; when it changes, the tab stop follows it. Inside
        /// a collapsed branch, the tab stop goes to the branch.
        #[props(default, into)]
        current: Option<String>,
        /// Notification only - it doesn't drive rendering.
        #[props(default)]
        onexpandedchange: Option<EventHandler<HashSet<String>>>,
    }
}

/// Generic shim: erases `props.data`/`render_node` once, then hands off to the
/// non-generic `TreeCore`. Only this conversion monomorphizes per `T`; the
/// tree machinery compiles once.
///
/// Panics if a `TreeRow` hands back data that isn't a `T` - the erasure trades
/// that compile-time guarantee for a runtime check.
#[component]
pub fn Tree<T: TreeValue>(props: TreeProps<T>) -> Element {
    // Cached so the `Rc<dyn Any>` pointers stay stable across renders that
    // don't change `props.data`, which is what lets `TreeNodeErased`'s
    // pointer equality skip an untouched subtree. Not a signal: it derives
    // from `props.data`, and writing one here forces a second render pass.
    let cache = use_hook(|| {
        let erased = erase_nodes::<T>(&props.data);
        warn_on_repeated_id(&erased);
        Rc::new(RefCell::new((props.data.clone(), erased)))
    });
    let erased_data = {
        let mut cache = cache.borrow_mut();
        if cache.0 != props.data {
            let erased = erase_nodes::<T>(&props.data);
            warn_on_repeated_id(&erased);
            *cache = (props.data.clone(), erased);
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
            current: props.current,
            onexpandedchange: props.onexpandedchange,
        }
    }
}

/// Once per new `data`, not per render: the tree re-renders on every key.
fn warn_on_repeated_id(nodes: &[TreeNodeErased]) {
    if !cfg!(debug_assertions) {
        return;
    }
    if let Some(id) = repeated_id(nodes) {
        warn(&format!(
            "Tree: the id \"{id}\" is used by more than one node. Ids must be unique; \
             only the first of them can take the tab stop."
        ));
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
        current: Option<String>,
        #[props(default)]
        onexpandedchange: Option<EventHandler<HashSet<String>>>,
    }
}

/// The real `Tree`, non-generic and compiled once. See [`Tree`] for why.
#[component]
fn TreeCore(props: TreeCoreProps) -> Element {
    let theme = use_theme();
    let root = use_element();
    let mut active_id = use_signal(|| None::<String>);
    let expanded = use_signal(|| props.default_expanded.clone());

    let size = props.size.copied_or(theme.tree.size);

    let order = visible_order(&props.data, &expanded.read());
    let visible = |id: &String| order.iter().any(|node| node.id == id);
    let resolved_active = active_id
        .read()
        .clone()
        .filter(visible)
        .or_else(|| {
            // Inside a collapsed branch, the branch that hides it.
            let path = path_to(&props.data, props.current.as_deref()?)?;
            path.into_iter()
                .rev()
                .map(str::to_string)
                .find(|id| visible(id))
        })
        .or_else(|| order.first().map(|node| node.id.to_string()));
    // The first row with that id, as `Tabs` resolves its value to the first
    // tab that matches.
    let active_path = resolved_active
        .as_deref()
        .and_then(|id| visible_path(&props.data, &expanded.read(), id));

    // A new `current` (the route changed) wins over wherever the arrow keys
    // left the tab stop. `peek`, so the mount run writes nothing.
    let current = props.current.clone();
    use_effect(use_reactive!(|current| {
        let _ = current;
        if active_id.peek().is_some() {
            active_id.set(None);
        }
    }));

    let onexpandedchange = props.onexpandedchange;
    let data_for_keydown = props.data.clone();
    let mut active_id_for_keydown = active_id;
    let resolved_active_for_keydown = resolved_active.clone();

    let typeahead = use_typeahead(TYPEAHEAD_RESET);

    let onkeydown = move |event: Event<KeyboardData>| {
        let Some(current) = resolved_active_for_keydown.clone() else {
            return;
        };
        if !tree_item_focused(&root, &current) {
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
                focus_tree_item(&root, &target);
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
            key @ (Key::ArrowRight | Key::ArrowLeft) => {
                let forward = key == Key::ArrowRight;
                let Some(step) = horizontal(&order, node, is_expanded, forward) else {
                    return;
                };
                event.prevent_default();
                match step {
                    Horizontal::Toggle => toggle_expanded(&current, expanded, onexpandedchange),
                    Horizontal::Go(target) => go_to(target),
                    Horizontal::Stay => {}
                }
            }
            Key::Enter => {
                if !node.disabled {
                    event.prevent_default();
                    if node.has_children {
                        toggle_expanded(&current, expanded, onexpandedchange);
                    } else {
                        click_tree_item(&root, &current);
                    }
                }
            }
            Key::Character(ref c) if c == " " && has_shortcut_modifier(&event) => {}
            // A space mid-query is part of "new folder", not an activation.
            Key::Character(ref c) if c == " " && typeahead.is_typing() => {
                event.prevent_default();
                if let Some(target) = typeahead_target(&order, &current, &typeahead.push(' ')) {
                    go_to(Some(target));
                }
            }
            Key::Character(ref c) if c == " " => {
                if !node.disabled {
                    event.prevent_default();
                    if node.has_children {
                        toggle_expanded(&current, expanded, onexpandedchange);
                    } else {
                        click_tree_item(&root, &current);
                    }
                }
            }
            Key::Character(ref c) if !has_shortcut_modifier(&event) => {
                if let Some(target) = c
                    .chars()
                    .next()
                    .and_then(|ch| typeahead_target(&order, &current, &typeahead.push(ch)))
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
            class: props.class,
            onmounted: root.mount(),
            sx: root_sx,
            states: props.states,
            size,
            "role": "tree",
            "aria-label": props.aria_label,
            onkeydown,
            attributes: props.attributes,
            for (index , node) in props.data.iter().enumerate() {
                TreeRow {
                    key: "{node.id}",
                    node: node.clone(),
                    size,
                    depth: 0,
                    expanded,
                    active: child_active(active_path.as_deref(), index),
                    active_id,
                    render_node: props.render_node.clone(),
                    onexpandedchange: props.onexpandedchange,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(ids: &[(&'static str, &[&'static str])]) -> Vec<TreeNodeErased> {
        let nodes = ids
            .iter()
            .map(|(id, children)| {
                TreeNode::new(*id, id.to_string()).children(
                    children
                        .iter()
                        .map(|child| TreeNode::new(*child, child.to_string()))
                        .collect(),
                )
            })
            .collect::<Vec<_>>();
        erase_nodes::<String>(&nodes)
    }

    #[test]
    fn a_repeated_id_is_found_across_branches() {
        assert_eq!(repeated_id(&tree(&[("a", &["x"]), ("b", &["y"])])), None);
        assert_eq!(
            repeated_id(&tree(&[("a", &["x"]), ("b", &["x"])])),
            Some("x")
        );
        assert_eq!(repeated_id(&tree(&[("a", &[]), ("a", &[])])), Some("a"));
    }

    #[test]
    fn a_repeated_id_warns() {
        crate::utils::take_warnings();
        warn_on_repeated_id(&tree(&[("a", &["x"]), ("b", &["x"])]));
        let warnings = crate::utils::take_warnings();
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].contains(r#""x""#), "{warnings:?}");
        warn_on_repeated_id(&tree(&[("a", &["x"]), ("b", &["y"])]));
        assert!(crate::utils::take_warnings().is_empty());
    }

    /// Only an open branch counts, and the first match in document order wins.
    #[test]
    fn the_visible_path_finds_the_first_open_match() {
        let nodes = tree(&[("a", &["x"]), ("b", &["x"])]);
        let open = |ids: &[&str]| ids.iter().map(|id| id.to_string()).collect::<HashSet<_>>();
        assert_eq!(
            visible_path(&nodes, &open(&["a", "b"]), "x"),
            Some(vec![0, 0])
        );
        assert_eq!(visible_path(&nodes, &open(&["b"]), "x"), Some(vec![1, 0]));
        assert_eq!(visible_path(&nodes, &open(&[]), "x"), None);
    }

    fn node<'a>(id: &'a str, label: &'a str, disabled: bool) -> VisibleNode<'a> {
        VisibleNode {
            id,
            parent_id: None,
            has_children: false,
            disabled,
            label,
        }
    }

    fn order() -> Vec<VisibleNode<'static>> {
        vec![
            node("a", "Src", false),
            node("b", "Styles", false),
            node("c", "Scripts", true),
            node("d", "Docs", false),
            node("e", "Setup", false),
        ]
    }

    /// What `Tree` did before the shared buffer: one character, from the row
    /// after the current one, wrapping, skipping disabled rows.
    #[test]
    fn one_character_behaves_as_it_always_did() {
        let order = order();
        assert_eq!(typeahead_target(&order, "a", "s").as_deref(), Some("b"));
        // "Scripts" is disabled.
        assert_eq!(typeahead_target(&order, "b", "s").as_deref(), Some("e"));
        assert_eq!(typeahead_target(&order, "e", "s").as_deref(), Some("a"));
        assert_eq!(typeahead_target(&order, "a", "D").as_deref(), Some("d"));
        assert_eq!(typeahead_target(&order, "a", "x"), None);
    }

    #[test]
    fn a_longer_query_narrows() {
        let order = order();
        assert_eq!(typeahead_target(&order, "a", "se").as_deref(), Some("e"));
        assert_eq!(typeahead_target(&order, "b", "st").as_deref(), Some("b"));
        assert_eq!(typeahead_target(&order, "a", "sc"), None);
    }

    fn branch<'a>(id: &'a str, parent_id: Option<&'a str>, disabled: bool) -> VisibleNode<'a> {
        VisibleNode {
            id,
            parent_id,
            has_children: true,
            disabled,
            label: id,
        }
    }

    #[test]
    fn the_arrows_open_and_close_an_enabled_branch() {
        let order = [branch("a", Some("root"), false), node("a1", "Child", false)];
        let a = &order[0];
        assert_eq!(horizontal(&order, a, false, true), Some(Horizontal::Toggle));
        assert_eq!(horizontal(&order, a, true, false), Some(Horizontal::Toggle));
        assert_eq!(
            horizontal(&order, a, true, true),
            Some(Horizontal::Go(Some("a1".into())))
        );
        assert_eq!(
            horizontal(&order, a, false, false),
            Some(Horizontal::Go(Some("root".into())))
        );
    }

    /// Review 3 A5: ArrowRight and ArrowLeft used to open and close a disabled
    /// branch that a click, Enter and Space all refused.
    #[test]
    fn the_arrows_walk_through_a_disabled_branch_without_toggling_it() {
        let order = [branch("a", Some("root"), true), node("a1", "Child", false)];
        let a = &order[0];
        assert_eq!(horizontal(&order, a, false, true), Some(Horizontal::Stay));
        assert_eq!(
            horizontal(&order, a, true, true),
            Some(Horizontal::Go(Some("a1".into())))
        );
        assert_eq!(
            horizontal(&order, a, true, false),
            Some(Horizontal::Go(Some("root".into())))
        );
    }

    #[test]
    fn arrow_right_on_a_leaf_is_left_to_the_page() {
        let leaf = node("a", "Leaf", false);
        assert_eq!(horizontal(&[], &leaf, false, true), None);
    }

    #[test]
    fn an_unknown_current_row_matches_nothing() {
        assert_eq!(typeahead_target(&order(), "zzz", "s"), None);
    }
}
