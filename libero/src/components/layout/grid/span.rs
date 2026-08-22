use crate::{components::Input, str_enum::str_enum, tokens::Size};

str_enum! {
    /// An item's width, in twelfths of its zone.
    #[state_prefix = "span"]
    pub enum GridSpan {
        #[default]
        Full = "full",
        ThreeQuarters = "three-quarters",
        TwoThirds = "two-thirds",
        Half = "half",
        Third = "third",
        Quarter = "quarter",
        Sixth = "sixth",
        Twelfth = "twelfth",
    }
}

crate::components::common::input_from_str!(GridSpan);

impl From<GridSpan> for Input<GridSpan> {
    fn from(value: GridSpan) -> Self {
        Input::Value(value)
    }
}

impl GridSpan {
    /// Columns of the zone's twelve.
    pub const fn columns(self) -> u8 {
        match self {
            Self::Full => 12,
            Self::ThreeQuarters => 9,
            Self::TwoThirds => 8,
            Self::Half => 6,
            Self::Third => 4,
            Self::Quarter => 3,
            Self::Sixth => 2,
            Self::Twelfth => 1,
        }
    }
}

/// A span that changes with the *zone's* width, not the viewport's - built by
/// [`sp`] and resolved against the zone's query container.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpanValue {
    base: GridSpan,
    breakpoints: Vec<(Size, GridSpan)>,
}

impl SpanValue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Below every breakpoint. `Full` unless set.
    pub fn base(mut self, span: GridSpan) -> Self {
        self.base = span;
        self
    }

    pub fn with(mut self, size: Size, span: GridSpan) -> Self {
        self.breakpoints.retain(|(existing, _)| existing != &size);
        self.breakpoints.push((size, span));
        self
    }

    pub fn xs(self, span: GridSpan) -> Self {
        self.with(Size::Xs, span)
    }

    pub fn sm(self, span: GridSpan) -> Self {
        self.with(Size::Sm, span)
    }

    pub fn md(self, span: GridSpan) -> Self {
        self.with(Size::Md, span)
    }

    pub fn lg(self, span: GridSpan) -> Self {
        self.with(Size::Lg, span)
    }

    pub fn xl(self, span: GridSpan) -> Self {
        self.with(Size::Xl, span)
    }

    pub(crate) fn base_span(&self) -> GridSpan {
        self.base
    }

    pub(crate) fn breakpoints(&self) -> &[(Size, GridSpan)] {
        &self.breakpoints
    }
}

/// A zone-relative span: `sp().base(Full).md(Half)`.
pub fn sp() -> SpanValue {
    SpanValue::new()
}

impl From<GridSpan> for SpanValue {
    fn from(span: GridSpan) -> Self {
        Self::new().base(span)
    }
}

impl From<GridSpan> for Input<SpanValue> {
    fn from(span: GridSpan) -> Self {
        Input::Value(span.into())
    }
}

impl From<SpanValue> for Input<SpanValue> {
    fn from(value: SpanValue) -> Self {
        Input::Value(value)
    }
}

impl From<&str> for SpanValue {
    fn from(value: &str) -> Self {
        GridSpan::from(value).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_span_is_a_base_with_no_breakpoints() {
        let value = SpanValue::from(GridSpan::Half);

        assert_eq!(value.base_span(), GridSpan::Half);
        assert!(value.breakpoints().is_empty());
    }

    #[test]
    fn re_declaring_a_size_replaces_it() {
        let value = sp()
            .base(GridSpan::Full)
            .md(GridSpan::Half)
            .md(GridSpan::Third);

        assert_eq!(value.breakpoints(), [(Size::Md, GridSpan::Third)]);
    }
}
