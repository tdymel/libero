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
