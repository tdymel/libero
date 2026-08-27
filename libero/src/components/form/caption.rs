use dioxus::prelude::*;

/// The text around a field's control: its label, its description, its helper.
///
/// `Text` is the case the a11y wiring can use - it gets an id and joins
/// `aria-describedby`. `Node` is markup the caller built, so it renders and is
/// styled like the others, but names nothing: a caller passing markup owns its
/// own a11y, the same rule a caller-supplied `aria-describedby` follows.
#[derive(Clone, Default, PartialEq)]
pub enum Caption {
    #[default]
    None,
    Text(String),
    Node(Element),
}

impl Caption {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// The plain text, if this caption is text. `None` for markup - which is
    /// what keeps a `Node` out of `aria-describedby`.
    pub fn text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            Self::None | Self::Node(_) => None,
        }
    }
}

impl std::fmt::Debug for Caption {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => formatter.write_str("None"),
            Self::Text(text) => formatter.debug_tuple("Text").field(text).finish(),
            Self::Node(_) => formatter.write_str("Node(..)"),
        }
    }
}

impl From<&str> for Caption {
    fn from(text: &str) -> Self {
        Self::Text(text.to_string())
    }
}

impl From<String> for Caption {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}

impl From<Element> for Caption {
    fn from(element: Element) -> Self {
        Self::Node(element)
    }
}

impl From<Option<String>> for Caption {
    fn from(text: Option<String>) -> Self {
        match text {
            Some(text) => Self::Text(text),
            None => Self::None,
        }
    }
}
