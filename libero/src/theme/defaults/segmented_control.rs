use crate::theme::Variant;

/// What `SegmentedControl` does not borrow from `ButtonDefaults`. Its size
/// and radius still follow `theme.button`; its chrome is its own, so a
/// project can theme tonal buttons without changing its segmented controls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SegmentedControlDefaults {
    /// The chrome a segmented control takes when a call site names none.
    pub variant: Variant,
}

impl SegmentedControlDefaults {
    pub const DEFAULT: Self = Self {
        variant: Variant::Filled,
    };
}
