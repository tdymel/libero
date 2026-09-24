/// What a `ThemeToggle` announces itself with. The three `to_*` name
/// what a press *does*, as the glyph beside them shows it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemeToggleLabels {
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

impl ThemeToggleLabels {
    pub const ENGLISH: Self = Self {
        to_light: "Switch to the light theme",
        to_dark: "Switch to the dark theme",
        to_system: "Follow the system theme",
        group: "Theme",
        picker: "Choose a theme",
        themes: "Themes",
    };

    pub const GERMAN: Self = Self {
        to_light: "Zum hellen Design wechseln",
        to_dark: "Zum dunklen Design wechseln",
        to_system: "Dem Systemdesign folgen",
        group: "Design",
        picker: "Design wählen",
        themes: "Designs",
    };
}
