use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Size, SizeCss, Sizes};

pub const COMBOBOX_FONT_SIZE: SizeCss = SizeCss::new("--lsx-combobox-font-size-");
pub const COMBOBOX_ROW_HEIGHT: SizeCss = SizeCss::new("--lsx-combobox-row-height-");
pub const COMBOBOX_PADDING_X: SizeCss = SizeCss::new("--lsx-combobox-padding-x-");

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComboboxSizeLevel {
    pub font_size: &'static str,
    /// A row's minimum height, and the `item_size` handed to `Virtualize` -
    /// which is why it is a number: rows never probe their own height. A
    /// taller custom row passes its own through `option_height`.
    pub row_height: f64,
    pub padding_x: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComboboxDefaults {
    pub size: Size,
    pub radius: Size,
    pub max_dropdown_height: &'static str,
    pub sizes: Sizes<ComboboxSizeLevel>,
}

impl ComboboxDefaults {
    pub fn row_sx(size: Size) -> Sx {
        sx().font_size(COMBOBOX_FONT_SIZE.value(size))
            // `min-height`, not `height`: a rich `OptionLabel` is taller than
            // the themed row and must not be clipped.
            .min_height(COMBOBOX_ROW_HEIGHT.value(size))
            .padding_left(COMBOBOX_PADDING_X.value(size))
            .padding_right(COMBOBOX_PADDING_X.value(size))
    }

    pub fn radius_sx(radius: Size) -> Sx {
        sx().border_radius(SizeCss::RADIUS.value(radius))
    }

    pub fn row_theme_vars() -> Sx {
        sx().per_size(Self::row_sx)
    }

    pub fn dropdown_theme_vars() -> Sx {
        sx().per_radius(Self::radius_sx)
    }

    pub fn row_height(&self, size: Size) -> f64 {
        self.sizes.get(size).row_height
    }
}

impl ToCssDeclarations for ComboboxDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = Vec::new();
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(COMBOBOX_FONT_SIZE.declare(size, level.font_size));
            declarations.push(COMBOBOX_ROW_HEIGHT.declare(size, format!("{}px", level.row_height)));
            declarations.push(COMBOBOX_PADDING_X.declare(size, level.padding_x));
        }
        declarations
    }
}
