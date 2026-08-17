use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};

use crate::theme::{CssVar, Size, SizeCss};

pub const FLEX_COLUMN_ALIGN: CssVar = CssVar::new("--lsx-flex-column-align");
pub const FLEX_COLUMN_JUSTIFY: CssVar = CssVar::new("--lsx-flex-column-justify");
pub const FLEX_COLUMN_SPACING: CssVar = CssVar::new("--lsx-flex-column-spacing");
pub const FLEX_COLUMN_WRAP: CssVar = CssVar::new("--lsx-flex-column-wrap");
pub const FLEX_ROW_ALIGN: CssVar = CssVar::new("--lsx-flex-row-align");
pub const FLEX_ROW_JUSTIFY: CssVar = CssVar::new("--lsx-flex-row-justify");
pub const FLEX_ROW_SPACING: CssVar = CssVar::new("--lsx-flex-row-spacing");
pub const FLEX_ROW_WRAP: CssVar = CssVar::new("--lsx-flex-row-wrap");

// Per-instance overrides, set directly in the element's `style` attribute
// (see `Variables`) rather than baked into a generated class - `default_sx`
// references these with a fallback to the axis's own theme default, so an
// override never needs a new class/stylesheet entry of its own.
pub const FLEX_ALIGN_VAR: &str = "--lsx-flex-align";
pub const FLEX_JUSTIFY_VAR: &str = "--lsx-flex-justify";
pub const FLEX_WRAP_VAR: &str = "--lsx-flex-wrap";

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
            .align_items(format!("var({FLEX_ALIGN_VAR}, {align})"))
            .justify_content(format!("var({FLEX_JUSTIFY_VAR}, {justify})"))
            .gap(spacing)
            .flex_wrap(format!("var({FLEX_WRAP_VAR}, {wrap})"))
    }
}

impl ToCssDeclarations for FlexDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            FLEX_COLUMN_ALIGN.declare(self.column.align),
            FLEX_COLUMN_JUSTIFY.declare(self.column.justify),
            FLEX_COLUMN_SPACING.declare(SizeCss::SPACING.value(self.column.spacing)),
            FLEX_COLUMN_WRAP.declare(if self.column.wrap { "wrap" } else { "nowrap" }),
            FLEX_ROW_ALIGN.declare(self.row.align),
            FLEX_ROW_JUSTIFY.declare(self.row.justify),
            FLEX_ROW_SPACING.declare(SizeCss::SPACING.value(self.row.spacing)),
            FLEX_ROW_WRAP.declare(if self.row.wrap { "wrap" } else { "nowrap" }),
        ]
    }
}
