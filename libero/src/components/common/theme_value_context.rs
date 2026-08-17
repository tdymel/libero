use dioxus::prelude::*;

use crate::theme::Size;

/// Ambient "concept -> current value" bag - what a component instance
/// explicitly set, or inherited from whichever ancestor set one. Cascades
/// through ordinary Dioxus context: read what's there via
/// [`use_theme_value_context`], override the concepts this component
/// actually has an opinion on via the dedicated builder methods below,
/// `.provide()` it again for descendants.
#[derive(Clone, Copy, Default, PartialEq)]
pub struct ThemeValueContext {
    size: Option<Size>,
    radius: Option<Size>,
    // more fields as concepts get wired in - each concretely typed to what
    // that concept actually needs, not forced through one generic type.
}

impl ThemeValueContext {
    pub fn size(mut self, size: Size) -> Self {
        self.size = Some(size);
        self
    }

    pub fn radius(mut self, radius: Size) -> Self {
        self.radius = Some(radius);
        self
    }

    pub fn apply_if<T>(self, value: Option<T>, f: impl FnOnce(Self, T) -> Self) -> Self {
        match value {
            Some(value) => f(self, value),
            None => self,
        }
    }

    pub fn get_size(&self) -> Option<Size> {
        self.size
    }

    pub fn get_radius(&self) -> Option<Size> {
        self.radius
    }

    /// Publishes this as the ambient context for descendants, returning
    /// itself so the same value can be used immediately for this
    /// component's own styling too.
    pub fn provide(self) -> Self {
        provide_context(self);
        self
    }
}

/// Reads the ambient context - whatever the nearest ancestor provided, or
/// a blank one if nothing has yet.
pub fn use_theme_value_context() -> ThemeValueContext {
    try_consume_context::<ThemeValueContext>().unwrap_or_default()
}
