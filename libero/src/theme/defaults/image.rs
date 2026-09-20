use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::sx::{Sx, sx};
use crate::theme::CssVar;

pub const IMAGE_RADIUS: CssVar = CssVar::new("--lsx-image-radius");

str_enum! {
    /// How an `Image` fills its box, as `object-fit`.
    #[state_prefix = "fit"]
    pub enum ImageFit {
        Fill = "fill",
        Contain = "contain",
        #[default]
        Cover = "cover",
        None = "none",
        ScaleDown = "scale-down" | "scaledown",
    }
}

/// Theme defaults for `Image`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageDefaults {
    pub fit: ImageFit,
    /// CSS length for an `Image` with no `radius` of its own.
    pub radius: &'static str,
}

impl ImageDefaults {
    pub const DEFAULT: Self = Self {
        fit: ImageFit::Cover,
        radius: "0",
    };

    pub fn theme_vars() -> Sx {
        sx().border_radius(IMAGE_RADIUS.overridable())
    }
}

impl ToCssDeclarations for ImageDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![IMAGE_RADIUS.declare(self.radius)]
    }
}
