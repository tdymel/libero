use super::{ColorCss, NamedColorCss};

/// How a [`Color`] names its CSS custom properties, as `(own, contrast)`: a
/// palette color has a 9-shade var per role, black/white have exactly one
/// var each and are each other's contrast.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ColorVars {
    Palette(ColorCss, ColorCss),
    Named(NamedColorCss, NamedColorCss),
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Color {
    Primary,
    Secondary,
    Error,
    Warning,
    Info,
    Success,
    Grey,
    Black,
    White,
}

impl Color {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "primary" => Some(Self::Primary),
            "secondary" => Some(Self::Secondary),
            "error" => Some(Self::Error),
            "warning" => Some(Self::Warning),
            "info" => Some(Self::Info),
            "success" => Some(Self::Success),
            "grey" => Some(Self::Grey),
            "black" => Some(Self::Black),
            "white" => Some(Self::White),
            _ => None,
        }
    }

    /// The single table mapping a color to its CSS var names - every
    /// shade/contrast, name/value lookup goes through this.
    pub(crate) const fn vars(self) -> ColorVars {
        match self {
            Self::Primary => ColorVars::Palette(ColorCss::PRIMARY, ColorCss::PRIMARY_CONTRAST),
            Self::Secondary => {
                ColorVars::Palette(ColorCss::SECONDARY, ColorCss::SECONDARY_CONTRAST)
            }
            Self::Error => ColorVars::Palette(ColorCss::ERROR, ColorCss::ERROR_CONTRAST),
            Self::Warning => ColorVars::Palette(ColorCss::WARNING, ColorCss::WARNING_CONTRAST),
            Self::Info => ColorVars::Palette(ColorCss::INFO, ColorCss::INFO_CONTRAST),
            Self::Success => ColorVars::Palette(ColorCss::SUCCESS, ColorCss::SUCCESS_CONTRAST),
            Self::Grey => ColorVars::Palette(ColorCss::GREY, ColorCss::GREY_CONTRAST),
            Self::Black => ColorVars::Named(NamedColorCss::BLACK, NamedColorCss::WHITE),
            Self::White => ColorVars::Named(NamedColorCss::WHITE, NamedColorCss::BLACK),
        }
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
            Self::Success => "success",
            Self::Grey => "grey",
            Self::Black => "black",
            Self::White => "white",
        }
    }
}
