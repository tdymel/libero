use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            Input, base_props, css_string, has_shortcut_modifier, parts_enum, parts_under_sx,
        },
        data_display::List,
        form::{CheckboxLook, Disabled, use_checkbox_look},
    },
    hooks::{
        ElementHandle, TYPEAHEAD_RESET, listener, typeahead_match, use_element, use_theme,
        use_typeahead,
    },
    platform::{
        ElementApi, OBSERVE_ATTR, PlatformError, click_selector, focus_closest_of_focused,
        focus_selector, logical_key, meta_held, when_free,
    },
    theme::Size,
    utils::warn,
};

use super::{
    tree_node::{
        ErasedRenderNode, TreeNode, TreeNodeErased, TreeNodeRenderArgs, TreeValue, erase_nodes,
    },
    tree_row::{TreeRow, child_active, open_key},
};

/// Borrows from `data`: rebuilt on every keystroke, owning meant two `String`s per node.
struct VisibleNode<'a> {
    id: &'a str,
    parent_id: Option<&'a str>,
    has_children: bool,
    disabled: bool,
    label: &'a str,
}

/// `disabled` cascades to the nodes under a disabled branch, or all of them when `off`.
fn push_visible_nodes<'a>(
    nodes: &'a [TreeNodeErased],
    expanded: &HashSet<String>,
    parent: Option<(&'a str, bool)>,
    off: bool,
    out: &mut Vec<VisibleNode<'a>>,
) {
    let (parent_id, inherited) = (parent.map(|(id, _)| id), parent.is_some_and(|(_, off)| off));
    for node in nodes {
        let disabled = off || inherited || node.disabled;
        out.push(VisibleNode {
            id: &node.id,
            parent_id,
            has_children: node.has_children(),
            disabled,
            label: &node.label,
        });
        if node.has_children() && expanded.contains(&node.id) {
            push_visible_nodes(
                &node.children,
                expanded,
                Some((&node.id, disabled)),
                off,
                out,
            );
        }
    }
}

fn visible_order<'a>(
    nodes: &'a [TreeNodeErased],
    expanded: &HashSet<String>,
    off: bool,
) -> Vec<VisibleNode<'a>> {
    let mut out = Vec::new();
    push_visible_nodes(nodes, expanded, None, off, &mut out);
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

/// Where focus goes if `id` cannot take it: its parent, then the rows around it, nearest first.
fn focus_fallbacks(order: &[VisibleNode], id: &str) -> Vec<String> {
    let Some(index) = order.iter().position(|node| node.id == id) else {
        return Vec::new();
    };
    let near = (1..order.len()).flat_map(|gap| {
        [index.checked_sub(gap), index.checked_add(gap)]
            .into_iter()
            .flatten()
            .filter_map(|at| order.get(at))
    });
    order[index]
        .parent_id
        .into_iter()
        .chain(near.map(|node| node.id).take(4))
        .map(str::to_string)
        .collect()
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

/// A row's checkbox, as `aria-checked` reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Check {
    On,
    Off,
    Mixed,
}

impl Check {
    pub fn aria(self) -> &'static str {
        match self {
            Check::On => "true",
            Check::Off => "false",
            Check::Mixed => "mixed",
        }
    }
}

/// Every node's check. Unless `strict`, a checked node checks its enabled rows below, and a
/// branch reads its enabled children: all on is on, all off is off, else mixed.
fn check_states(
    nodes: &[TreeNodeErased],
    checked: &HashSet<String>,
    strict: bool,
    off: bool,
) -> HashMap<String, Check> {
    fn walk(
        node: &TreeNodeErased,
        checked: &HashSet<String>,
        strict: bool,
        (inherited, disabled_above): (bool, bool),
        out: &mut HashMap<String, Check>,
    ) -> Check {
        let disabled = disabled_above || node.disabled;
        let own = checked.contains(&node.id) || (inherited && !disabled);
        let (mut on, mut off) = (false, false);
        for child in node.children.iter() {
            let state = walk(child, checked, strict, (own && !strict, disabled), out);
            if !(disabled || child.disabled) {
                on |= state != Check::Off;
                off |= state != Check::On;
            }
        }
        let state = match (strict, on, off) {
            (false, true, true) => Check::Mixed,
            (false, true, false) => Check::On,
            (false, false, true) => Check::Off,
            _ if own => Check::On,
            _ => Check::Off,
        };
        out.insert(node.id.clone(), state);
        state
    }
    let mut out = HashMap::new();
    for node in nodes {
        walk(node, checked, strict, (false, off), &mut out);
    }
    out
}

