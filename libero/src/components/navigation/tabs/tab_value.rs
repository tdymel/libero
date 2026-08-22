//! What a `Tabs` strip is made of: a finite, ordered set of values. The
//! component is generic over it, so the tabs are the caller's own domain type
//! rather than a bag of strings.

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
