use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Responsive, Size, SizeCss, responsive};

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
        /// `standard`, with every second cell shortened to 70% and centred for
        /// an alternating rhythm. Decoration, and it crops.
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
    /// `GridZone`'s twelve tracks. `2` at every width: a theme that wants more
    /// on a wide screen says `responsive(2).md(3)`.
    pub cols: Responsive<u8>,
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
    pub const DEFAULT: Self = Self {
        cols: responsive(2),
        variant: ImageListVariant::Standard,
        gap: Size::Xs,
        radius: Size::Sm,
        bar_position: BarPosition::Bottom,
        // Never under 60% black: white text holds 4.5:1 over a pure-white photo (todo 929).
        bar_background: "linear-gradient(to top, rgba(0,0,0,0.72), rgba(0,0,0,0.6))",
        bar_background_top: "linear-gradient(to bottom, rgba(0,0,0,0.72), rgba(0,0,0,0.6))",
        bar_color: "#fff",
        bar_padding: Size::Sm,
    };

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::HexColor;

    /// Every stop of both scrims, composited over pure white, keeps the bar's
    /// white text at 4.5:1 - the worst photo the sweep cannot see (todo 929).
    #[test]
    fn the_scrim_holds_white_text_over_a_white_photo() {
        let defaults = ImageListDefaults::DEFAULT;
        assert_eq!(defaults.bar_color, "#fff");
        for scrim in [defaults.bar_background, defaults.bar_background_top] {
            assert!(!scrim.contains("transparent"), "{scrim} fades out");
            let alphas: Vec<f32> = scrim
                .split("rgba(0,0,0,")
                .skip(1)
                .map(|rest| rest.split(')').next().unwrap().parse().unwrap())
                .collect();
            assert!(!alphas.is_empty(), "{scrim} has no black stop");
            for alpha in alphas {
                let grey = (255.0 * (1.0 - alpha)).round() as u32;
                let ratio =
                    HexColor::new(0xFF_FF_FF).contrast_ratio(HexColor::new(grey * 0x01_01_01));
                assert!(ratio >= 4.5, "{scrim}: {alpha} black gives {ratio:.2}:1");
            }
        }
    }
}
