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
/// (whether it's currently expanded/selected).
#[derive(Clone, PartialEq)]
pub struct TreeNodeRenderArgs<T> {
    pub id: String,
    pub data: T,
    /// `None` for a leaf (no children, no chevron, no `aria-expanded`).
    pub expanded: Option<bool>,
    pub selected: bool,
    pub disabled: bool,
}
