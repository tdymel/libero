use std::fmt::{self, Display};

use crate::tokens::CssVar;

/// CSS custom properties (`--name:value;`), built up incrementally and
/// rendered directly into an element's `style` attribute - `Display`/
/// `to_string()` is its string representation. Lets a component's shared,
/// cached static class reference a per-instance value via `var(--name,
/// fallback)`, instead of generating a whole new class/stylesheet entry for
/// every distinct value combination a caller might pass.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Variables(Vec<(CssVar, String)>);

impl Variables {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets `name` to `value`, skipping `None`/empty ones - safe to chain
    /// directly off an optional per-instance override.
    pub fn with(mut self, name: CssVar, value: impl Into<Option<String>>) -> Self {
        self.0.retain(|(existing, _)| existing != &name);
        if let Some(value) = value.into().filter(|value| !value.is_empty()) {
            self.0.push((name, value));
        }
        self
    }

    /// Adds every entry from `other` on top of `self` (each still replaces
    /// any existing entry of the same name) - for a component that forwards
    /// a caller-supplied `Variables` alongside its own internally-computed
    /// ones (e.g. `Dialog` forwarding `Drawer`'s).
    pub fn merge(mut self, other: Self) -> Self {
        for (name, value) in other.0 {
            self = self.with(name, value);
        }
        self
    }
}

impl Display for Variables {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (name, value) in &self.0 {
            write!(f, "{}:{value};", name.name())?;
        }
        Ok(())
    }
}

pub fn variables() -> Variables {
    Variables::new()
}
