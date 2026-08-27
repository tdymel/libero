use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Size, SizeCss, Sizes};

pub const TEXT_FIELD_FONT_SIZE: SizeCss = SizeCss::new("--lsx-text-field-font-size-");
pub const TEXT_FIELD_HEIGHT: SizeCss = SizeCss::new("--lsx-text-field-height-");
pub const TEXT_FIELD_PADDING_X: SizeCss = SizeCss::new("--lsx-text-field-padding-x-");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextFieldSizeLevel {
    pub font_size: &'static str,
    pub height: &'static str,
    pub padding_x: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextFieldDefaults {
    pub size: Size,
    pub radius: Size,
    pub sizes: Sizes<TextFieldSizeLevel>,
}

impl TextFieldDefaults {
    pub fn size_sx(size: Size) -> Sx {
        sx().font_size(TEXT_FIELD_FONT_SIZE.value(size))
            .height(TEXT_FIELD_HEIGHT.value(size))
            .padding_left(TEXT_FIELD_PADDING_X.value(size))
            .padding_right(TEXT_FIELD_PADDING_X.value(size))
    }

    pub fn radius_sx(radius: Size) -> Sx {
        sx().border_radius(SizeCss::RADIUS.value(radius))
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx).per_radius(Self::radius_sx)
    }
}

impl ToCssDeclarations for TextFieldDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = Vec::new();
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(TEXT_FIELD_FONT_SIZE.declare(size, level.font_size));
            declarations.push(TEXT_FIELD_HEIGHT.declare(size, level.height));
            declarations.push(TEXT_FIELD_PADDING_X.declare(size, level.padding_x));
        }
        declarations
    }
}
