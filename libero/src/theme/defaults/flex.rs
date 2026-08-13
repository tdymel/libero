use crate::sx::{Sx, sx};

use crate::theme::{CssVar, Size};

pub const FLEX_COLUMN_ALIGN: CssVar = CssVar::new("--lsx-flex-column-align");
pub const FLEX_COLUMN_JUSTIFY: CssVar = CssVar::new("--lsx-flex-column-justify");
pub const FLEX_COLUMN_SPACING: CssVar = CssVar::new("--lsx-flex-column-spacing");
pub const FLEX_COLUMN_WRAP: CssVar = CssVar::new("--lsx-flex-column-wrap");
pub const FLEX_ROW_ALIGN: CssVar = CssVar::new("--lsx-flex-row-align");
pub const FLEX_ROW_JUSTIFY: CssVar = CssVar::new("--lsx-flex-row-justify");
pub const FLEX_ROW_SPACING: CssVar = CssVar::new("--lsx-flex-row-spacing");
pub const FLEX_ROW_WRAP: CssVar = CssVar::new("--lsx-flex-row-wrap");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlexAxisDefaults {
    pub align: &'static str,
    pub justify: &'static str,
    pub spacing: Size,
    pub wrap: bool,
}

impl FlexAxisDefaults {
    pub const fn new(
        align: &'static str,
        justify: &'static str,
        spacing: Size,
        wrap: bool,
    ) -> Self {
        Self {
            align,
            justify,
            spacing,
            wrap,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlexDefaults {
    pub column: FlexAxisDefaults,
    pub row: FlexAxisDefaults,
}

impl FlexDefaults {
    pub const fn new(column: FlexAxisDefaults, row: FlexAxisDefaults) -> Self {
        Self { column, row }
    }

    pub fn default_sx(is_row: bool) -> Sx {
        let direction = if is_row { "row" } else { "column" };
        let align = if is_row {
            FLEX_ROW_ALIGN.value()
        } else {
            FLEX_COLUMN_ALIGN.value()
        };
        let justify = if is_row {
            FLEX_ROW_JUSTIFY.value()
        } else {
            FLEX_COLUMN_JUSTIFY.value()
        };
        let spacing = if is_row {
            FLEX_ROW_SPACING.value()
        } else {
            FLEX_COLUMN_SPACING.value()
        };
        let wrap = if is_row {
            FLEX_ROW_WRAP.value()
        } else {
            FLEX_COLUMN_WRAP.value()
        };

        sx().flex_direction(direction)
            .align_items(align)
            .justify_content(justify)
            .gap(spacing)
            .flex_wrap(wrap)
    }
}
