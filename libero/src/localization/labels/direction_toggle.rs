/// A `DirectionToggle`'s name, which says what a press does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirectionToggleLabels {
    /// Names the button while the text runs left to right.
    pub to_rtl: &'static str,
    /// Names the button while the text runs right to left.
    pub to_ltr: &'static str,
}

impl DirectionToggleLabels {
    pub const ENGLISH: Self = Self {
        to_rtl: "Switch to right-to-left text",
        to_ltr: "Switch to left-to-right text",
    };

    pub const GERMAN: Self = Self {
        to_rtl: "Zu Text von rechts nach links wechseln",
        to_ltr: "Zu Text von links nach rechts wechseln",
    };
}
