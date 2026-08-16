use std::{
    collections::HashSet,
    sync::atomic::{AtomicU64, Ordering},
};

use dioxus::{document, prelude::*};

use crate::{
    components::{
        Box, Icon, Input, List,
        common::{focus_ring_sx, states},
    },
    hooks::use_theme,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{Size, SizeCss},
};

use super::tree_node::{TreeLabel, TreeNode, TreeNodeRenderArgs};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

// The `<li role="treeitem">` itself - block-level, not flex, so a nested
// `List` (a node's children) stacks below its own row instead of beside it,
// same reasoning as `ListItem`'s own base. No hover/selected background here
// on purpose: this element structurally contains its own children's `<li>`s,
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
// (that's a sibling of this element, not inside it) - so hover/selected only
// ever paints this row's own content, never a descendant's.
static TREE_ROW_CONTENT_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("center")
        .gap("6px")
        .padding("4px 8px")
        .border_radius("4px")
        .cursor("pointer")
        .hover(sx().background("grey.1"))
        .when("selected", sx().background("primary.1"))
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

fn activate_node(
    id: &str,
    has_children: bool,
    disabled: bool,
    expanded: &HashSet<String>,
    onexpandedchange: EventHandler<HashSet<String>>,
    onselectedchange: EventHandler<Option<String>>,
) {
    if disabled {
        return;
    }
    if has_children {
        let mut next = expanded.clone();
        if !next.remove(id) {
            next.insert(id.to_string());
        }
        onexpandedchange.call(next);
    }
    onselectedchange.call(Some(id.to_string()));
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
    /// Required - WAI-ARIA's tree pattern needs an accessible name on the root.
    #[props(into)]
    aria_label: String,
    data: Vec<TreeNode<T>>,
    /// Renders each visible row's content given its data and live state
    /// (expanded/selected/disabled). Falls back to `T::tree_label()` as
    /// plain text when not given.
    #[props(default)]
    render_node: Option<Callback<TreeNodeRenderArgs<T>, Element>>,
    #[props(default)]
    expanded: HashSet<String>,
    #[props(default)]
    onexpandedchange: EventHandler<HashSet<String>>,
    #[props(default)]
    selected: Option<String>,
    #[props(default)]
    onselectedchange: EventHandler<Option<String>>,
}

