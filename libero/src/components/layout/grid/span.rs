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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpanValue {
    base: GridSpan,
    breakpoints: [Option<GridSpan>; Size::ALL.len()],
}

impl SpanValue {
    pub const fn new() -> Self {
        Self {
            base: GridSpan::Full,
            breakpoints: [None; Size::ALL.len()],
        }
    }

    /// Below every breakpoint. `Full` unless set.
    pub const fn base(mut self, span: GridSpan) -> Self {
        self.base = span;
        self
    }

    pub const fn with(mut self, size: Size, span: GridSpan) -> Self {
        self.breakpoints[size.index()] = Some(span);
        self
    }

    pub const fn xs(self, span: GridSpan) -> Self {
        self.with(Size::Xs, span)
    }

    pub const fn sm(self, span: GridSpan) -> Self {
        self.with(Size::Sm, span)
    }

    pub const fn md(self, span: GridSpan) -> Self {
        self.with(Size::Md, span)
    }

    pub const fn lg(self, span: GridSpan) -> Self {
        self.with(Size::Lg, span)
    }

    pub const fn xl(self, span: GridSpan) -> Self {
        self.with(Size::Xl, span)
    }

    pub(crate) const fn base_span(&self) -> GridSpan {
        self.base
    }

    /// Ascending, so the widest matching query wins the cascade.
    pub(crate) fn breakpoints(&self) -> impl Iterator<Item = (Size, GridSpan)> {
        Size::ALL
            .into_iter()
            .zip(self.breakpoints)
            .filter_map(|(size, span)| span.map(|span| (size, span)))
    }
}

/// A zone-relative span: `sp().base(Full).md(Half)`.
pub const fn sp() -> SpanValue {
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
        assert_eq!(value.breakpoints().count(), 0);
    }

    #[test]
    fn re_declaring_a_size_replaces_it() {
        let value = sp()
            .base(GridSpan::Full)
            .md(GridSpan::Half)
            .md(GridSpan::Third);

        assert_eq!(
            value.breakpoints().collect::<Vec<_>>(),
            [(Size::Md, GridSpan::Third)]
        );
    }
}
