use crate::sx::{Sx, sx};

use crate::theme::{CssVar, Size};

pub const CONTAINER_SIZE: CssVar = CssVar::new("--lsx-container-size");
pub const CONTAINER_GUTTERS: CssVar = CssVar::new("--lsx-container-gutters");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContainerDefaults {
    pub size: Size,
    pub gutters: Size,
}

impl ContainerDefaults {
    pub const fn new(size: Size, gutters: Size) -> Self {
        Self { size, gutters }
    }

    pub fn default_sx() -> Sx {
        sx().max_width(CONTAINER_SIZE.value())
            .padding_left(CONTAINER_GUTTERS.value())
            .padding_right(CONTAINER_GUTTERS.value())
    }
}
