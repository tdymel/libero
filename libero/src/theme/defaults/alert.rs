use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size, SizeCss, Variant};

pub const ALERT_RADIUS: CssVar = CssVar::new("--lsx-alert-radius");
pub const ALERT_PADDING: CssVar = CssVar::new("--lsx-alert-padding");
/// Icon to body.
pub const ALERT_GAP: CssVar = CssVar::new("--lsx-alert-gap");
/// Title to message.
pub const ALERT_BODY_GAP: CssVar = CssVar::new("--lsx-alert-body-gap");
pub const ALERT_ICON_SIZE: CssVar = CssVar::new("--lsx-alert-icon-size");

/// Theme defaults for `Alert`, set on [`Theme`](crate::theme::Theme).
///
/// Spacing fields are [`SizeCss::SPACING`] steps; `icon_size` is a length (a glyph box).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AlertDefaults {
    /// Chrome, shared with `Button`; `Tonal` is the tinted arm.
    pub variant: Variant,
    /// Palette colour name for the `color` prop. `"info"`, not the brand colour:
    /// severity is the caller's to state.
    pub color: &'static str,
    /// Matches `paper.radius`: an alert is a surface.
    pub radius: Size,
    pub padding: Size,
    pub gap: Size,
    pub body_gap: Size,
    pub icon_size: &'static str,
}

impl AlertDefaults {
    pub const DEFAULT: Self = Self {
        variant: Variant::Tonal,
        color: "info",
        radius: Size::Md,
        padding: Size::Md,
        gap: Size::Md,
        body_gap: Size::Xs,
        icon_size: "20px",
    };

    pub fn theme_vars() -> Sx {
        sx().border_radius(ALERT_RADIUS.overridable())
            .padding(ALERT_PADDING.value())
            .gap(ALERT_GAP.value())
    }
}

impl ToCssDeclarations for AlertDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            ALERT_RADIUS.declare(SizeCss::RADIUS.value(self.radius)),
            ALERT_PADDING.declare(SizeCss::SPACING.value(self.padding)),
            ALERT_GAP.declare(SizeCss::SPACING.value(self.gap)),
            ALERT_BODY_GAP.declare(SizeCss::SPACING.value(self.body_gap)),
            ALERT_ICON_SIZE.declare(self.icon_size),
        ]
    }
}
