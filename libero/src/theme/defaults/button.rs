use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};

use crate::theme::{Size, SizeCss, Sizes, Variant};

pub const BUTTON_FONT_SIZE: SizeCss = SizeCss::new("--lsx-button-font-size-");
pub const BUTTON_HEIGHT: SizeCss = SizeCss::new("--lsx-button-height-");
pub const BUTTON_PADDING_X: SizeCss = SizeCss::new("--lsx-button-padding-x-");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButtonSizeLevel {
    pub font_size: &'static str,
    pub height: &'static str,
    pub padding_x: &'static str,
}

/// Theme defaults for `Button`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButtonDefaults {
    /// The chrome a button takes when a call site names none.
    pub variant: Variant,
    pub size: Size,
    pub radius: Size,
    pub sizes: Sizes<ButtonSizeLevel>,
}

impl ButtonDefaults {
    pub const DEFAULT: Self = Self {
        variant: Variant::Filled,
        size: Size::Md,
        radius: Size::Md,
        sizes: Sizes::new(
            ButtonSizeLevel {
                font_size: "0.75rem",
                height: "24px",
                padding_x: "10px",
            },
            ButtonSizeLevel {
                font_size: "0.875rem",
                height: "30px",
                padding_x: "14px",
            },
            ButtonSizeLevel {
                font_size: "1rem",
                height: "42px",
                padding_x: "18px",
            },
            ButtonSizeLevel {
                font_size: "1.125rem",
                height: "50px",
                padding_x: "22px",
            },
            ButtonSizeLevel {
                font_size: "1.25rem",
                height: "60px",
                padding_x: "28px",
            },
            ButtonSizeLevel {
                font_size: "1.375rem",
                height: "72px",
                padding_x: "34px",
            },
        ),
    };

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
