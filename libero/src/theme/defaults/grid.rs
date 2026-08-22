use crate::css::{CssDeclaration, ToCssDeclarations};

use crate::theme::{CssVar, Size, SizeCss};

pub const GRID_GAP: CssVar = CssVar::new("--lsx-grid-gap");
pub const GRID_ZONE_GAP: CssVar = CssVar::new("--lsx-grid-zone-gap");
pub const GRID_ROW_UNIT: CssVar = CssVar::new("--lsx-grid-row-unit");

// Set per instance in `style`, not baked into a class.
pub const GRID_AREAS_VAR: CssVar = CssVar::new("--lsx-grid-areas");
pub const GRID_COLUMNS_VAR: CssVar = CssVar::new("--lsx-grid-columns");
pub const GRID_ZONE_AREA_VAR: CssVar = CssVar::new("--lsx-grid-zone-area");
pub const GRID_ITEM_ROWS_VAR: CssVar = CssVar::new("--lsx-grid-item-rows");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridDefaults {
    /// Between zones.
    pub gap: Size,
    /// Between items inside a zone.
    pub zone_gap: Size,
    /// Masonry row quantum, in pixels. A number, not a CSS length: the row span
    /// is `ceil(height / row_unit)`, computed in Rust.
    pub row_unit: u32,
}

impl ToCssDeclarations for GridDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            GRID_GAP.declare(SizeCss::SPACING.value(self.gap)),
            GRID_ZONE_GAP.declare(SizeCss::SPACING.value(self.zone_gap)),
            GRID_ROW_UNIT.declare(format!("{}px", self.row_unit)),
        ]
    }
}
