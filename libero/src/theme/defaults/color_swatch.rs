use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const COLOR_SWATCH_SIZE_SIZE: SizeCss = SizeCss::new("--lsx-color-swatch-size-");

pub const COLOR_SWATCH_SIZE: CssVar = CssVar::new("--lsx-color-swatch-size");
pub const COLOR_SWATCH_RADIUS: CssVar = CssVar::new("--lsx-color-swatch-radius");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorSwatchDefaults {
    pub size: Size,
    pub radius: Size,
    /// The swatch's diameter, per size step.
    pub sizes: Sizes<&'static str>,
}

impl ColorSwatchDefaults {
    pub fn size_sx(size: Size) -> Sx {
        sx().var(COLOR_SWATCH_SIZE, COLOR_SWATCH_SIZE_SIZE.value(size))
    }

    pub fn radius_sx(radius: Size) -> Sx {
        sx().var(COLOR_SWATCH_RADIUS, SizeCss::RADIUS.value(radius))
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx).per_radius(Self::radius_sx)
    }
}

impl ToCssDeclarations for ColorSwatchDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        Size::ALL
            .into_iter()
            .map(|size| COLOR_SWATCH_SIZE_SIZE.declare(size, self.sizes.get(size)))
            .collect()
    }
}
