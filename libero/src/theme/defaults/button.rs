use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};

use crate::theme::{Size, SizeCss, Sizes};

pub const BUTTON_FONT_SIZE: SizeCss = SizeCss::new("--lsx-button-font-size-");
pub const BUTTON_HEIGHT: SizeCss = SizeCss::new("--lsx-button-height-");
pub const BUTTON_PADDING_X: SizeCss = SizeCss::new("--lsx-button-padding-x-");

/// Two names for one animation. Restarting a CSS animation on an element that
/// is already running it needs the *name* to change - a class or custom
/// property does not replay it - so consecutive clicks alternate between these.
pub const BUTTON_RIPPLE_ANIMATION: [&str; 2] = ["lsx-ripple-a", "lsx-ripple-b"];
pub const BUTTON_RIPPLE_KEYFRAMES: &str = concat!(
    "@keyframes lsx-ripple-a{from{opacity:0.3;transform:translate(-50%, -50%) scale(0);}",
    "to{transform:translate(-50%, -50%) scale(1);opacity:0;}}",
    "@keyframes lsx-ripple-b{from{opacity:0.3;transform:translate(-50%, -50%) scale(0);}",
    "to{transform:translate(-50%, -50%) scale(1);opacity:0;}}"
);

/// `ripple-a` / `ripple-b`, the `data-state` that runs each one.
pub const BUTTON_RIPPLE_STATE: [&str; 2] = ["ripple-a", "ripple-b"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButtonSizeLevel {
    pub font_size: &'static str,
    pub height: &'static str,
    pub padding_x: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButtonDefaults {
    pub size: Size,
    pub radius: Size,
    pub sizes: Sizes<ButtonSizeLevel>,
}

impl ButtonDefaults {
    pub fn size_sx(size: Size) -> Sx {
        sx().font_size(BUTTON_FONT_SIZE.value(size))
            .height(BUTTON_HEIGHT.value(size))
            .padding_left(BUTTON_PADDING_X.value(size))
            .padding_right(BUTTON_PADDING_X.value(size))
    }

    // The shared global radius scale, keyed by `radius-{size}` rather than
    // `size-{size}`, so it can be set independently of `size`.
    pub fn radius_sx(radius: Size) -> Sx {
        sx().border_radius(SizeCss::RADIUS.value(radius))
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx).per_radius(Self::radius_sx)
    }
}

impl ToCssDeclarations for ButtonDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = Vec::new();
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(BUTTON_FONT_SIZE.declare(size, level.font_size));
            declarations.push(BUTTON_HEIGHT.declare(size, level.height));
            declarations.push(BUTTON_PADDING_X.declare(size, level.padding_x));
        }
        declarations
    }
}
