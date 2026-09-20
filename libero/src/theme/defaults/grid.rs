use crate::css::{CssDeclaration, ToCssDeclarations};

use crate::theme::{CssVar, Size, SizeCss};

pub const GRID_GAP: CssVar = CssVar::new("--lsx-grid-gap");
pub const GRID_ZONE_GAP: CssVar = CssVar::new("--lsx-grid-zone-gap");
pub const GRID_ROW_UNIT: CssVar = CssVar::new("--lsx-grid-row-unit");

// Set per instance in `style`, not baked into a class.
pub const GRID_AREAS_VAR: CssVar = CssVar::new("--lsx-grid-areas");
pub const GRID_COLUMNS_VAR: CssVar = CssVar::new("--lsx-grid-columns");
pub const GRID_ZONE_AREA_VAR: CssVar = CssVar::new("--lsx-grid-zone-area");
/// The zone's `@container` name, so an item can key a span off its width; `none` without an area.
pub const GRID_ZONE_CONTAINER_VAR: CssVar = CssVar::new("--lsx-grid-zone-container");
/// The masonry row span from a measured height: written by `GridItem`, read by the zone's rule.
pub const GRID_ITEM_ROWS_VAR: CssVar = CssVar::new("--lsx-grid-item-rows");
/// A caller's own `GridItem { rows }` span; not [`GRID_ITEM_ROWS_VAR`]: one property, one writer.
pub const GRID_ITEM_ROW_SPAN_VAR: CssVar = CssVar::new("--lsx-grid-item-row-span");

/// Theme defaults for `Grid`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridDefaults {
    /// Between zones.
    pub gap: Size,
    /// Between items inside a zone.
    pub zone_gap: Size,
    /// Masonry row quantum in px; the span is `ceil(height / row_unit)`, in Rust.
    pub row_unit: u32,
}

impl GridDefaults {
    pub const DEFAULT: Self = Self {
        gap: Size::Md,
        zone_gap: Size::Md,
        row_unit: 2,
    };
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
