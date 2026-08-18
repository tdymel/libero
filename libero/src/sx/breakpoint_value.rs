use crate::tokens::Size;

use super::ThemeAwareValue;

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct BreakpointValue {
    values: Vec<(Size, ThemeAwareValue)>,
}

impl BreakpointValue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, size: Size, value: impl Into<ThemeAwareValue>) -> Self {
        self.values.retain(|(existing, _)| existing != &size);
        self.values.push((size, value.into()));
        self
    }

    pub fn xs(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with(Size::Xs, value)
    }

    pub fn sm(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with(Size::Sm, value)
    }

    pub fn md(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with(Size::Md, value)
    }

    pub fn lg(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with(Size::Lg, value)
    }

    pub fn xl(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with(Size::Xl, value)
    }

    pub(crate) fn values(&self) -> &[(Size, ThemeAwareValue)] {
        &self.values
    }
}

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
