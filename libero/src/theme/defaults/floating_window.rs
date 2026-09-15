use crate::theme::{Placement, Size};

/// Plain values read from Rust: a window's chrome comes from `Paper`, and its
/// geometry is per instance, so nothing here is a CSS var.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FloatingWindowDefaults {
    /// Where a window first appears, until it is dragged.
    pub placement: Placement,
    pub radius: Size,
    pub shadow: Size,
    /// Pixels an arrow key moves the window by. Shift moves one.
    pub move_step: u16,
    /// Pixels an arrow key resizes it by. Shift resizes by one.
    pub resize_step: u16,
}

impl FloatingWindowDefaults {
    pub const DEFAULT: Self = Self {
        placement: Placement::CenterCenter,
        radius: Size::Md,
        shadow: Size::Xl,
        move_step: 10,
        resize_step: 10,
    };
}
