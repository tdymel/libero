use std::fmt::{self, Display};

use crate::tokens::CssVar;

/// CSS custom properties on an element's `style` attribute, so a shared class can
/// read a per-instance value through `var(--name)` instead of minting a class per value.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Variables(Vec<(CssVar, String)>);

impl Variables {
    /// No variables.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets `name`, replacing an earlier value. `None` or empty clears it instead,
    /// so an optional override chains directly.
    pub fn with(mut self, name: CssVar, value: impl Into<Option<String>>) -> Self {
        self.0.retain(|(existing, _)| existing != &name);
        if let Some(value) = value.into().filter(|value| !value.is_empty()) {
            self.0.push((name, value));
        }
        self
    }

    /// `other`'s entries win by name.
    pub fn merge(mut self, other: Self) -> Self {
        for (name, value) in other.0 {
            self = self.with(name, value);
        }
        self
    }
}

impl Variables {
    /// Plain concatenation: a `write!` per entry costs more than the rest of the attribute.
    pub(crate) fn render(&self) -> String {
        let mut out = String::with_capacity(
            self.0
                .iter()
                .map(|(name, value)| name.name().len() + value.len() + 2)
                .sum(),
        );
        for (name, value) in &self.0 {
            out.push_str(name.name());
            out.push(':');
            out.push_str(value);
            out.push(';');
        }
        out
    }
}

impl Display for Variables {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render())
    }
}

/// Shorthand for [`Variables::new`].
pub fn variables() -> Variables {
    Variables::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    const COLOR: CssVar = CssVar::new("--lsx-test-color");

    /// `Display` writes `render`'s output.
    #[test]
    fn render_matches_the_display_impl() {
        let variables = variables()
            .with(COLOR, "red".to_string())
            .with(CssVar::new("--lsx-a"), "1rem".to_string());

        assert_eq!(variables.render(), variables.to_string());
        assert_eq!(Variables::new().render(), String::new());
    }

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

    /// `CssVar` equality is by name, not representation.
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
