use crate::css::{CssDeclaration, ToCssDeclarations};

use crate::theme::{Color, ColorShade, ColorValue, CssVar};
pub use crate::tokens::{
    FOCUS_RING_COLOR, FOCUS_RING_HALO, FOCUS_RING_HALO_SPREAD, FOCUS_RING_HALO_WIDTH,
    FOCUS_RING_OFFSET, FOCUS_RING_WIDTH,
};

/// An element's resting `box-shadow`, which the ring's halo shadow composes back in.
/// Set it on any focusable with a resting shadow; [`shadow_sx`](crate::components::common::shadow_sx) does both.
pub const OWN_SHADOW: CssVar = CssVar::new("--lsx-own-shadow");

/// The library's one focus indicator: a dark stripe with a light halo. Two tones
/// carry their own contrast, whatever surface the caller paints it on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FocusRingDefaults {
    /// The stripe. Dark, since the halo only rescues a dark surround.
    pub color: ColorValue,
    /// What the stripe reads against.
    pub halo: ColorValue,
    /// The stripe's thickness, in px.
    pub width: u8,
    /// The gap between the element and the stripe, in px. The halo fills it.
    pub offset: u8,
    /// How far the halo reaches past the stripe, in px.
    pub halo_width: u8,
}

impl FocusRingDefaults {
    /// The theme's own ink and surface (no ramp, shade ignored), so the ring
    /// follows any scheme that redefines the page's two ends.
    pub const DEFAULT: Self = Self {
        color: ColorValue::Shade(Color::Ink, ColorShade::DEFAULT),
        halo: ColorValue::Shade(Color::Surface, ColorShade::DEFAULT),
        width: 2,
        offset: 2,
        halo_width: 2,
    };
}

impl ToCssDeclarations for FocusRingDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            FOCUS_RING_COLOR.declare(self.color.value()),
            FOCUS_RING_HALO.declare(self.halo.value()),
            FOCUS_RING_WIDTH.declare(format!("{}px", self.width)),
            FOCUS_RING_OFFSET.declare(format!("{}px", self.offset)),
            FOCUS_RING_HALO_WIDTH.declare(format!("{}px", self.halo_width)),
            FOCUS_RING_HALO_SPREAD.declare(format!(
                "calc({} + {} + {})",
                FOCUS_RING_OFFSET.value(),
                FOCUS_RING_WIDTH.value(),
                FOCUS_RING_HALO_WIDTH.value()
            )),
        ]
    }
}
