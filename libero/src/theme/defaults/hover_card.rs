use crate::theme::{Align, Side, Size};

/// Theme defaults for `HoverCard`, set on [`Theme`](crate::theme::Theme).
///
/// Plain values read from Rust, no CSS vars: the surface is `Paper`'s.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoverCardDefaults {
    /// The preferred side; the card flips when that side has no room.
    pub side: Side,
    pub align: Align,
    /// Milliseconds the pointer must rest on the trigger before the card opens.
    pub open_delay: u32,
    /// Milliseconds after the pointer leaves: the time to cross the gap, so `0`
    /// makes the card unreachable by pointer.
    pub close_delay: u32,
    pub radius: Size,
    pub shadow: Size,
}

impl HoverCardDefaults {
    pub const DEFAULT: Self = Self {
        side: Side::Bottom,
        align: Align::Start,
        open_delay: 0,
        close_delay: 150,
        radius: Size::Sm,
        shadow: Size::Md,
    };
}
