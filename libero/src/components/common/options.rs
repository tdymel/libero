//! A finite, ordered set of choices, so a component's choices are the caller's own type.

use dioxus::prelude::*;

/// A value one choice stands for; `#[derive(Options)]` writes it for a unit enum.
///
/// `PartialEq` is identity: a type whose fields change must compare its id alone,
/// or the selection vanishes.
pub trait Options: Clone + PartialEq + 'static {
    /// Every choice, in order. None by default: a runtime set comes through the component's prop.
    fn options() -> &'static [Self]
    where
        Self: Sized,
    {
        &[]
    }

    /// The choice's visible text and accessible name.
    fn label(&self) -> String;

    /// What a form posts for this choice. The derive uses the variant's name, so a
    /// translated label never changes it.
    fn value(&self) -> String {
        self.label()
    }
}

/// A runtime set, listed by the component rather than the type.
impl Options for String {
    fn label(&self) -> String {
        self.clone()
    }
}

/// What a choice is called, and optionally how it is drawn. A bare string is both.
#[derive(Clone, PartialEq)]
pub struct OptionLabel {
    pub(crate) name: String,
    pub(crate) content: Option<Element>,
}

impl OptionLabel {
    /// Drawn `content` with a plain-text accessible `name`.
    pub fn rich(name: impl Into<String>, content: Element) -> Self {
        Self {
            name: name.into(),
            content: Some(content),
        }
    }

    /// The accessible name - always plain text, even when the label is rich.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// What to draw: the rich content, or the name as text.
    pub fn render(&self) -> Element {
        match &self.content {
            Some(content) => content.clone(),
            None => {
                let name = &self.name;
                rsx! { "{name}" }
            }
        }
    }
}

impl From<String> for OptionLabel {
    fn from(name: String) -> Self {
        Self {
            name,
            content: None,
        }
    }
}

impl From<&str> for OptionLabel {
    fn from(name: &str) -> Self {
        Self::from(name.to_string())
    }
}
