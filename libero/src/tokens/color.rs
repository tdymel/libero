use super::{ColorCss, NamedColorCss, ShadeRamp};

/// A [`Color`]'s custom properties, as `(own, contrast)`. Ink and surface have one
/// each and are each other's contrast.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ColorVars {
    Palette(ColorCss, ColorCss),
    Named(NamedColorCss, NamedColorCss),
}

/// A theme palette colour, written as `"primary"` or with a shade, `"primary.7"`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Color {
    Primary,
    Secondary,
    Error,
    Warning,
    Info,
    Success,
    Neutral,
    Muted,
    /// Text colour: the page's dark end on a light theme, its light end on a dark one.
    Ink,
    /// The page, which every colour role is measured against. Not `Paper`'s card.
    Surface,
}

impl Color {
    /// Muted, neutral and ink/surface walk the neutral mix curve, hues the chromatic one.
    pub(crate) const fn shade_ramp(self) -> ShadeRamp {
        match self {
            Self::Neutral | Self::Muted | Self::Ink | Self::Surface => ShadeRamp::Neutral,
            _ => ShadeRamp::Chromatic,
        }
    }

    /// `None` for an unknown name, which `sx` passes through as raw CSS. So `"white"`
    /// and `"black"` stay fixed CSS keywords: use `"surface"`/`"ink"` to follow the theme.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "primary" => Some(Self::Primary),
            "secondary" => Some(Self::Secondary),
            "error" => Some(Self::Error),
            "warning" => Some(Self::Warning),
            "info" => Some(Self::Info),
            "success" => Some(Self::Success),
            "neutral" => Some(Self::Neutral),
            "muted" => Some(Self::Muted),
            "ink" => Some(Self::Ink),
            "surface" => Some(Self::Surface),
            _ => None,
        }
    }

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
            Self::Neutral => ColorVars::Palette(ColorCss::NEUTRAL, ColorCss::NEUTRAL_CONTRAST),
            Self::Muted => ColorVars::Palette(ColorCss::MUTED, ColorCss::MUTED_CONTRAST),
            Self::Ink => ColorVars::Named(NamedColorCss::INK, NamedColorCss::SURFACE),
            Self::Surface => ColorVars::Named(NamedColorCss::SURFACE, NamedColorCss::INK),
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
            Self::Neutral => "neutral",
            Self::Muted => "muted",
            Self::Ink => "ink",
            Self::Surface => "surface",
        }
    }
}
