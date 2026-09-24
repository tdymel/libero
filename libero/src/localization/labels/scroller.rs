/// A `Scroller`'s two controls. "Backward"/"forward" rather than
/// "left"/"right", so the names do not lie under a right-to-left page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollerLabels {
    pub backward: &'static str,
    pub forward: &'static str,
}

impl ScrollerLabels {
    pub const ENGLISH: Self = Self {
        backward: "Scroll backward",
        forward: "Scroll forward",
    };

    pub const GERMAN: Self = Self {
        backward: "Zurückscrollen",
        forward: "Vorwärtsscrollen",
    };
}
