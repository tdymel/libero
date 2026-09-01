use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Size, SizeCss, Sizes};

pub const COMBOBOX_FONT_SIZE: SizeCss = SizeCss::new("--lsx-combobox-font-size-");
pub const COMBOBOX_ROW_HEIGHT: SizeCss = SizeCss::new("--lsx-combobox-row-height-");
pub const COMBOBOX_PADDING_X: SizeCss = SizeCss::new("--lsx-combobox-padding-x-");

/// The dropdown's own padding, and so the inset a row nests at - which is what
/// its corner radius has to be smaller by.
pub const COMBOBOX_PADDING: &str = "4px";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComboboxSizeLevel {
    pub font_size: &'static str,
    /// A row's minimum height - a taller row grows past it.
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
            // `min-height`, not `height`: a rich row is taller than the
            // themed one and must not be clipped.
            .min_height(COMBOBOX_ROW_HEIGHT.value(size))
            .padding_left(COMBOBOX_PADDING_X.value(size))
            .padding_right(COMBOBOX_PADDING_X.value(size))
    }

    /// A row nests `COMBOBOX_PADDING` inside the dropdown, so its corner has
    /// to be that much tighter or it crosses the dropdown's own - visibly, at
    /// `xxl`, where the radius is 64px. `max` keeps the small steps at 0
    /// rather than negative, which is not a radius at all.
    pub fn row_radius_sx(radius: Size) -> Sx {
        sx().border_radius(format!(
            "max(0px, calc({} - {COMBOBOX_PADDING}))",
            SizeCss::RADIUS.value(radius)
        ))
    }

    pub fn row_theme_vars() -> Sx {
        sx().per_size(Self::row_sx).per_radius(Self::row_radius_sx)
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