fn find_node<'a>(nodes: &'a [TreeNodeErased], id: &str) -> Option<&'a TreeNodeErased> {
    nodes.iter().find_map(|node| match node.id == id {
        true => Some(node),
        false => find_node(&node.children, id),
    })
}

/// `checked` after a click or Space on `id`. Unless `strict`, every fully checked node.
fn toggled_checks(
    nodes: &[TreeNodeErased],
    checked: &HashSet<String>,
    strict: bool,
    off: bool,
    id: &str,
) -> HashSet<String> {
    let mut next = checked.clone();
    if strict {
        if !next.remove(id) {
            next.insert(id.to_string());
        }
        return next;
    }
    let states = check_states(nodes, checked, false, off);
    let on = |states: &HashMap<String, Check>| -> HashSet<String> {
        states
            .iter()
            .filter(|(_, state)| **state == Check::On)
            .map(|(id, _)| id.clone())
            .collect()
    };
    let want = states.get(id) != Some(&Check::On);
    next = on(&states);
    // A checked ancestor would check `id`'s rows again; the recount restores it.
    for ancestor in path_to(nodes, id).unwrap_or_default().iter().rev().skip(1) {
        next.remove(*ancestor);
    }
    fn set(node: &TreeNodeErased, want: bool, next: &mut HashSet<String>) {
        match want {
            true => next.insert(node.id.clone()),
            false => next.remove(&node.id),
        };
        for child in node.children.iter().filter(|child| !child.disabled) {
            set(child, want, next);
        }
    }
    if let Some(node) = find_node(nodes, id) {
        set(node, want, &mut next);
    }
    on(&check_states(nodes, &next, false, off))
}

/// How a click or key picks a row in a `multiple` tree; a single one always replaces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Pick {
    Toggle,
    /// The enabled visible rows from the anchor to the row, added.
    Range,
}

fn picked(
    selected: &HashSet<String>,
    multiple: bool,
    order: &[VisibleNode],
    anchor: Option<&str>,
    id: &str,
    pick: Pick,
) -> HashSet<String> {
    if !multiple {
        return HashSet::from([id.to_string()]);
    }
    let at = |id: &str| order.iter().position(|node| node.id == id);
    match (pick, anchor.and_then(at), at(id)) {
        (Pick::Range, Some(from), Some(to)) => {
            let mut next = selected.clone();
            next.extend(enabled_between(order, from, to));
            next
        }
        _ => {
            let mut next = selected.clone();
            if !next.remove(id) {
                next.insert(id.to_string());
            }
            next
        }
    }
}

fn enabled_between(order: &[VisibleNode], from: usize, to: usize) -> Vec<String> {
    order[from.min(to)..=from.max(to)]
        .iter()
        .filter(|node| !node.disabled)
        .map(|node| node.id.to_string())
        .collect()
}

/// Every enabled node, collapsed ones included: Ctrl+A's set.
fn enabled_ids(nodes: &[TreeNodeErased], off: bool, out: &mut Vec<String>) {
    for node in nodes.iter().filter(|node| !(off || node.disabled)) {
        out.push(node.id.clone());
        enabled_ids(&node.children, off, out);
    }
}

/// The selection and checks, which each row reads through a memo of its own.
#[derive(Clone, Copy)]
pub(super) struct TreeMarks {
    /// `None` without `Tree`'s `selected`.
    pub selected: Signal<Option<HashSet<String>>>,
    /// `None` without `Tree`'s `checked`.
    pub checks: Signal<Option<HashMap<String, Check>>>,
    pub pick: Callback<(String, Pick)>,
    pub check: Callback<String>,
    pub look: Signal<Option<CheckboxLook>>,
    /// The row whose Enter clicked its control: that click's bubble must not pick again.
    pub enter_click: CopyValue<Option<String>>,
}

