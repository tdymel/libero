/// A `Marquee`'s pause toggle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarqueeLabels {
    /// The toggle's name in both states: `aria-pressed` carries the state.
    pub pause: &'static str,
}

impl MarqueeLabels {
    pub const ENGLISH: Self = Self { pause: "Pause" };
    pub const GERMAN: Self = Self { pause: "Anhalten" };
}
