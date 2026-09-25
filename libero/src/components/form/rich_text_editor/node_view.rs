//! [`NodeViews`]: a caller's component per custom node type, on the view side so the
//! model stays free of Dioxus.

use std::collections::BTreeMap;
use std::fmt;

use dioxus::core::{VComponent, view::ViewExt};
use dioxus::prelude::*;

use super::model::{Attrs, NodeRegistry};

/// What a node view gets: the node and, for a node with content, that content.
#[derive(Props, Clone, PartialEq)]
pub struct NodeViewProps {
    /// The node type's registered name.
    pub name: String,
    pub attrs: Attrs,
    /// The node's editable content, already rendered; empty for atoms. Render it exactly
    /// once: omitting or duplicating it breaks the caret. Mark your own markup around it
    /// `contenteditable: "false"`.
    pub children: Element,
}

/// One component per custom node type, by name. An atom's view is a non-editable
/// island, so its text must name the node for screen readers.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::rich_text::{NodeViewProps, NodeViews, RichTextEditor};
/// #[component]
/// fn Mention(props: NodeViewProps) -> Element {
///     let user = props.attrs.get("user").and_then(|user| user.as_str()).unwrap_or("?");
///     rsx! { span { class: "mention", "@{user}" } }
/// }
///
/// # fn app() -> Element {
/// rsx! {
///     RichTextEditor { label: "Message", nodes: NodeViews::new().with("mention", Mention) }
/// }
/// # }
/// ```
#[derive(Clone, Default)]
pub struct NodeViews {
    views: BTreeMap<String, fn(NodeViewProps) -> Element>,
}

impl NodeViews {
    pub fn new() -> Self {
        Self::default()
    }

    /// Renders nodes named `name` with `view`, replacing an earlier one.
    pub fn with(mut self, name: impl Into<String>, view: fn(NodeViewProps) -> Element) -> Self {
        let name = name.into();
        #[cfg(debug_assertions)]
        if NodeRegistry::BUILTIN
            .iter()
            .any(|(builtin, _)| *builtin == name)
        {
            crate::utils::warn(&format!(
                "NodeViews: \"{name}\" is a built-in node; built-ins do not render through NodeViews yet."
            ));
        }
        self.views.insert(name, view);
        self
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.views.keys().map(String::as_str)
    }

    /// `name`'s view as an element, `None` when none is set.
    pub(crate) fn render(&self, name: &str, attrs: &Attrs, children: Element) -> Option<Element> {
        let view = *self.views.get(name)?;
        let props = NodeViewProps {
            name: name.to_string(),
            attrs: attrs.clone(),
            children,
        };
        Some(Ok(VComponent::new(view, props, "NodeView").into_vnode()))
    }
}

/// Same names drawn by the same functions.
impl PartialEq for NodeViews {
    fn eq(&self, other: &Self) -> bool {
        self.views.len() == other.views.len()
            && self
                .views
                .iter()
                .zip(&other.views)
                .all(|((a, f), (b, g))| a == b && std::ptr::fn_addr_eq(*f, *g))
    }
}

impl fmt::Debug for NodeViews {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.views.keys()).finish()
    }
}
