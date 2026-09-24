use std::{cell::RefCell, collections::HashSet, rc::Rc};

use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            Input, base_props, css_string, has_shortcut_modifier, parts_enum, parts_under_sx,
        },
        data_display::List,
    },
    hooks::{
        ElementHandle, TYPEAHEAD_RESET, typeahead_match, use_element, use_theme, use_typeahead,
    },
    platform::{ElementApi, logical_key, when_free},
    theme::Size,
    utils::warn,
};

use super::{
    tree_node::{
        ErasedRenderNode, TreeNode, TreeNodeErased, TreeNodeRenderArgs, TreeValue, erase_nodes,
    },
    tree_row::{TreeRow, child_active},
};

/// Borrows from `data`: rebuilt on every keystroke, owning meant two `String`s per node.
struct VisibleNode<'a> {
    id: &'a str,
    parent_id: Option<&'a str>,
    has_children: bool,
    disabled: bool,
    label: &'a str,
}

/// `disabled` cascades to the nodes under a disabled branch.
fn push_visible_nodes<'a>(
    nodes: &'a [TreeNodeErased],
    expanded: &HashSet<String>,
    parent: Option<(&'a str, bool)>,
    out: &mut Vec<VisibleNode<'a>>,
) {
    let (parent_id, inherited) = (parent.map(|(id, _)| id), parent.is_some_and(|(_, off)| off));
    for node in nodes {
        let disabled = inherited || node.disabled;
        out.push(VisibleNode {
            id: &node.id,
            parent_id,
            has_children: node.has_children(),
            disabled,
            label: &node.label,
        });
        if node.has_children() && expanded.contains(&node.id) {
            push_visible_nodes(&node.children, expanded, Some((&node.id, disabled)), out);
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

/// The first visible row with `id`, as child indices; by position, so a repeated
/// id makes only one row the tab stop.
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

/// APG's tree arrows; a disabled node is walked through but never toggled.
/// `None` for ArrowRight on a leaf, left to the page.
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

/// The row a typed `query` lands on via [`typeahead_match`], skipping disabled rows.
fn typeahead_target(order: &[VisibleNode], current: &str, query: &str) -> Option<String> {
    let current = order.iter().position(|node| node.id == current)?;
    typeahead_match(order.len(), Some(current), query, |index| {
        let node = &order[index];
        (!node.disabled).then_some(node.label)
    })
    .map(|index| order[index].id.to_string())
}

/// The open branches, shared by every row. Controlled, a change only asks via `onchange`.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Expansion {
    pub open: Signal<HashSet<String>>,
    controlled: bool,
    onchange: Option<EventHandler<HashSet<String>>>,
}

impl Expansion {
    // Shared by a branch row's click and the keyboard; selection is `render_node`'s.
    pub fn toggle(self, id: &str) {
        let mut next = self.open.read().clone();
        if !next.remove(id) {
            next.insert(id.to_string());
        }
        self.set(next);
    }

    fn set(mut self, next: HashSet<String>) {
        if !self.controlled {
            self.open.set(next.clone());
        }
        if let Some(onchange) = self.onchange {
            onchange.call(next);
        }
    }
}

/// The closed, enabled branches beside `node`, `node` included: APG's `*`.
fn closed_siblings<'a>(
    order: &[VisibleNode<'a>],
    node: &VisibleNode,
    expanded: &HashSet<String>,
) -> Vec<&'a str> {
    order
        .iter()
        .filter(|other| other.parent_id == node.parent_id && other.has_children)
        .filter(|other| !other.disabled && !expanded.contains(other.id))
        .map(|other| other.id)
        .collect()
}

/// One write and one `onexpandedchange` for the lot, not one per branch.
fn expand_siblings(order: &[VisibleNode], node: &VisibleNode, expansion: Expansion) {
    let closed = closed_siblings(order, node, &expansion.open.read());
    if closed.is_empty() {
        return;
    }
    let mut next = expansion.open.read().clone();
    next.extend(closed.into_iter().map(str::to_string));
    expansion.set(next);
}

// A leaf's link or button is out of the tab order, so Enter clicks it from here.
fn click_tree_item(root: &ElementHandle, target_id: &str) {
    let id = css_string(target_id);
    let selector = format!("[data-tree-id={id}] a, [data-tree-id={id}] button");
    let _ = root.query_selector(&selector).and_then(|el| el.click());
}

// Only while the row itself holds focus; an input inside a row keeps its keys.
fn tree_item_focused(root: &ElementHandle, target_id: &str) -> bool {
    let selector = format!("[data-tree-id={}]", css_string(target_id));
    root.query_selector(&selector)
        .is_ok_and(|el| el.is_focused())
}

// The row's own content only (`> div`), not a nested row's in its group.
fn row_control_focused(root: &ElementHandle, target_id: &str) -> bool {
    let id = css_string(target_id);
    let selector = format!("[data-tree-id={id}] > div :is(a, button)");
    root.query_selector(&selector)
        .is_ok_and(|el| el.is_focused())
}

// Scoped to this tree's root, so two `Tree`s can share node ids.
fn focus_tree_item(root: &ElementHandle, target_id: &str) {
    let selector = format!("[data-tree-id={}]", css_string(target_id));
    let _ = root.query_selector(&selector).and_then(|el| el.focus());
}

parts_enum! {
    /// [`Tree`]'s inner parts, for its `parts` prop. Descendant selectors: a
    /// `Tree` nested in a row's content matches too.
    pub enum TreePart {
        /// A `treeitem` `<li>`: the row and its open subtree.
        Row = "row" => "& [data-slot='row']",
        /// The clickable line around `render_node`'s content.
        Content = "content" => "& [data-slot='row'] > [data-slot='content']",
        /// An open branch's child list.
        Group = "group" => "& [data-slot='row'] > [data-slot='group']",
    }
}

base_props! {
    parts(TreePart);
    pub struct TreeProps<T: TreeValue> {
        /// Row gap and per-level indent, on `List`'s scale; set them apart through `sx`.
        #[props(default, into)]
        size: Input<Size>,
        /// A line down each open branch, the `current` row's segment marked.
        #[props(default)]
        guides: Option<bool>,
        /// Required by WAI-ARIA's tree pattern.
        #[props(into)]
        aria_label: String,
        data: Vec<TreeNode<T>>,
        /// Each row's content. Defaults to [`default_tree_render`], which a custom one can call.
        #[props(default = Callback::new(super::tree_node::default_tree_render))]
        render_node: Callback<TreeNodeRenderArgs<T>, Element>,
        /// Seeds the open branches once. Ignored when `expanded` is set.
        #[props(default)]
        default_expanded: HashSet<String>,
        /// The open branches' ids; set, the tree is controlled.
        #[props(default)]
        expanded: Option<HashSet<String>>,
        /// The node where the user is, such as a nav's current page: the tab stop and `aria-current`.
        #[props(default, into)]
        current: Option<String>,
        /// The whole new set of open ids.
        #[props(default)]
        onexpandedchange: Option<EventHandler<HashSet<String>>>,
    }
}

/// A WAI-ARIA tree of expandable branches.
///
/// ```
/// # use std::collections::HashSet;
/// # use dioxus::prelude::*;
/// # use libero::components::{Tree, TreeNode};
/// # fn app() -> Element {
/// let data = vec![TreeNode::new("src", "src".to_string())
///     .children(vec![TreeNode::new("main", "main.rs".to_string())])];
/// let mut open = use_signal(|| HashSet::from(["src".to_string()]));
/// rsx! {
///     Tree {
///         aria_label: "Files",
///         data,
///         expanded: open(),
///         onexpandedchange: move |next| open.set(next),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/navigation/tree>
#[component]
pub fn Tree<T: TreeValue>(props: TreeProps<T>) -> Element {
    // Erased once, so only this shim monomorphizes per `T`. Cached, not a signal,
    // so stable pointers let an untouched subtree skip.
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
            sx: parts_under_sx(&props.parts, props.sx),
            states: props.states,
            size: props.size,
            guides: props.guides,
            aria_label: props.aria_label,
            data: erased_data,
            render_node: erased_render_node,
            default_expanded: props.default_expanded,
            expanded: props.expanded,
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
        #[props(default)]
        guides: Option<bool>,
        #[props(into)]
        aria_label: String,
        data: Vec<TreeNodeErased>,
        render_node: ErasedRenderNode,
        #[props(default)]
        default_expanded: HashSet<String>,
        #[props(default)]
        expanded: Option<HashSet<String>>,
        #[props(default)]
        current: Option<String>,
        #[props(default)]
        onexpandedchange: Option<EventHandler<HashSet<String>>>,
    }
}

