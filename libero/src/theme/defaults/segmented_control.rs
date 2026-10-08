use crate::theme::Variant;
use crate::tokens::Color;

/// Theme defaults for `SegmentedControl`, set on [`Theme`](crate::theme::Theme).
/// Size and radius follow `theme.button`; the chrome is its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SegmentedControlDefaults {
    /// The chrome a segmented control takes when a call site names none.
    pub variant: Variant,
    /// The colour a segmented control takes when a call site names none.
    pub color: Color,
}

impl SegmentedControlDefaults {
    pub const DEFAULT: Self = Self {
        variant: Variant::Filled,
        color: Color::Primary,
    };
}
