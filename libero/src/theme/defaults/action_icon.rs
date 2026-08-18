use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{CssVar, ICON_SIZE, Size, SizeCss};

pub const ACTION_ICON_SIZE: CssVar = CssVar::new("--lsx-action-icon-size");
pub const ACTION_ICON_RADIUS: CssVar = CssVar::new("--lsx-action-icon-radius");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionIconDefaults {
    pub size: Size,
    pub radius: Size,
}

impl ToCssDeclarations for ActionIconDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            ACTION_ICON_SIZE.declare(ICON_SIZE.value(self.size)),
            ACTION_ICON_RADIUS.declare(SizeCss::RADIUS.value(self.radius)),
        ]
    }
}
