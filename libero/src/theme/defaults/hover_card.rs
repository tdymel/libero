use crate::theme::Size;

/// Plain values read from Rust: the card's surface is `Paper`'s, so it
/// publishes no CSS vars of its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoverCardDefaults {
    /// Milliseconds the pointer must rest on the trigger before the card opens.
    pub open_delay: u32,
    /// Milliseconds the card waits after the pointer leaves. This is also the
    /// time the pointer has to cross the gap into the card, so `0` makes the
    /// card unreachable by pointer.
    pub close_delay: u32,
    pub radius: Size,
    pub shadow: Size,
}

impl HoverCardDefaults {
    // Mantine's values: a card opens at once and lingers long enough for
    // the pointer to cross the gap into it.
    pub const DEFAULT: Self = Self {
        open_delay: 0,
        close_delay: 150,
        radius: Size::Sm,
        shadow: Size::Md,
    };
}
