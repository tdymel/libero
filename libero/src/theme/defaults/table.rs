use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Color, ColorShade, ColorValue, CssVar};

pub const TABLE_PADDING_X: CssVar = CssVar::new("--lsx-table-padding-x");
pub const TABLE_PADDING_Y: CssVar = CssVar::new("--lsx-table-padding-y");
pub const TABLE_FONT_SIZE: CssVar = CssVar::new("--lsx-table-font-size");
pub const TABLE_BORDER_COLOR: CssVar = CssVar::new("--lsx-table-border-color");
pub const TABLE_HOVER: CssVar = CssVar::new("--lsx-table-hover");

/// Flat values, not a `Sizes` scale: `Table` has no `size` prop yet, so a
/// per-size scale would be six numbers expressing one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableDefaults {
    pub padding_x: u8,
    pub padding_y: u8,
    pub font_size: u16,
    pub border_color: ColorValue,
    pub hover_color: ColorValue,
}

impl TableDefaults {
    pub const DEFAULT: Self = Self {
        padding_x: 12,
        padding_y: 10,
        font_size: 14,
        border_color: ColorValue::Shade(Color::Muted, ColorShade::S3),
        hover_color: ColorValue::Shade(Color::Muted, ColorShade::S1),
    };

    fn border() -> String {
        format!("1px solid {}", TABLE_BORDER_COLOR.value())
    }

    /// Also used by the sort button, which has to fill its header cell.
    pub(crate) fn padding() -> String {
        format!("{} {}", TABLE_PADDING_Y.value(), TABLE_PADDING_X.value())
    }

    pub fn theme_vars() -> Sx {
        sx().font_size(TABLE_FONT_SIZE.value())
            .selector("& th, & td", sx().padding(Self::padding()))
            .selector("& thead th", sx().border_bottom(Self::border()))
            // On the cells: a row's own border draws only when borders collapse.
            .selector("& tbody tr > *", sx().border_bottom(Self::border()))
            .selector("& tbody tr:hover", sx().background(TABLE_HOVER.value()))
    }
}

impl ToCssDeclarations for TableDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            TABLE_PADDING_X.declare(format!("{}px", self.padding_x)),
            TABLE_PADDING_Y.declare(format!("{}px", self.padding_y)),
            TABLE_FONT_SIZE.declare(format!("{}px", self.font_size)),
            TABLE_BORDER_COLOR.declare(self.border_color.value()),
            TABLE_HOVER.declare(self.hover_color.value()),
        ]
    }
}
