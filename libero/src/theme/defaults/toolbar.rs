/// Theme defaults for `Toolbar`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolbarDefaults {
    /// Whether the arrow keys wrap at the ends.
    pub loop_focus: bool,
}

impl ToolbarDefaults {
    pub const DEFAULT: Self = Self { loop_focus: true };
}