#[component]
pub fn Tree<T: TreeLabel + Clone + PartialEq + 'static>(props: TreeProps<T>) -> Element {
    let theme = use_theme();
    let root_id = use_hook(|| format!("lsx-tree-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed)));
    let active_id = use_signal(|| None::<String>);

    let size = match props.size.as_ref() {
        Some(ThemeAwareValue::Size(size)) => *size,
        _ => theme.tree.size,
    };

    let order = visible_order(&props.data, &props.expanded);
    let resolved_active = active_id
        .read()
        .clone()
        .filter(|id| order.iter().any(|node| &node.id == id))
        .or_else(|| order.first().map(|node| node.id.clone()));

    let expanded_for_keydown = props.expanded.clone();
    let onexpandedchange = props.onexpandedchange;
    let onselectedchange = props.onselectedchange;
    let data_for_keydown = props.data.clone();
    let root_id_for_keydown = root_id.clone();
    let mut active_id_for_keydown = active_id;
    let resolved_active_for_keydown = resolved_active.clone();

    let onkeydown = move |event: Event<KeyboardData>| {
        let Some(current) = resolved_active_for_keydown.clone() else {
            return;
        };
        let order = visible_order(&data_for_keydown, &expanded_for_keydown);
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
                if expanded_for_keydown.contains(&current) {
                    go_to(sibling_id(&order, &current, 1));
                } else {
                    let mut next = expanded_for_keydown.clone();
                    next.insert(current.clone());
                    onexpandedchange.call(next);
                }
            }
            Key::ArrowLeft => {
                event.prevent_default();
                if node.has_children && expanded_for_keydown.contains(&current) {
                    let mut next = expanded_for_keydown.clone();
                    next.remove(&current);
                    onexpandedchange.call(next);
                } else {
                    go_to(node.parent_id.clone());
                }
            }
            Key::Enter => {
                event.prevent_default();
                activate_node(
                    &current,
                    node.has_children,
                    node.disabled,
                    &expanded_for_keydown,
                    onexpandedchange,
                    onselectedchange,
                );
            }
            Key::Character(ref c) if c == " " => {
                event.prevent_default();
                activate_node(
                    &current,
                    node.has_children,
                    node.disabled,
                    &expanded_for_keydown,
                    onexpandedchange,
                    onselectedchange,
                );
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

    rsx! {
        List {
            id: "{root_id}",
            class: props.class,
            sx: props.sx,
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
                    expanded: props.expanded.clone(),
                    selected: props.selected.clone(),
                    resolved_active: resolved_active.clone(),
                    active_id,
                    render_node: props.render_node,
                    onexpandedchange: props.onexpandedchange,
                    onselectedchange: props.onselectedchange,
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct TreeRowProps<T: TreeLabel + Clone + PartialEq + 'static> {
    node: TreeNode<T>,
    size: Size,
    expanded: HashSet<String>,
    selected: Option<String>,
    resolved_active: Option<String>,
    active_id: Signal<Option<String>>,
    render_node: Option<Callback<TreeNodeRenderArgs<T>, Element>>,
    onexpandedchange: EventHandler<HashSet<String>>,
    onselectedchange: EventHandler<Option<String>>,
}

#[component]
fn TreeRow<T: TreeLabel + Clone + PartialEq + 'static>(props: TreeRowProps<T>) -> Element {
    let node = &props.node;
    let has_children = node.has_children();
    let is_expanded = has_children.then(|| props.expanded.contains(&node.id));
    let is_selected = props.selected.as_deref() == Some(node.id.as_str());
    let disabled = node.disabled;
    let is_roving_active = props.resolved_active.as_deref() == Some(node.id.as_str());

    let content = match &props.render_node {
        Some(render_node) => render_node.call(TreeNodeRenderArgs {
            id: node.id.clone(),
            data: node.data.clone(),
            expanded: is_expanded,
            selected: is_selected,
            disabled,
        }),
        None => rsx! { "{node.data.tree_label()}" },
    };

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

    let row_states = states()
        .with("selected", is_selected)
        .with("disabled", disabled);

    let mut active_id = props.active_id;
    let id = node.id.clone();
    let expanded_set = props.expanded.clone();
    let onexpandedchange = props.onexpandedchange;
    let onselectedchange = props.onselectedchange;

    let onclick = move |event: MouseEvent| {
        // Every ancestor row up to the tree root also has its own `onclick`
        // - without this, clicking a leaf would bubble up and also toggle
        // its parent group's expanded state.
        event.stop_propagation();
        active_id.set(Some(id.clone()));
        activate_node(
            &id,
            has_children,
            disabled,
            &expanded_set,
            onexpandedchange,
            onselectedchange,
        );
    };

    rsx! {
        Box {
            component: "li",
            framework_sx: &TREE_ROW_SX,
            states: row_states.clone(),
            "role": "treeitem",
            "data-tree-id": "{node.id}",
            "aria-selected": "{is_selected}",
            "aria-expanded": is_expanded.map(|value| value.to_string()),
            "aria-disabled": disabled.then_some("true"),
            tabindex: if is_roving_active { "0" } else { "-1" },
            onclick,
            Box {
                framework_sx: &TREE_ROW_CONTENT_SX,
                states: row_states,
                {leading}
                {content}
            }
            if is_expanded == Some(true) {
                List {
                    "role": "group",
                    size: ThemeAwareValue::Size(props.size),
                    for child in &node.children {
                        TreeRow {
                            key: "{child.id}",
                            node: child.clone(),
                            size: props.size,
                            expanded: props.expanded.clone(),
                            selected: props.selected.clone(),
                            resolved_active: props.resolved_active.clone(),
                            active_id: props.active_id,
                            render_node: props.render_node,
                            onexpandedchange: props.onexpandedchange,
                            onselectedchange: props.onselectedchange,
                        }
                    }
                }
            }
        }
    }
}
