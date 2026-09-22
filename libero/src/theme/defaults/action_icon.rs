use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{BUTTON_HEIGHT, CssVar, ICON_SIZE, Size, SizeCss, Variant};

pub const ACTION_ICON_SIZE: CssVar = CssVar::new("--lsx-action-icon-size");
pub const ACTION_ICON_RADIUS: CssVar = CssVar::new("--lsx-action-icon-radius");
/// The glyph inside the box: `Icon`'s size at the box's step.
pub(crate) const ACTION_ICON_GLYPH: CssVar = CssVar::new("--lsx-action-icon-glyph");

/// Theme defaults for `ActionIcon`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionIconDefaults {
    /// The chrome an action icon takes when a call site names none.
    pub variant: Variant,
    /// A step of `Button`'s heights, so the two line up in a row.
    pub size: Size,
    pub radius: Size,
}

impl ActionIconDefaults {
    pub const DEFAULT: Self = Self {
        variant: Variant::Filled,
        size: Size::Md,
        radius: Size::Sm,
    };
}

impl ToCssDeclarations for ActionIconDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            ACTION_ICON_SIZE.declare(BUTTON_HEIGHT.value(self.size)),
            ACTION_ICON_GLYPH.declare(ICON_SIZE.value(self.size)),
            ACTION_ICON_RADIUS.declare(SizeCss::RADIUS.value(self.radius)),
        ]
    }
}
