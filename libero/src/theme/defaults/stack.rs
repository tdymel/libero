use crate::sx::{Sx, sx};

use crate::theme::{CssVar, Size};

pub const STACK_COLUMN_ALIGN: CssVar = CssVar::new("--lsx-stack-column-align");
pub const STACK_COLUMN_JUSTIFY: CssVar = CssVar::new("--lsx-stack-column-justify");
pub const STACK_COLUMN_SPACING: CssVar = CssVar::new("--lsx-stack-column-spacing");
pub const STACK_COLUMN_WRAP: CssVar = CssVar::new("--lsx-stack-column-wrap");
pub const STACK_ROW_ALIGN: CssVar = CssVar::new("--lsx-stack-row-align");
pub const STACK_ROW_JUSTIFY: CssVar = CssVar::new("--lsx-stack-row-justify");
pub const STACK_ROW_SPACING: CssVar = CssVar::new("--lsx-stack-row-spacing");
pub const STACK_ROW_WRAP: CssVar = CssVar::new("--lsx-stack-row-wrap");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StackAxisDefaults {
    pub align: &'static str,
    pub justify: &'static str,
    pub spacing: Size,
    pub wrap: bool,
}

impl StackAxisDefaults {
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
pub struct StackDefaults {
    pub column: StackAxisDefaults,
    pub row: StackAxisDefaults,
}

impl StackDefaults {
    pub const fn new(column: StackAxisDefaults, row: StackAxisDefaults) -> Self {
        Self { column, row }
    }

    pub fn default_sx(is_row: bool) -> Sx {
        let direction = if is_row { "row" } else { "column" };
        let align = if is_row {
            STACK_ROW_ALIGN.value()
        } else {
            STACK_COLUMN_ALIGN.value()
        };
        let justify = if is_row {
            STACK_ROW_JUSTIFY.value()
        } else {
            STACK_COLUMN_JUSTIFY.value()
        };
        let spacing = if is_row {
            STACK_ROW_SPACING.value()
        } else {
            STACK_COLUMN_SPACING.value()
        };
        let wrap = if is_row {
            STACK_ROW_WRAP.value()
        } else {
            STACK_COLUMN_WRAP.value()
        };

        sx().flex_direction(direction)
            .align_items(align)
            .justify_content(justify)
            .gap(spacing)
            .flex_wrap(wrap)
    }
}
