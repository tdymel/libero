use crate::theme::Size;

/// Theme defaults for `Cascader`, set on [`Theme`](crate::theme::Theme).
///
/// Frame and dropdown come from `FieldDefaults` and `ComboboxDefaults`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CascaderDefaults {
    pub size: Size,
    pub radius: Size,
    /// Each column's width, written onto its `style` (no CSS var).
    pub column_width: &'static str,
}

impl CascaderDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        radius: Size::Sm,
        column_width: "220px",
    };
}
