use crate::str_enum::str_enum;

str_enum! {
    /// Which axes show a scrollbar / allow overflow.
    pub enum ScrollAxis {
        #[default]
        Vertical = "vertical",
        Horizontal = "horizontal",
        Both = "both",
        None = "none",
    }
}

str_enum! {
    /// When the scrollbar is actually visible - `Scroll` is treated the same as
    /// `Hover` (no idle-timeout primitive exists in this codebase yet).
    pub enum ScrollbarVisibility {
        #[default]
        Always = "always",
        Hover = "hover",
        Hidden = "hidden",
        Scroll = "scroll",
    }
}

str_enum! {
    /// Maps directly to the CSS `scrollbar-width` keyword.
    pub enum ScrollbarSize {
        #[default]
        Thin = "thin",
        Auto = "auto",
    }
}

impl ScrollbarSize {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Thin => "thin",
            Self::Auto => "auto",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollAreaDefaults {
    pub scrollbars: ScrollAxis,
    pub visibility: ScrollbarVisibility,
    pub size: ScrollbarSize,
}
