use crate::components::ButtonVariant;
use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size, SizeCss};

pub const ALERT_RADIUS: CssVar = CssVar::new("--lsx-alert-radius");
pub const ALERT_PADDING: CssVar = CssVar::new("--lsx-alert-padding");
/// Icon to body.
pub const ALERT_GAP: CssVar = CssVar::new("--lsx-alert-gap");
/// Title to message.
pub const ALERT_BODY_GAP: CssVar = CssVar::new("--lsx-alert-body-gap");
pub const ALERT_ICON_SIZE: CssVar = CssVar::new("--lsx-alert-icon-size");

/// What an alert looks like when nobody says otherwise.
///
/// The spacing fields are `Size` steps resolved through [`SizeCss::SPACING`],
/// not CSS lengths: a raw `"md"` is not something a `CssVar` can declare.
/// `icon_size` is the exception and is a length, because 20px is off the
/// spacing scale and is a glyph box rather than a gap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AlertDefaults {
    /// Chrome, shared with `Button`. `Tonal` is the tinted arm, which is what
    /// an alert is; it carries no `var()` of its own, only a `data-state`.
    pub variant: ButtonVariant,
    /// A palette colour name, read in Rust to default the `color` prop - not
    /// a CSS declaration, the way `badge.size` is not one. The component
    /// publishes the resolved shades as its own `var()`s per instance.
    ///
    /// `"info"` rather than the primary colour: severity is the caller's to
    /// state, and an alert painted in the brand colour reads as decoration.
    pub color: &'static str,
    /// Meant to match `paper.radius` - an alert is a surface. Spelled rather
    /// than referenced because `Theme::DEFAULT` is a const struct literal.
    pub radius: Size,
    pub padding: Size,
    pub gap: Size,
    pub body_gap: Size,
    pub icon_size: &'static str,
    /// An English literal, like `DateDefaults`'s. The library has no i18n
    /// story yet (todo 28); a project overrides it once here.
    pub close_label: &'static str,
}

impl AlertDefaults {
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
