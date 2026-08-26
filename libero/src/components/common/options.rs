//! What a strip of choices is made of: a finite, ordered set of values. A
//! component is generic over it, so the choices are the caller's own domain
//! type rather than a bag of strings.
//!
//! `String` is the only impl libero ships, and it lists nothing - a runtime
//! set has no canonical options, so it arrives through the component's own
//! override prop instead.

use dioxus::prelude::*;

/// A value one choice stands for.
///
/// `#[derive(Options)]` writes this for an enum of unit variants: the
/// variants in declaration order are the choices, each one's name its label.
pub trait Options: Clone + PartialEq + 'static {
    /// Every choice, in order. Empty means the set is not known statically -
    /// the component's own override prop supplies it.
    fn options() -> &'static [Self]
    where
        Self: Sized;

    /// The choice's visible text and accessible name. Override it per
    /// instance with the component's `label` prop - that one runs during
    /// render, so it can read a locale from context.
    fn label(&self) -> String;
}

/// A runtime set, listed by the component rather than the type.
impl Options for String {
    fn options() -> &'static [Self] {
        &[]
    }

    fn label(&self) -> String {
        self.clone()
    }
}

/// What a choice is called, and optionally how it is drawn.
///
/// A bare string is both (`"Konto".into()`); [`OptionLabel::rich`] takes the
/// two apart for an icon or a badge - and asks for the name anyway, because
/// the rsx is what a screen reader cannot use.
#[derive(Clone)]
pub struct OptionLabel {
    pub(crate) name: String,
    pub(crate) content: Option<Element>,
}

impl OptionLabel {
    pub fn rich(name: impl Into<String>, content: Element) -> Self {
        Self {
            name: name.into(),
            content: Some(content),
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
