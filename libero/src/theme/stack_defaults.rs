use crate::sx::{Sx, sx};

use super::Size;

pub const STACK_COLUMN_ALIGN_VAR: &str = "--lsx-stack-column-align";
pub const STACK_COLUMN_JUSTIFY_VAR: &str = "--lsx-stack-column-justify";
pub const STACK_COLUMN_SPACING_VAR: &str = "--lsx-stack-column-spacing";
pub const STACK_COLUMN_WRAP_VAR: &str = "--lsx-stack-column-wrap";
pub const STACK_ROW_ALIGN_VAR: &str = "--lsx-stack-row-align";
pub const STACK_ROW_JUSTIFY_VAR: &str = "--lsx-stack-row-justify";
pub const STACK_ROW_SPACING_VAR: &str = "--lsx-stack-row-spacing";
pub const STACK_ROW_WRAP_VAR: &str = "--lsx-stack-row-wrap";

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
            format!("var({STACK_ROW_ALIGN_VAR})")
        } else {
            format!("var({STACK_COLUMN_ALIGN_VAR})")
        };
        let justify = if is_row {
            format!("var({STACK_ROW_JUSTIFY_VAR})")
        } else {
            format!("var({STACK_COLUMN_JUSTIFY_VAR})")
        };
        let spacing = if is_row {
            format!("var({STACK_ROW_SPACING_VAR})")
        } else {
            format!("var({STACK_COLUMN_SPACING_VAR})")
        };
        let wrap = if is_row {
            format!("var({STACK_ROW_WRAP_VAR})")
        } else {
            format!("var({STACK_COLUMN_WRAP_VAR})")
        };

        sx().flex_direction(direction)
            .align_items(align)
            .justify_content(justify)
            .gap(spacing)
            .flex_wrap(wrap)
    }
}