/// The real `Tree`, non-generic so it compiles once.
#[component]
fn TreeCore(props: TreeCoreProps) -> Element {
    let theme = use_theme();
    let root = use_element();
    let mut active_id = use_signal(|| None::<String>);
    let mut expanded = use_signal(|| {
        props
            .expanded
            .clone()
            .unwrap_or_else(|| props.default_expanded.clone())
    });
    // Guarded render-time write, as `List` does, so an unchanged set wakes no row.
    if let Some(controlled) = &props.expanded
        && *expanded.peek() != *controlled
    {
        expanded.set(controlled.clone());
    }
    if props.expanded.is_some() && props.onexpandedchange.is_none() {
        warn("Tree: a controlled `expanded` without `onexpandedchange` never opens or closes.");
    }
    let expansion = Expansion {
        open: expanded,
        controlled: props.expanded.is_some(),
        onchange: props.onexpandedchange,
    };

    let size = props.size.copied_or(theme.tree.size);
    let guides = props.guides.unwrap_or(theme.tree.guides);

    let order = visible_order(&props.data, &expanded.read());
    let visible = |id: &String| order.iter().any(|node| node.id == id);
    // So a removed row hands the tab stop to the row taking its place.
    let mut last_index = use_hook(|| CopyValue::new(0_usize));
    let vanished = active_id.read().as_ref().is_some_and(|id| !visible(id));
    let resolved_active = active_id
        .read()
        .clone()
        .filter(visible)
        .or_else(|| {
            vanished.then_some(())?;
            let index = last_index().min(order.len().checked_sub(1)?);
            Some(order[index].id.to_string())
        })
        .or_else(|| {
            // Inside a collapsed branch, the branch that hides it.
            let path = path_to(&props.data, props.current.as_deref()?)?;
            path.into_iter()
                .rev()
                .map(str::to_string)
                .find(|id| visible(id))
        })
        .or_else(|| order.first().map(|node| node.id.to_string()));
    if let Some(index) = resolved_active
        .as_deref()
        .and_then(|id| order.iter().position(|node| node.id == id))
    {
        last_index.set(index);
    }

    // A focused row that leaves `data` takes focus with it. Read before the
    // DOM update, while the doomed row still holds focus.
    let refocus = vanished && root.query_selector(":focus").is_ok();
    use_effect(use_reactive!(|resolved_active, refocus| {
        if let Some(id) = resolved_active.filter(|_| refocus) {
            focus_tree_item(&root, &id);
        }
    }));
    // The first row with that id, as `Tabs` does.
    let active_path = resolved_active
        .as_deref()
        .and_then(|id| visible_path(&props.data, &expanded.read(), id));
    let current_path = props
        .current
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

        // Ctrl/Alt/Meta with a navigation key is the browser's chord.
        let navigation = matches!(
            event.key(),
            Key::ArrowDown | Key::ArrowUp | Key::ArrowLeft | Key::ArrowRight | Key::Home | Key::End
        );
        if navigation && has_shortcut_modifier(&event) {
            return;
        }
        match logical_key(&event) {
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
                    Horizontal::Toggle => expansion.toggle(&current),
                    Horizontal::Go(target) => go_to(target),
                    Horizontal::Stay => {}
                }
            }
            Key::Enter => {
                if !node.disabled {
                    event.prevent_default();
                    if node.has_children {
                        expansion.toggle(&current);
                    } else {
                        click_tree_item(&root, &current);
                    }
                }
            }
            Key::Character(ref c) if c == "*" && !has_shortcut_modifier(&event) => {
                event.prevent_default();
                expand_siblings(&order, node, expansion);
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
                        expansion.toggle(&current);
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

    // A clicked link or button in a row hands focus back to the row. From a task,
    // as Blitz focuses the clicked control after the click's handlers.
    let onclick = move |_: Event<MouseData>| {
        spawn(async move {
            when_free(move || {
                if let Some(id) = active_id.peek().clone()
                    && row_control_focused(&root, &id)
                {
                    focus_tree_item(&root, &id);
                }
            });
        });
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
            onclick,
            attributes: props.attributes,
            for (index , node) in props.data.iter().enumerate() {
                TreeRow {
                    key: "{node.id}",
                    node: node.clone(),
                    size,
                    depth: 0,
                    expansion,
                    active: child_active(active_path.as_deref(), index),
                    current: child_active(current_path.as_deref(), index),
                    active_id,
                    render_node: props.render_node.clone(),
                    guides,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::part_table;

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            part_table::<TreePart>(),
            [
                ("row", "& [data-slot='row']"),
                ("content", "& [data-slot='row'] > [data-slot='content']"),
                ("group", "& [data-slot='row'] > [data-slot='group']"),
            ]
        );
    }

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

    /// Todo 430: the mouse already could not reach a child of a disabled
    /// branch, so the keys treat it as disabled too.
    #[test]
    fn a_disabled_branch_disables_its_visible_children() {
        let nodes = erase_nodes::<String>(&[
            TreeNode::new("a", "a".to_string())
                .disabled(true)
                .children(vec![TreeNode::new("a1", "a1".to_string())]),
            TreeNode::new("b", "b".to_string()),
        ]);
        let open: HashSet<String> = ["a".to_string()].into();
        let order = visible_order(&nodes, &open);
        let disabled: Vec<_> = order.iter().map(|node| (node.id, node.disabled)).collect();
        assert_eq!(disabled, [("a", true), ("a1", true), ("b", false)]);
        assert_eq!(order[1].parent_id, Some("a"));
    }

    /// Todo 516: `*` opens the closed, enabled branches on the focused row's
    /// level only.
    #[test]
    fn star_opens_the_closed_enabled_branches_beside_the_row() {
        let order = [
            branch("a", None, false),
            branch("b", None, true),
            branch("c", None, false),
            branch("c1", Some("c"), false),
            node("d", "Leaf", false),
        ];
        let open: HashSet<String> = ["c".to_string()].into();
        assert_eq!(closed_siblings(&order, &order[4], &open), ["a"]);
        assert_eq!(closed_siblings(&order, &order[3], &open), ["c1"]);
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
