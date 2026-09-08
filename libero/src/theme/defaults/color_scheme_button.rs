use crate::theme::{Color, Variant};

/// What a [`ColorSchemeButton`](crate::components::ColorSchemeButton)
/// announces itself with: one place to translate. Split from the chrome
/// beside it for the [`BurgerLabels`](crate::theme::BurgerLabels) reason.
///
/// The three `to_*` name what a press *does*, as the glyph beside them shows
/// it - a screen reader user cannot see the glyph.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorSchemeButtonLabels {
    /// Names the button when a press pins the light scheme.
    pub to_light: &'static str,
    /// Names the button when a press pins the dark scheme.
    pub to_dark: &'static str,
    /// Names the button when a press hands the choice back to the platform.
    pub to_system: &'static str,
    /// Names the pair of buttons, when a theme picker makes it a pair.
    pub group: &'static str,
    /// Names the picker's own button.
    pub picker: &'static str,
    /// Names the group of theme sets inside the picker's menu.
    pub themes: &'static str,
}

impl ColorSchemeButtonLabels {
    pub const ENGLISH: Self = Self {
        to_light: "Switch to the light theme",
        to_dark: "Switch to the dark theme",
        to_system: "Follow the system theme",
        group: "Theme",
        picker: "Choose a theme",
        themes: "Themes",
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
