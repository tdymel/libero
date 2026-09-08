use super::{ColorCss, NamedColorCss, ShadeRamp};

/// How a [`Color`] names its custom properties, as `(own, contrast)`. Palette
/// colors get a 9-shade var per role; ink/surface get one each and are each
/// other's contrast.
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
    Neutral,
    Grey,
    /// What text is set in. The dark end of the page on a light theme, the
    /// light end on a dark one - which is why it is not called "black".
    Ink,
    /// The page text is set on, and the surface every color role is measured
    /// against. Not `Paper`'s background: that is the card drawn on top of
    /// this one.
    Surface,
}

impl Color {
    /// Greys and ink/surface walk the neutral mix curve, hues the chromatic one.
    pub(crate) const fn shade_ramp(self) -> ShadeRamp {
        match self {
            Self::Neutral | Self::Grey | Self::Ink | Self::Surface => ShadeRamp::Neutral,
            _ => ShadeRamp::Chromatic,
        }
    }

    /// A name this enum does not know returns `None`, and `sx` then passes
    /// the string through as raw CSS.
    ///
    /// That matters here: `"white"` and `"black"` used to name the two ends
    /// of the page and now do not, so they no longer resolve to
    /// `--lsx-surface`/`--lsx-ink` - they fall through to the CSS keywords
    /// `white` and `black`, which are valid CSS and so fail **silently**,
    /// pinning the color to one scheme instead of following the theme. Todo
    /// 378 asks whether an unrecognised color name should fall through at
    /// all.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "primary" => Some(Self::Primary),
            "secondary" => Some(Self::Secondary),
            "error" => Some(Self::Error),
            "warning" => Some(Self::Warning),
            "info" => Some(Self::Info),
            "success" => Some(Self::Success),
            "neutral" => Some(Self::Neutral),
            "grey" => Some(Self::Grey),
            "ink" => Some(Self::Ink),
            "surface" => Some(Self::Surface),
            _ => None,
        }
    }

    /// Every shade/contrast and name/value lookup goes through here.
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
            Self::Grey => ColorVars::Palette(ColorCss::GREY, ColorCss::GREY_CONTRAST),
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
            Self::Grey => "grey",
            Self::Ink => "ink",
            Self::Surface => "surface",
        }
    }
}
