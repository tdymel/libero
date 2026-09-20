use std::{any::Any, rc::Rc};

use dioxus::prelude::*;

use crate::{
    components::{
        common::{ChevronRightIcon, States},
        data_display::Icon,
        layout::Box,
    },
    sx::{StaticSx, sx},
    theme::{ICON_SIZE, Size},
};

/// A node's text, for typeahead and the default `render_node`.
pub trait TreeLabel {
    fn tree_label(&self) -> String;
}

/// A `Tree` payload's bounds as one name, as `base_props!` takes one bound per
/// type parameter. Blanket-implemented.
pub trait TreeValue: TreeLabel + Clone + PartialEq + 'static {}

impl<T: TreeLabel + Clone + PartialEq + 'static> TreeValue for T {}

impl TreeLabel for String {
    fn tree_label(&self) -> String {
        self.clone()
    }
}

impl TreeLabel for &'static str {
    fn tree_label(&self) -> String {
        (*self).to_string()
    }
}

/// One `Tree` node: a unique id, its payload and its children.
#[derive(Clone, PartialEq)]
pub struct TreeNode<T> {
    pub id: String,
    pub data: T,
    pub children: Vec<TreeNode<T>>,
    pub disabled: bool,
}

impl<T> TreeNode<T> {
    pub fn new(id: impl Into<String>, data: T) -> Self {
        Self {
            id: id.into(),
            data,
            children: Vec::new(),
            disabled: false,
        }
    }

    pub fn children(mut self, children: Vec<TreeNode<T>>) -> Self {
        self.children = children;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Type-erased `TreeNode<T>`, so the tree compiles once; `label` is precomputed.
#[derive(Clone)]
pub(super) struct TreeNodeErased {
    pub id: String,
    pub label: String,
    pub children: Vec<TreeNodeErased>,
    pub disabled: bool,
    pub data: Rc<dyn Any>,
}

impl TreeNodeErased {
    pub(super) fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

// `data` compares by pointer identity: `Tree`'s cache re-erases only on a
// real change, so a stable `Rc` is exactly the "subtree untouched" signal.
impl PartialEq for TreeNodeErased {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.label == other.label
            && self.disabled == other.disabled
            && self.children == other.children
            && Rc::ptr_eq(&self.data, &other.data)
    }
}

pub(super) fn erase_nodes<T: TreeLabel + Clone + 'static>(
    nodes: &[TreeNode<T>],
) -> Vec<TreeNodeErased> {
    nodes
        .iter()
        .map(|node| TreeNodeErased {
            id: node.id.clone(),
            label: node.data.tree_label(),
            disabled: node.disabled,
            children: erase_nodes(&node.children),
            data: Rc::new(node.data.clone()) as Rc<dyn Any>,
        })
        .collect()
}

/// Type-erased [`TreeNodeRenderArgs<T>`]; `Tree<T>` downcasts `data` back.
pub(super) struct TreeNodeRenderArgsErased {
    pub id: String,
    pub data: Rc<dyn Any>,
    pub expanded: Option<bool>,
    pub disabled: bool,
    pub tabindex: &'static str,
    pub depth: usize,
}

/// Lets `TreeRowProps` derive `Clone`/`PartialEq` without `T`. Always equal, so
/// it never gates memoization.
#[derive(Clone)]
pub(super) struct ErasedRenderNode(Rc<dyn Fn(TreeNodeRenderArgsErased) -> Element>);

impl ErasedRenderNode {
    pub(super) fn new(f: impl Fn(TreeNodeRenderArgsErased) -> Element + 'static) -> Self {
        Self(Rc::new(f))
    }

    pub(super) fn call(&self, args: TreeNodeRenderArgsErased) -> Element {
        (self.0)(args)
    }
}

impl PartialEq for ErasedRenderNode {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

/// Passed to `render_node` per visible row: the data plus live state. No
/// "selected": a link knows its own.
#[derive(Clone, PartialEq)]
pub struct TreeNodeRenderArgs<T> {
    pub id: String,
    pub data: T,
    /// `None` for a leaf (no children, no chevron, no `aria-expanded`).
    pub expanded: Option<bool>,
    /// By the node's own flag or an ancestor's.
    pub disabled: bool,
    /// Put on any link or button you render; the row is already the tab stop.
    pub tabindex: &'static str,
    /// 0 at top level, for a `render_node` that indents on its own.
    pub depth: usize,
}

// Chevron-width, so leaves line up with their branch siblings.
static DEFAULT_RENDER_LEADING_SPACER_SX: StaticSx =
    StaticSx::new(|| sx().flex_shrink("0").width(ICON_SIZE.value(Size::Xs)));

// Row and chevron in one static, so a visible row builds no `Sx` of its own.
static DEFAULT_RENDER_ROW_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("center")
        .gap("6px")
        .padding("6px 0")
        .selector(
            "& [data-tree-chevron]",
            sx().flex_shrink("0")
                .transition("transform 120ms ease")
                .transform("rotate(0deg)"),
        )
        // A closed row's chevron points to the start of the line. Before the
        // expanded rule, which wins at equal specificity.
        .rtl(sx().selector("& [data-tree-chevron]", sx().transform("rotate(180deg)")))
        .selector(
            "& [data-tree-chevron][data-state~=\"expanded\"]",
            sx().transform("rotate(90deg)"),
        )
});

/// `render_node`'s default: a chevron or a leaf spacer, then `tree_label()`.
/// A custom `render_node` can fall back to it for some rows.
pub fn default_tree_render<T: TreeLabel>(args: TreeNodeRenderArgs<T>) -> Element {
    let leading = match args.expanded {
        Some(expanded) => rsx! {
            Icon {
                variant: "standard",
                size: "xs",
                color: "muted.6",
                "data-tree-chevron": true,
                states: States::new().with("expanded", expanded),
                ChevronRightIcon {}
            }
        },
        None => rsx! {
            Box { framework_sx: &DEFAULT_RENDER_LEADING_SPACER_SX }
        },
    };

    rsx! {
        Box {
            framework_sx: &DEFAULT_RENDER_ROW_SX,
            {leading}
            "{args.data.tree_label()}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nodes() -> Vec<TreeNode<&'static str>> {
        vec![
            TreeNode::new("a", "Alpha").children(vec![TreeNode::new("a1", "Alpha One")]),
            TreeNode::new("b", "Beta"),
        ]
    }

    #[test]
    fn cloning_one_erasure_stays_equal() {
        let erased = erase_nodes(&nodes());
        assert!(erased == erased.clone());
    }

    #[test]
    fn re_erasing_equal_data_is_not_equal() {
        assert!(erase_nodes(&nodes()) != erase_nodes(&nodes()));
    }

    #[test]
    fn a_changed_child_breaks_equality() {
        let erased = erase_nodes(&nodes());
        let mut changed = erased.clone();
        changed[0].children[0].label = "Alpha Two".to_string();
        assert!(erased != changed);
    }
}
