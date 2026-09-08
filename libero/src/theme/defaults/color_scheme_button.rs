use crate::theme::{Color, Variant};

/// What a [`ColorSchemeButton`](crate::components::ColorSchemeButton)
/// announces itself with: one place to translate. Each names what a press
/// *does*, not what is on screen - a screen reader user cannot see the glyph
/// it swaps. Split from the chrome beside it for the
/// [`BurgerLabels`](crate::theme::BurgerLabels) reason.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorSchemeButtonLabels {
    /// Names the button while the dark scheme is showing.
    pub to_light: &'static str,
    /// Names the button while the light scheme is showing.
    pub to_dark: &'static str,
}

impl ColorSchemeButtonLabels {
    pub const ENGLISH: Self = Self {
        to_light: "Switch to the light theme",
        to_dark: "Switch to the dark theme",
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorSchemeButtonDefaults {
    /// `Outlined`, so it reads as a control with a box of its own rather
    /// than a bare glyph in a header - Mantine's `default` action icon.
    pub variant: Variant,
    /// `Muted` keeps the border quiet: the accent belongs to the page, not to
    /// the chrome around it.
    pub color: Color,
    pub labels: ColorSchemeButtonLabels,
}

impl ColorSchemeButtonDefaults {
    pub const DEFAULT: Self = Self {
        variant: Variant::Outlined,
        color: Color::Muted,
        labels: ColorSchemeButtonLabels::ENGLISH,
    };
}
