use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};

use crate::theme::{CssVar, Size, SizeCss};

pub const CONTAINER_SIZE: CssVar = CssVar::new("--lsx-container-size");
pub const CONTAINER_GUTTERS: CssVar = CssVar::new("--lsx-container-gutters");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContainerDefaults {
    pub size: Size,
    pub gutters: Size,
}

impl ContainerDefaults {
    pub fn default_sx() -> Sx {
        sx().max_width(CONTAINER_SIZE.value())
            .padding_left(CONTAINER_GUTTERS.value())
            .padding_right(CONTAINER_GUTTERS.value())
    }
}

impl ToCssDeclarations for ContainerDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            CONTAINER_SIZE.declare(SizeCss::BREAKPOINT.value(self.size)),
            CONTAINER_GUTTERS.declare(SizeCss::SPACING.value(self.gutters)),
        ]
    }
}
