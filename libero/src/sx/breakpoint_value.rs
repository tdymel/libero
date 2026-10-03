use crate::tokens::Size;

use super::ThemeAwareValue;

/// One value per breakpoint, for a responsive `sx` property. Setting a size twice keeps the last.
///
/// ```
/// # use libero::sx::{bp, sx};
/// let responsive = sx().padding(bp().xs("sm").md("lg"));
/// ```
///
/// Docs: <https://libero-ui.dev/about/styling>
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct BreakpointValue {
    values: Vec<(Size, ThemeAwareValue)>,
}

impl BreakpointValue {
    /// No value yet, the same as [`bp()`].
    pub fn new() -> Self {
        Self::default()
    }

    /// The value from `size`'s breakpoint up. A nested `bp()` is ignored with a warning.
    pub fn with(mut self, size: Size, value: impl Into<ThemeAwareValue>) -> Self {
        let value = value.into();
        // Expansion is one level deep, so a nested one could only render as a panic.
        if matches!(value, ThemeAwareValue::BreakpointValue(_)) {
            crate::utils::warn(
                "bp(): a breakpoint value inside another is ignored; set each size on one bp().",
            );
            return self;
        }
        self.values.retain(|(existing, _)| existing != &size);
        self.values.push((size, value));
        self
    }

    /// [`with`](Self::with) at [`Size::Xs`].
    pub fn xs(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with(Size::Xs, value)
    }

    /// [`with`](Self::with) at [`Size::Sm`].
    pub fn sm(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with(Size::Sm, value)
    }

    /// [`with`](Self::with) at [`Size::Md`].
    pub fn md(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with(Size::Md, value)
    }

    /// [`with`](Self::with) at [`Size::Lg`].
    pub fn lg(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with(Size::Lg, value)
    }

    /// [`with`](Self::with) at [`Size::Xl`].
    pub fn xl(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with(Size::Xl, value)
    }

    pub(crate) fn values(&self) -> &[(Size, ThemeAwareValue)] {
        &self.values
    }
}

/// Starts an empty [`BreakpointValue`].
pub fn bp() -> BreakpointValue {
    BreakpointValue::new()
}

#[cfg(test)]
mod tests {
    use crate::tokens::Size;

    use super::*;

    #[test]
    fn breakpoint_value_overwrite_same_size_last_wins() {
        let value = bp().sm("one").xl("two").sm("three");

        assert_eq!(value.values().len(), 2);
        assert!(
            value
                .values()
                .contains(&(Size::Sm, ThemeAwareValue::from("three")))
        );
        assert!(
            value
                .values()
                .contains(&(Size::Xl, ThemeAwareValue::from("two")))
        );
    }
}
