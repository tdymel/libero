use crate::css::{CssDeclaration, ToCssDeclarations};

use crate::theme::{CssVar, Size, SizeCss};

pub const GRID_GAP: CssVar = CssVar::new("--lsx-grid-gap");
pub const GRID_ZONE_GAP: CssVar = CssVar::new("--lsx-grid-zone-gap");
pub const GRID_ROW_UNIT: CssVar = CssVar::new("--lsx-grid-row-unit");

// Set per instance in `style`, not baked into a class.
pub const GRID_AREAS_VAR: CssVar = CssVar::new("--lsx-grid-areas");
pub const GRID_COLUMNS_VAR: CssVar = CssVar::new("--lsx-grid-columns");
pub const GRID_ZONE_AREA_VAR: CssVar = CssVar::new("--lsx-grid-zone-area");
/// The zone's `@container` name, so an item can key a span off its zone's
/// width. `none` when the zone has no area to name it after.
pub const GRID_ZONE_CONTAINER_VAR: CssVar = CssVar::new("--lsx-grid-zone-container");
/// The masonry engine's row span, computed from a measured height. Written by
/// `GridItem` and read by the **zone's** rule, gated on `measured`.
pub const GRID_ITEM_ROWS_VAR: CssVar = CssVar::new("--lsx-grid-item-rows");
/// A caller's own row span, from `GridItem { rows }`. Deliberately **not** the
/// same variable as the masonry engine's: one property, one writer. The two
/// never coexist on an element - `rows` is dropped with a warn inside a masonry
/// zone - and if that guard were ever lost, the zone's rule outranks the item's
/// on specificity anyway.
pub const GRID_ITEM_ROW_SPAN_VAR: CssVar = CssVar::new("--lsx-grid-item-row-span");

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