/// The row whose `<li>` holds focus, kept from the rows' focus events for a
/// WebView, whose handles cannot query (5).
#[derive(Clone, Copy)]
pub(super) struct FocusedRow(pub Signal<Option<String>>);

/// This tree's rows: through the root's handle, or on a WebView by page-side
/// selectors scoped to the root's tag, so two `Tree`s can share node ids.
#[derive(Clone, Copy)]
struct TreeDom {
    root: ElementHandle,
    focused: Signal<Option<String>>,
    /// Whether focus is anywhere in the tree, from its `focusin`/`focusout`.
    inside: Signal<bool>,
}

impl TreeDom {
    fn page_row(self, id: &str) -> String {
        let tag = self.root.tag().unwrap_or_default();
        format!("[{OBSERVE_ATTR}='{tag}'] [data-tree-id={}]", css_string(id))
    }

    // A leaf's link, button or `TreeItem` is out of the tab order, so Enter clicks it from here.
    fn click(self, target_id: &str) {
        let id = css_string(target_id);
        let selector = format!("[data-tree-id={id}] :is(a, button, [data-tree-activate])");
        match self.root.query_selector(&selector) {
            Err(PlatformError::Unsupported) => {
                let row = self.page_row(target_id);
                let _ = click_selector(&format!("{row} :is(a, button, [data-tree-activate])"));
            }
            found => {
                let _ = found.and_then(|el| el.click());
            }
        }
    }

    // Only while the row itself holds focus; an input inside a row keeps its keys.
    fn focused(self, target_id: &str) -> bool {
        let selector = format!("[data-tree-id={}]", css_string(target_id));
        match self.root.query_selector(&selector) {
            Ok(el) => el.is_focused(),
            Err(PlatformError::Unsupported) => self.focused.peek().as_deref() == Some(target_id),
            Err(_) => false,
        }
    }

    // A clicked control in the row's own content (`> div`), not a nested row's,
    // hands focus back to the row.
    fn refocus_from_control(self, target_id: &str) {
        let id = css_string(target_id);
        let selector = format!("[data-tree-id={id}] > div :is(a, button, [data-tree-activate])");
        match self.root.query_selector(&selector) {
            Err(PlatformError::Unsupported) => {
                let row = self.page_row(target_id);
                let _ = focus_closest_of_focused(
                    &format!("{row} > div :is(a, button, [data-tree-activate])"),
                    &format!("[data-tree-id={id}]"),
                );
            }
            found => {
                if found.is_ok_and(|el| el.is_focused()) {
                    self.focus(target_id);
                }
            }
        }
    }

    /// Whether the row was found and focused.
    fn focus(self, target_id: &str) -> bool {
        let selector = format!("[data-tree-id={}]", css_string(target_id));
        match self.root.query_selector(&selector) {
            Err(PlatformError::Unsupported) => focus_selector(&self.page_row(target_id)).is_ok(),
            found => found.and_then(|el| el.focus()).is_ok(),
        }
    }

    // Whether focus sits anywhere in the tree; a WebView's row button counts too.
    fn holds_focus(self) -> bool {
        match self.root.query_selector(":focus") {
            Err(PlatformError::Unsupported) => *self.inside.peek(),
            found => found.is_ok(),
        }
    }
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
        /// Each row's content. Defaults to `default_tree_render`, which a custom one can call.
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
        /// The selected rows' ids, strictly controlled: set, the rows are selectable
        /// (`aria-selected`). A click or Space picks a row.
        #[props(default)]
        selected: Option<HashSet<String>>,
        /// With `selected`, any number of rows: a click or Space toggles one, Shift extends.
        #[props(default)]
        multiple: bool,
        /// The whole new set of selected ids.
        #[props(default)]
        onselectedchange: Option<EventHandler<HashSet<String>>>,
        /// The checked rows' ids, strictly controlled: set, every row draws a checkbox
        /// (`aria-checked`). Space or a click on the box toggles it.
        #[props(default)]
        checked: Option<HashSet<String>>,
        /// With `checked`, each row checks alone. Unset, a branch checks its enabled rows
        /// and reads mixed when only some are, and `oncheckedchange` gets every fully checked node.
        #[props(default)]
        check_strictly: bool,
        /// The whole new set of checked ids.
        #[props(default)]
        oncheckedchange: Option<EventHandler<HashSet<String>>>,
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
            selected: args.selected,
            checked: args.checked,
            indeterminate: args.indeterminate,
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
            selected: props.selected,
            multiple: props.multiple,
            onselectedchange: props.onselectedchange,
            checked: props.checked,
            check_strictly: props.check_strictly,
            oncheckedchange: props.oncheckedchange,
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
        #[props(default)]
        selected: Option<HashSet<String>>,
        #[props(default)]
        multiple: bool,
        #[props(default)]
        onselectedchange: Option<EventHandler<HashSet<String>>>,
        #[props(default)]
        checked: Option<HashSet<String>>,
        #[props(default)]
        check_strictly: bool,
        #[props(default)]
        oncheckedchange: Option<EventHandler<HashSet<String>>>,
    }
}

