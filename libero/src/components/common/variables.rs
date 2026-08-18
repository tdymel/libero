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

#[cfg(test)]
mod tests {
    use super::*;

    const COLOR: CssVar = CssVar::new("--lsx-test-color");

    #[test]
    fn renders_as_style_attribute_declarations() {
        let variables = variables()
            .with(COLOR, "red".to_string())
            .with(CssVar::new("--lsx-test-size"), "1rem".to_string());

        assert_eq!(
            variables.to_string(),
            "--lsx-test-color:red;--lsx-test-size:1rem;"
        );
    }

    #[test]
    fn absent_and_empty_values_are_skipped() {
        let variables = variables()
            .with(COLOR, None)
            .with(CssVar::new("--lsx-test-size"), String::new());

        assert_eq!(variables.to_string(), "");
    }

    #[test]
    fn setting_a_variable_to_none_clears_an_earlier_value() {
        let variables = variables().with(COLOR, "red".to_string()).with(COLOR, None);

        assert_eq!(variables.to_string(), "");
    }

    /// A borrowed and an owned `CssVar` naming the same property are one
    /// variable - `CssVar`'s equality is by name, not by representation.
    #[test]
    fn a_static_and_an_owned_name_are_the_same_variable() {
        let owned = CssVar::parse("--lsx-test-color").expect("a valid custom property name");
        let variables = variables()
            .with(COLOR, "red".to_string())
            .with(owned, "blue".to_string());

        assert_eq!(variables.to_string(), "--lsx-test-color:blue;");
    }

    #[test]
    fn merge_lets_the_other_side_win_and_keeps_the_rest() {
        let size = CssVar::new("--lsx-test-size");
        let base = variables()
            .with(COLOR, "red".to_string())
            .with(size, "1rem".to_string());
        let merged = base.merge(variables().with(COLOR, "blue".to_string()));

        assert_eq!(
            merged.to_string(),
            "--lsx-test-size:1rem;--lsx-test-color:blue;"
        );
    }
}
