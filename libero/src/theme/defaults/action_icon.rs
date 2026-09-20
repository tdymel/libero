use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{CssVar, ICON_SIZE, Size, SizeCss, Variant};

pub const ACTION_ICON_SIZE: CssVar = CssVar::new("--lsx-action-icon-size");
pub const ACTION_ICON_RADIUS: CssVar = CssVar::new("--lsx-action-icon-radius");

/// Theme defaults for `ActionIcon`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionIconDefaults {
    /// The chrome an action icon takes when a call site names none.
    pub variant: Variant,
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
            ACTION_ICON_SIZE.declare(ICON_SIZE.value(self.size)),
            ACTION_ICON_RADIUS.declare(SizeCss::RADIUS.value(self.radius)),
        ]
    }
}
