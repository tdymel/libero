use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{CssVar, Size, SizeCss};

pub const ACTION_ICON_SIZE: CssVar = CssVar::new("--lsx-action-icon-size");
pub const ACTION_ICON_RADIUS: CssVar = CssVar::new("--lsx-action-icon-radius");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionIconDefaults {
    pub size: Size,
    pub radius: Size,
}

impl ActionIconDefaults {
    pub const fn new(size: Size, radius: Size) -> Self {
        Self { size, radius }
    }
}

impl ToCssDeclarations for ActionIconDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            ACTION_ICON_SIZE.declare(SizeCss::ICON_SIZE.value(self.size)),
            ACTION_ICON_RADIUS.declare(SizeCss::RADIUS.value(self.radius)),
        ]
    }
}
