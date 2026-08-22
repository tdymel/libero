//! What a `Tabs` strip is made of: a finite, ordered set of values. The
//! component is generic over it, so the tabs are the caller's own domain type
//! rather than a bag of strings.

use dioxus::prelude::*;

/// A value one tab stands for.
///
/// `#[derive(TabValue)]` writes this for an enum of unit variants: the
/// variants in declaration order are the tabs, each one's name its label.
pub trait TabValue: Clone + PartialEq + 'static {
    /// Every tab, left to right.
    fn options() -> &'static [Self]
    where
        Self: Sized;

    /// The tab's visible text and accessible name. Override it per instance
    /// with `Tabs`' `label` prop - that one runs during render, so it can
    /// read a locale from context.
    fn label(&self) -> String;
}

/// What a tab is called, and optionally how it is drawn.
///
/// A bare string is both (`"Konto".into()`); [`TabLabel::rich`] takes the two
/// apart for an icon or a badge - and asks for the name anyway, because the
/// rsx is what a screen reader cannot use.
#[derive(Clone)]
pub struct TabLabel {
    pub(crate) name: String,
    pub(crate) content: Option<Element>,
}

impl TabLabel {
    pub fn rich(name: impl Into<String>, content: Element) -> Self {
        Self {
            name: name.into(),
            content: Some(content),
        }
    }
}

impl From<String> for TabLabel {
    fn from(name: String) -> Self {
        Self {
            name,
            content: None,
        }
    }
}

impl From<&str> for TabLabel {
    fn from(name: &str) -> Self {
        Self::from(name.to_string())
    }
}
