use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size, SizeCss};

/// The scrim behind a `Bottom` bar, and behind a `Top` one. Two values rather
/// than one rotated: a scrim fades *away* from its edge, so the gradient's
/// direction is part of the value and not a transform of it.
pub const IMAGE_LIST_BAR_BACKGROUND: CssVar = CssVar::new("--lsx-image-list-bar-background");
pub const IMAGE_LIST_BAR_BACKGROUND_TOP: CssVar =
    CssVar::new("--lsx-image-list-bar-background-top");
pub const IMAGE_LIST_BAR_COLOR: CssVar = CssVar::new("--lsx-image-list-bar-color");
/// A spacing step, resolved here rather than published per size: the bar's
/// padding is not a prop, so no `data-state` ever picks a different step.
pub const IMAGE_LIST_BAR_PADDING: CssVar = CssVar::new("--lsx-image-list-bar-padding");

/// The cell's corner radius, resolved on the cell from its own `data-state`.
pub const IMAGE_LIST_RADIUS: CssVar = CssVar::new("--lsx-image-list-radius");

str_enum! {
    /// How the cells are packed.
    #[state_prefix = "variant"]
    pub enum ImageListVariant {
        /// Every cell the same height, from the list's aspect ratio.
        #[default]
        Standard = "standard",
        /// Cells keep their own height and are packed with no dead space,
        /// by `GridZone`'s measuring engine.
        Masonry = "masonry",
        /// `standard`, plus `ImageItem::rows`: a cell may take more than one
        /// row, and the quilt's rows stay equal.
        Quilted = "quilted",
        /// `standard`, with every second cell shortened to 70% and centred -
        /// the alternating rhythm MUI calls woven. Decoration, and it crops.
        Woven = "woven",
    }
}

str_enum! {
    /// Where a cell's bar sits.
    #[state_prefix = "bar"]
    pub enum BarPosition {
        /// Over the bottom of the image, on a scrim.
        #[default]
        Bottom = "bottom",
        /// Over the top of the image, on a scrim.
        Top = "top",
        /// Under the image, in flow - the only position that is not an
        /// overlay, and the only one that does not darken the picture.
        Below = "below",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageListDefaults {
    /// Snapped to a divisor of twelve by `ImageList` - a cell is a span of a
    /// `GridZone`'s twelve tracks. `2` rather than MUI's larger counts: a
    /// fixed column count has to be the one that still works on a phone
    /// until per-breakpoint columns land.
    pub cols: u8,
    pub variant: ImageListVariant,
    pub gap: Size,
    pub radius: Size,
    pub bar_position: BarPosition,
    pub bar_background: &'static str,
    pub bar_background_top: &'static str,
    pub bar_color: &'static str,
    pub bar_padding: Size,
}

impl ImageListDefaults {
    fn radius_sx(radius: Size) -> Sx {
        sx().var(IMAGE_LIST_RADIUS, SizeCss::RADIUS.value(radius))
    }

    /// Composed into the **cell's** base `Sx`, not the list's: the radius is
    /// the cell's corner, so the `data-state` token that picks a step rides
    /// the element that draws it.
    pub fn theme_vars() -> Sx {
        sx().per_radius(Self::radius_sx)
    }
}

impl ToCssDeclarations for ImageListDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            IMAGE_LIST_BAR_BACKGROUND.declare(self.bar_background),
            IMAGE_LIST_BAR_BACKGROUND_TOP.declare(self.bar_background_top),
            IMAGE_LIST_BAR_COLOR.declare(self.bar_color),
            IMAGE_LIST_BAR_PADDING.declare(SizeCss::SPACING.value(self.bar_padding)),
        ]
    }
}