/// The rows' [`TreeMarks`]: the controlled sets mirrored into signals, guarded so an
/// unchanged set wakes no row, and the callbacks that ask for a new one.
fn use_marks(
    props: &TreeCoreProps,
    open: Signal<HashSet<String>>,
    off: bool,
    size: Size,
) -> TreeMarks {
    if props.selected.is_some() && props.onselectedchange.is_none() {
        warn("Tree: a controlled `selected` without `onselectedchange` never changes.");
    }
    if props.checked.is_some() && props.oncheckedchange.is_none() {
        warn("Tree: a controlled `checked` without `oncheckedchange` never changes.");
    }
    let checks_now = props
        .checked
        .as_ref()
        .map(|checked| check_states(&props.data, checked, props.check_strictly, off));
    let mut selected = use_signal(|| props.selected.clone());
    if *selected.peek() != props.selected {
        selected.set(props.selected.clone());
    }
    let mut checks = use_signal(|| checks_now.clone());
    if *checks.peek() != checks_now {
        checks.set(checks_now);
    }
    let look_now = use_checkbox_look(size, props.checked.is_some());
    let mut look = use_signal(|| look_now.clone());
    if *look.peek() != look_now {
        look.set(look_now);
    }

    let mut anchor = use_hook(|| CopyValue::new(None::<String>));
    let (data, current, multiple) = (props.data.clone(), props.selected.clone(), props.multiple);
    let onchange = props.onselectedchange;
    let pick = use_callback(move |(id, pick): (String, Pick)| {
        let (Some(current), Some(onchange)) = (&current, &onchange) else {
            return;
        };
        let order = visible_order(&data, &open.read(), off);
        let next = picked(
            current,
            multiple,
            &order,
            anchor.peek().as_deref(),
            &id,
            pick,
        );
        if pick == Pick::Toggle || anchor.peek().is_none() {
            anchor.set(Some(id));
        }
        onchange.call(next);
    });
    let (data, checked, strict) = (
        props.data.clone(),
        props.checked.clone(),
        props.check_strictly,
    );
    let onchange = props.oncheckedchange;
    let check = use_callback(move |id: String| {
        if let (Some(checked), Some(onchange)) = (&checked, &onchange) {
            onchange.call(toggled_checks(&data, checked, strict, off, &id));
        }
    });
    let enter_click = use_hook(|| CopyValue::new(None));
    use_context_provider(|| TreeMarks {
        selected,
        checks,
        pick,
        check,
        look,
        enter_click,
    })
}

