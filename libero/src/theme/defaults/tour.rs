/// Theme defaults for `use_tour`, set on [`Theme`](crate::theme::Theme).
/// Pixels: the hole is measured arithmetic; the card's gap is [`PopoverDefaults::gap`](super::PopoverDefaults).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TourDefaults {
    /// Room between a step's target and the edge of its hole.
    pub padding: f64,
    /// The hole's corner radius.
    pub radius: f64,
    /// The card's widest.
    pub max_width: f64,
}

impl TourDefaults {
    pub const DEFAULT: Self = Self {
        padding: 6.0,
        radius: 4.0,
        max_width: 360.0,
    };
}
