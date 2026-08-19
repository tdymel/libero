use std::{any::Any, rc::Rc};

use dioxus::prelude::*;

use crate::{
    components::{Box, Icon},
    sx::{StaticSx, sx},
    theme::{ICON_SIZE, Size},
};

/// The one thing `Tree` needs from a user-defined `data: T`: text to match
/// typeahead against, and to render when there's no `render_node`.
pub trait TreeLabel {
    fn tree_label(&self) -> String;
}

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

/// Type-erased mirror of `TreeNode<T>`. The tree machinery is built against
/// this so it compiles once instead of once per `T`; `label` is precomputed
/// at erasure time, so nothing downstream needs `T: TreeLabel`.
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

/// Type-erased mirror of [`TreeNodeRenderArgs<T>`]. `Tree<T>` wraps the typed
/// callback in one that downcasts `data` back to `T`, so a downcast is the
/// only per-`T` cost.
pub(super) struct TreeNodeRenderArgsErased {
    pub id: String,
    pub data: Rc<dyn Any>,
    pub expanded: Option<bool>,
    pub disabled: bool,
    pub tabindex: &'static str,
    pub depth: usize,
}

/// `Rc<dyn Fn>` wrapper so `TreeRowProps` derives `Clone`/`PartialEq` without
/// being generic over `T`. Always equal: the closure instance doesn't change a
/// row's output for given args, so it shouldn't gate memoization.
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

/// Passed to `render_node` per visible row: the node's data plus live state a
/// static `TreeNode<T>` can't know. There is no "selected" - a rendered link
/// knows its own active state; anything else tracks selection itself.
#[derive(Clone, PartialEq)]
pub struct TreeNodeRenderArgs<T> {
    pub id: String,
    pub data: T,
    /// `None` for a leaf (no children, no chevron, no `aria-expanded`).
    pub expanded: Option<bool>,
    pub disabled: bool,
    /// Apply to any interactive element your content renders. The row is
    /// already the roving tab stop; without this the link/button adds a
    /// second one that arrow-key navigation never moves.
    pub tabindex: &'static str,
    /// 0 at top level. Exposed so `render_node` can take indentation over
    /// entirely - zero `TreeProps::indent` via `sx` and offset from this.
    pub depth: usize,
}

// Chevron-width, so leaves line up with their branch siblings. Reserved only
// here, by the render that draws the chevron - not by `Tree`.
static DEFAULT_RENDER_LEADING_SPACER_SX: StaticSx =
    StaticSx::new(|| sx().flex_shrink("0").width(ICON_SIZE.value(Size::Xs)));

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

/// `render_node`'s default: a chevron for a branch, a matching spacer for a
/// leaf, then `tree_label()`. `pub` so a custom `render_node` can fall back to
/// it for some rows rather than reimplementing it.
///
/// The chevron is this function's concern, not `Tree`'s. It also brings its
/// own vertical padding, since `TreeRow`'s wrapper deliberately has none.
pub fn default_tree_render<T: TreeLabel>(args: TreeNodeRenderArgs<T>) -> Element {
    let leading = match args.expanded {
        Some(expanded) => rsx! {
            Icon {
                variant: "transparent",
                size: "xs",
                color: "grey.6",
                sx: sx().flex_shrink("0")
                    .transition("transform 120ms ease")
                    .transform(if expanded { "rotate(90deg)" } else { "rotate(0deg)" }),
                {chevron_svg()}
            }
        },
        None => rsx! {
            Box { framework_sx: &DEFAULT_RENDER_LEADING_SPACER_SX }
        },
    };

    rsx! {
        Box {
            sx: sx().display("flex").align_items("center").gap("6px").padding("6px 0"),
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
