/// Which axes show a scrollbar / allow overflow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollAxis {
    Vertical,
    Horizontal,
    Both,
    None,
}

impl Default for ScrollAxis {
    fn default() -> Self {
        Self::Vertical
    }
}

impl From<&str> for ScrollAxis {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "horizontal" => Self::Horizontal,
            "both" => Self::Both,
            "none" => Self::None,
            _ => Self::Vertical,
        }
    }
}

impl From<String> for ScrollAxis {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

/// When the scrollbar is actually visible - `Scroll` is treated the same as
/// `Hover` (no idle-timeout primitive exists in this codebase yet).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarVisibility {
    Always,
    Hover,
    Hidden,
    Scroll,
}

impl Default for ScrollbarVisibility {
    fn default() -> Self {
        Self::Always
    }
}

impl From<&str> for ScrollbarVisibility {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "hover" => Self::Hover,
            "hidden" => Self::Hidden,
            "scroll" => Self::Scroll,
            _ => Self::Always,
        }
    }
}

impl From<String> for ScrollbarVisibility {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

/// Maps directly to the CSS `scrollbar-width` keyword.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarSize {
    Thin,
    Auto,
}

impl Default for ScrollbarSize {
    fn default() -> Self {
        Self::Thin
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

impl From<&str> for ScrollbarSize {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "auto" => Self::Auto,
            _ => Self::Thin,
        }
    }
}

impl From<String> for ScrollbarSize {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollAreaDefaults {
    pub scrollbars: ScrollAxis,
    pub visibility: ScrollbarVisibility,
    pub size: ScrollbarSize,
}
