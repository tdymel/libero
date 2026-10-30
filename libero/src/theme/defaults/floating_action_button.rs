use crate::theme::{Placement, Size, Variant};

/// Theme defaults for `FloatingActionButton`, set on [`Theme`](crate::theme::Theme).
/// Plain values read in Rust, no vars of their own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FloatingActionButtonDefaults {
    pub variant: Variant,
    /// A step of `Button`'s heights: `xl` is 60px.
    pub size: Size,
    /// `xxl` rounds the icon-only button into a circle and the extended one into a pill.
    pub radius: Size,
    pub placement: Placement,
    /// Distance from the placed edges, on the spacing scale.
    pub offset: Size,
    /// The resting shadow, on the elevation scale.
    pub elevation: Size,
}

impl FloatingActionButtonDefaults {
    pub const DEFAULT: Self = Self {
        variant: Variant::Filled,
        size: Size::Xl,
        radius: Size::Xxl,
        placement: Placement::BottomEnd,
        offset: Size::Md,
        elevation: Size::Md,
    };
}