/// The real `Tree`, non-generic so it compiles once.
#[component]
fn TreeCore(props: TreeCoreProps) -> Element {
    let theme = use_theme();
    let root = use_element();
    let FocusedRow(focused) = use_context_provider(|| FocusedRow(Signal::new(None)));
    let mut inside = use_signal(|| false);
    let dom = TreeDom {
        root,
        focused,
        inside,
    };
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

    // A disabled `Fieldset` cannot reach a span row the way it does a button.
    let group_disabled = try_use_context::<Disabled>().is_some_and(|Disabled(group)| group());
    let order = visible_order(&props.data, &expanded.read(), group_disabled);
    let marks = use_marks(&props, expansion.open, group_disabled, size);
    let selectable = props.selected.is_some();
    let checkable = props.checked.is_some();
    let multiple = selectable && props.multiple;
    let visible = |id: &String| order.iter().any(|node| node.id == id);
    // So a removed row hands the tab stop to the row taking its place.
    let mut last_index = use_hook(|| CopyValue::new(0_usize));
    let vanished = active_id.read().as_ref().is_some_and(|id| !visible(id));
    // Inside a collapsed branch, the branch that hides it.
    let hiding_branch = |id: &str| {
        path_to(&props.data, id)?
            .into_iter()
            .rev()
            .map(str::to_string)
            .find(|id| visible(id))
    };
    let resolved_active = active_id
        .read()
        .clone()
        .filter(visible)
        .or_else(|| hiding_branch(active_id.read().as_deref()?))
        .or_else(|| {
            // Only a row gone from `data`: the one now at its place.
            vanished.then_some(())?;
            let index = last_index().min(order.len().checked_sub(1)?);
            Some(order[index].id.to_string())
        })
        .or_else(|| hiding_branch(props.current.as_deref()?))
        .or_else(|| order.first().map(|node| node.id.to_string()));
    if let Some(index) = resolved_active
        .as_deref()
        .and_then(|id| order.iter().position(|node| node.id == id))
    {
        last_index.set(index);
    }

    // A focused row that leaves `data` takes focus with it. Read before the
    // DOM update, while the doomed row still holds focus.
    let refocus = vanished && dom.holds_focus();
    let fallbacks = match (&resolved_active, refocus) {
        (Some(id), true) => focus_fallbacks(&order, id),
        _ => Vec::new(),
    };
    use_effect(use_reactive!(|resolved_active, refocus, fallbacks| {
        if let Some(id) = resolved_active.filter(|_| refocus) {
            // A row that is not drawn leaves focus to its parent, then its nearest rows.
            let _ = std::iter::once(id)
                .chain(fallbacks)
                .any(|id| dom.focus(&id));
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
    let selected_for_keydown = props.selected.clone();
    let onselectedchange = props.onselectedchange;

    let onkeydown = move |event: Event<KeyboardData>| {
        let Some(current) = resolved_active_for_keydown.clone() else {
            return;
        };
        if !dom.focused(&current) {
            return;
        }
        let order = visible_order(&data_for_keydown, &expanded.read(), group_disabled);
        let Some(node) = order.iter().find(|node| node.id == current) else {
            return;
        };
        // A bool, not the set: `toggle_expanded` writes to `expanded` below.
        let is_expanded = expanded.read().contains(&current);

        let mut go_to = |target: Option<String>| {
            if let Some(target) = target {
                active_id_for_keydown.set(Some(target.clone()));
                dom.focus(&target);
            }
        };
        let modifiers = event.data().modifiers();
        let shift = modifiers.shift();

        // APG's multi-select chords, ahead of the browser's below.
        if multiple && (modifiers.ctrl() || meta_held(modifiers)) && !modifiers.alt() {
            let index = order
                .iter()
                .position(|node| node.id == current)
                .unwrap_or(0);
            let last = order.len() - 1;
            let chord = match logical_key(&event) {
                Key::Named(NamedKey::Home) if shift => {
                    Some((Some(0), enabled_between(&order, 0, index)))
                }
                Key::Named(NamedKey::End) if shift => {
                    Some((Some(last), enabled_between(&order, index, last)))
                }
                Key::Character(ref c) if !shift && c.eq_ignore_ascii_case("a") => {
                    let mut all = Vec::new();
                    enabled_ids(&data_for_keydown, group_disabled, &mut all);
                    Some((None, all))
                }
                _ => None,
            };
            if let (Some((to, ids)), Some(selected)) = (chord, &selected_for_keydown) {
                event.prevent_default();
                let mut next = selected.clone();
                // Ctrl+A on a fully selected tree clears it (APG's option).
                if to.is_none() && ids.iter().all(|id| next.contains(id)) {
                    ids.iter().for_each(|id| {
                        next.remove(id);
                    });
                } else {
                    next.extend(ids);
                }
                if let Some(onchange) = &onselectedchange {
                    onchange.call(next);
                }
                go_to(to.map(|to| order[to].id.to_string()));
                return;
            }
        }
        // Shift+Up or Down toggles the row it lands on.
        let extend = |target: &Option<String>| {
            let enabled = |id: &String| order.iter().any(|node| node.id == id && !node.disabled);
            if let Some(target) = target
                .as_ref()
                .filter(|_| shift && multiple)
                .filter(|id| enabled(id))
            {
                marks.pick.call((target.clone(), Pick::Toggle));
            }
        };

        // Ctrl/Alt/Meta with a navigation key is the browser's chord.
        let navigation = matches!(
            event.key(),
            Key::Named(NamedKey::ArrowDown)
                | Key::Named(NamedKey::ArrowUp)
                | Key::Named(NamedKey::ArrowLeft)
                | Key::Named(NamedKey::ArrowRight)
                | Key::Named(NamedKey::Home)
                | Key::Named(NamedKey::End)
        );
        if navigation && has_shortcut_modifier(&event) {
            return;
        }
        // A lone row has nowhere to go: the keys scroll the page (todo 2465).
        let lone = order.len() == 1;
        if lone
            && matches!(
                event.key(),
                Key::Named(NamedKey::ArrowDown)
                    | Key::Named(NamedKey::ArrowUp)
                    | Key::Named(NamedKey::Home)
                    | Key::Named(NamedKey::End)
            )
        {
            return;
        }
        let at = |end: Option<&VisibleNode>| end.is_some_and(|end| end.id == current);
        match logical_key(&event) {
            // Home on the first or End on the last row scrolls the page (todo 3053).
            Key::Named(NamedKey::Home) if at(order.first()) => {}
            Key::Named(NamedKey::End) if at(order.last()) => {}
            Key::Named(NamedKey::ArrowDown) => {
                event.prevent_default();
                let target = sibling_id(&order, &current, 1);
                extend(&target);
                go_to(target);
            }
            Key::Named(NamedKey::ArrowUp) => {
                event.prevent_default();
                let target = sibling_id(&order, &current, -1);
                extend(&target);
                go_to(target);
            }
            Key::Named(NamedKey::Home) => {
                event.prevent_default();
                go_to(order.first().map(|node| node.id.to_string()));
            }
            Key::Named(NamedKey::End) => {
                event.prevent_default();
                go_to(order.last().map(|node| node.id.to_string()));
            }
            key @ (Key::Named(NamedKey::ArrowRight) | Key::Named(NamedKey::ArrowLeft)) => {
                let forward = key == Key::Named(NamedKey::ArrowRight);
                let Some(step) = horizontal(&order, node, is_expanded, forward)
                    .filter(|step| !lone || *step == Horizontal::Toggle)
                else {
                    return;
                };
                event.prevent_default();
                match step {
                    Horizontal::Toggle => expansion.toggle(&current),
                    Horizontal::Go(target) => go_to(target),
                    Horizontal::Stay => {}
                }
            }
            Key::Named(NamedKey::Enter) => {
                if !node.disabled {
                    event.prevent_default();
                    if node.has_children {
                        expansion.toggle(&current);
                    } else {
                        if selectable {
                            let mut enter_click = marks.enter_click;
                            enter_click.set(Some(current.clone()));
                            // A single selection picks the leaf it activates.
                            if !multiple {
                                marks.pick.call((current.clone(), Pick::Toggle));
                            }
                        }
                        dom.click(&current);
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
                    if checkable {
                        marks.check.call(current.clone());
                    } else if selectable {
                        let pick = if shift { Pick::Range } else { Pick::Toggle };
                        marks.pick.call((current.clone(), pick));
                    } else if node.has_children {
                        expansion.toggle(&current);
                    } else {
                        dom.click(&current);
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
                if let Some(id) = active_id.peek().clone() {
                    dom.refocus_from_control(&id);
                }
            });
        });
    };

    // A move inside fires `focusout` then `focusin`, so the last write is `true`.
    let focus_attributes = vec![
        listener("onfocusin", move |_: FocusEvent| inside.set(true)),
        listener("onfocusout", move |_: FocusEvent| inside.set(false)),
    ];
    let root_sx = props.sx.into_option().unwrap_or_default();
    let open_keys: Vec<u64> = props
        .data
        .iter()
        .map(|node| open_key(node, &expanded.read()))
        .collect();

    rsx! {
        List {
            class: props.class,
            onmounted: root.mount(),
            sx: root_sx,
            states: props.states,
            size,
            "role": "tree",
            "aria-label": Some(props.aria_label).filter(|label| !label.trim().is_empty()),
            "aria-multiselectable": multiple.then_some("true"),
            onkeydown,
            onclick,
            attributes: [
                props.attributes,
                root.attributes(),
                focus_attributes,
            ]
            .concat(),
            for (index , node) in props.data.iter().enumerate() {
                TreeRow {
                    key: "{node.id}",
                    node: node.clone(),
                    size,
                    depth: 0,
                    expansion,
                    open_key: open_keys[index],
                    active: child_active(active_path.as_deref(), index),
                    current: child_active(current_path.as_deref(), index),
                    active_id,
                    render_node: props.render_node.clone(),
                    guides,
                    group_disabled,
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
        let order = visible_order(&nodes, &open, false);
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

    /// Todo 3779: a span row is no native control, so the fieldset's flag reaches every row.
    #[test]
    fn a_disabled_fieldset_disables_every_row() {
        fn app() -> Element {
            let data = vec![
                TreeNode::new("a", "a".to_string())
                    .children(vec![TreeNode::new("a1", "a1".to_string())]),
                TreeNode::new("b", "b".to_string()),
            ];
            rsx! {
                crate::LiberoProvider {
                    crate::components::Fieldset::<()> { disabled: true,
                        Tree { aria_label: "Files", data, default_expanded: HashSet::from(["a".to_string()]) }
                    }
                }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert_eq!(html.matches("role=\"treeitem\"").count(), 3, "{html}");
        // The three rows and the fieldset itself.
        assert_eq!(html.matches("aria-disabled=\"true\"").count(), 4, "{html}");
    }

    /// Todo 3780: a row that cannot take focus hands it to its parent, then the nearest rows.
    #[test]
    fn focus_falls_back_to_the_parent_then_the_nearest_rows() {
        let order = [
            node("a", "a", false),
            VisibleNode {
                parent_id: Some("a"),
                ..node("a1", "a1", false)
            },
            VisibleNode {
                parent_id: Some("a"),
                ..node("a2", "a2", false)
            },
            node("b", "b", false),
        ];
        assert_eq!(focus_fallbacks(&order, "a2"), ["a", "a1", "b", "a"]);
        assert_eq!(focus_fallbacks(&order, "a"), ["a1", "a2", "b"]);
        assert!(focus_fallbacks(&order, "zzz").is_empty());
    }

    #[test]
    fn a_disabled_group_disables_the_keys_on_every_visible_row() {
        let nodes = erase_nodes::<String>(&[
            TreeNode::new("a", "a".to_string())
                .children(vec![TreeNode::new("a1", "a1".to_string())]),
            TreeNode::new("b", "b".to_string()),
        ]);
        let open: HashSet<String> = ["a".to_string()].into();
        assert!(
            visible_order(&nodes, &open, true)
                .iter()
                .all(|node| node.disabled)
        );
        assert!(
            visible_order(&nodes, &open, false)
                .iter()
                .all(|node| !node.disabled)
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

    fn ids(list: &[&str]) -> HashSet<String> {
        list.iter().map(|id| id.to_string()).collect()
    }

    /// `a` holds `a1`, `a2` and a disabled `a3`; `b` is a leaf.
    fn checkable() -> Vec<TreeNodeErased> {
        erase_nodes::<String>(&[
            TreeNode::new("a", "a".to_string()).children(vec![
                TreeNode::new("a1", "a1".to_string()),
                TreeNode::new("a2", "a2".to_string()),
                TreeNode::new("a3", "a3".to_string()).disabled(true),
            ]),
            TreeNode::new("b", "b".to_string()),
        ])
    }

    #[test]
    fn a_branch_reads_its_enabled_children() {
        let nodes = checkable();
        let states = check_states(&nodes, &ids(&["a1"]), false, false);
        assert_eq!(states["a"], Check::Mixed);
        // The disabled `a3` does not count against a full branch.
        let states = check_states(&nodes, &ids(&["a1", "a2"]), false, false);
        assert_eq!(states["a"], Check::On);
        assert_eq!(states["a3"], Check::Off);
        // A checked branch checks its enabled rows, not the disabled one.
        let states = check_states(&nodes, &ids(&["a"]), false, false);
        assert_eq!((states["a1"], states["a3"]), (Check::On, Check::Off));
    }

    #[test]
    fn strict_checks_stand_alone() {
        let states = check_states(&checkable(), &ids(&["a"]), true, false);
        assert_eq!((states["a"], states["a1"]), (Check::On, Check::Off));
        assert_eq!(
            toggled_checks(&checkable(), &ids(&["a"]), true, false, "a1"),
            ids(&["a", "a1"])
        );
    }

    #[test]
    fn checking_a_branch_checks_its_enabled_rows_and_the_last_leaf_its_branch() {
        let nodes = checkable();
        assert_eq!(
            toggled_checks(&nodes, &ids(&[]), false, false, "a"),
            ids(&["a", "a1", "a2"])
        );
        assert_eq!(
            toggled_checks(&nodes, &ids(&["a1"]), false, false, "a2"),
            ids(&["a", "a1", "a2"])
        );
        // A mixed branch checks fully; unchecking a leaf unchecks the branch.
        assert_eq!(
            toggled_checks(&nodes, &ids(&["a1"]), false, false, "a"),
            ids(&["a", "a1", "a2"])
        );
        assert_eq!(
            toggled_checks(&nodes, &ids(&["a", "a1", "a2"]), false, false, "a1"),
            ids(&["a2"])
        );
    }

    #[test]
    fn a_pick_replaces_a_single_selection_and_toggles_or_extends_a_multiple_one() {
        let order = order();
        assert_eq!(
            picked(&ids(&["a"]), false, &order, None, "b", Pick::Toggle),
            ids(&["b"])
        );
        assert_eq!(
            picked(&ids(&["a"]), true, &order, None, "b", Pick::Toggle),
            ids(&["a", "b"])
        );
        assert_eq!(
            picked(&ids(&["a", "b"]), true, &order, None, "b", Pick::Toggle),
            ids(&["a"])
        );
        // The disabled `c` stays out of a range.
        assert_eq!(
            picked(&ids(&["a"]), true, &order, Some("a"), "d", Pick::Range),
            ids(&["a", "b", "d"])
        );
    }

    #[test]
    fn select_all_reaches_collapsed_rows_but_not_disabled_ones() {
        let mut all = Vec::new();
        enabled_ids(&checkable(), false, &mut all);
        assert_eq!(all, ["a", "a1", "a2", "b"]);
        let mut none = Vec::new();
        enabled_ids(&checkable(), true, &mut none);
        assert!(none.is_empty());
    }

    #[test]
    fn rows_carry_their_selection_and_check() {
        fn app() -> Element {
            let data = vec![TreeNode::new("a", "a".to_string()).children(vec![
                TreeNode::new("a1", "a1".to_string()),
                TreeNode::new("a2", "a2".to_string()),
            ])];
            rsx! {
                crate::LiberoProvider {
                    Tree {
                        aria_label: "Files",
                        data,
                        default_expanded: HashSet::from(["a".to_string()]),
                        selected: HashSet::from(["a1".to_string()]),
                        multiple: true,
                        onselectedchange: |_| {},
                        checked: HashSet::from(["a2".to_string()]),
                        oncheckedchange: |_| {},
                    }
                }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains(r#"aria-multiselectable="true""#), "{html}");
        assert_eq!(html.matches(r#"aria-selected="true""#).count(), 1, "{html}");
        assert_eq!(html.matches(r#"aria-checked="mixed""#).count(), 1, "{html}");
        assert_eq!(html.matches(r#"aria-checked="true""#).count(), 1, "{html}");
        assert_eq!(html.matches(r#"aria-checked="false""#).count(), 1, "{html}");
        assert_eq!(html.matches(r#"data-slot="box""#).count(), 3, "{html}");
    }
}
