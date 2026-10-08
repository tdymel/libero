use dioxus::prelude::*;

/// The text around a field's control: its label, description or helper.
/// Only `Text` joins `aria-describedby`; a `Node` caller owns its own a11y.
/// A `Node` label also stays out of a `Form`'s error summary, which names the
/// field by its `aria_label` then, or not at all.
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

/// An empty or blank text is `None`: no empty `<label>` or description (1525).
impl From<&str> for Caption {
    fn from(text: &str) -> Self {
        Self::from(text.to_string())
    }
}

impl From<String> for Caption {
    fn from(text: String) -> Self {
        match text.trim().is_empty() {
            true => Self::None,
            false => Self::Text(text),
        }
    }
}

impl From<Element> for Caption {
    fn from(element: Element) -> Self {
        Self::Node(element)
    }
}

impl From<Option<String>> for Caption {
    fn from(text: Option<String>) -> Self {
        text.map_or(Self::None, Self::from)
    }
}

impl From<Option<&str>> for Caption {
    fn from(text: Option<&str>) -> Self {
        text.map_or(Self::None, Self::from)
    }
}

impl From<Option<Element>> for Caption {
    fn from(element: Option<Element>) -> Self {
        element.map_or(Self::None, Self::Node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_or_blank_text_is_none() {
        assert!(Caption::from("").is_none());
        assert!(Caption::from("  ".to_string()).is_none());
        assert!(Caption::from(Some(String::new())).is_none());
        assert_eq!(Caption::from("Email").text(), Some("Email"));
    }

    #[test]
    fn an_option_of_str_or_element_takes_none_as_none() {
        assert!(Caption::from(None::<&str>).is_none());
        assert!(Caption::from(Some(" ")).is_none());
        assert_eq!(Caption::from(Some("Email")).text(), Some("Email"));
        assert!(Caption::from(None::<Element>).is_none());
        assert!(matches!(
            Caption::from(Some(VNode::empty())),
            Caption::Node(_)
        ));
    }
}
