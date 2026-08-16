use dioxus::prelude::*;

use crate::{components::Box, sx::sx};

/// Text `Tree` matches against for typeahead and falls back to rendering
/// when no `render_node` is given - the one thing `Tree` needs to know about
/// an otherwise fully user-defined `data: T`.
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

    pub(crate) fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

/// Passed to `render_node` for each visible row - the node's own data plus
/// the live state that a static `TreeNode<T>` value can't know on its own
/// (whether it's currently expanded). `Tree` doesn't have a notion of
/// "selected" - if `render_node` renders a real link, the link itself
/// already knows whether it's active (e.g. `NavLink` compares against the
/// current route); for anything else, track selection as your own state.
#[derive(Clone, PartialEq)]
pub struct TreeNodeRenderArgs<T> {
    pub id: String,
    pub data: T,
    /// `None` for a leaf (no children, no chevron, no `aria-expanded`).
    pub expanded: Option<bool>,
    pub disabled: bool,
    /// Apply this to whatever real interactive element (a link, a button)
    /// your own content renders - it suppresses that element's *native*
    /// tab stop, since `Tree` already made the row itself the one roving
    /// tab stop. Without it you'd get two independent tab stops per row:
    /// the tree's own roving one, and the link/button's native implicit
    /// one - and arrow-key navigation only ever moves the tree's.
    pub tabindex: &'static str,
}

/// `Tree`'s own fallback row content - just `tree_label()` as plain text.
/// This is what `render_node` defaults to when not given, and it's `pub` so
/// a custom `render_node` can selectively fall back to it too (e.g. use this
/// for branches, something custom for leaves) instead of reimplementing it.
///
/// Provides its own vertical padding - `TreeRow`'s content wrapper
/// deliberately has none (so a bordered, full-height `render_node` like a
/// `NavLink` can read as continuous between rows), so plain content needs
/// to bring its own breathing room.
pub fn default_tree_render<T: TreeLabel>(args: TreeNodeRenderArgs<T>) -> Element {
    rsx! {
        Box { sx: sx().padding("6px 0"), "{args.data.tree_label()}" }
    }
}
