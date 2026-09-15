use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{CssVar, Size, SizeCss};

pub const LIGHTBOX_WIDTH: CssVar = CssVar::new("--lsx-lightbox-width");
pub const LIGHTBOX_STAGE_HEIGHT: CssVar = CssVar::new("--lsx-lightbox-stage-height");
pub const LIGHTBOX_THUMBNAIL_SIZE: CssVar = CssVar::new("--lsx-lightbox-thumbnail-size");
pub const LIGHTBOX_THUMBNAILS_GAP: CssVar = CssVar::new("--lsx-lightbox-thumbnails-gap");

/// No backdrop of its own: a lightbox dims like every other modal, through
/// `OverlayDefaults` (decided 2026-09-16, [[wont-do]]'s "Modal has no stylable
/// knobs").
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightboxDefaults {
    /// The dialog's widest extent, a CSS length. Below the `sm` breakpoint,
    /// or under 30rem tall (a phone on its side), the viewer is full screen
    /// instead, inside the safe-area insets.
    pub width: &'static str,
    /// The stage's height, a CSS length. Fixed, so every slide is the same
    /// box and a picture is fitted into it rather than sizing it. A picture
    /// smaller than the stage keeps its natural size. Full screen, the stage
    /// is the room the close button, caption and thumbnails leave.
    pub stage_height: &'static str,
    /// One thumbnail's edge, a CSS length. The strip is a `Carousel`, so this
    /// caps the strip at `thumbnails_per_view` of them rather than sizing
    /// each directly; a narrower dialog shrinks them.
    pub thumbnail_size: &'static str,
    /// Thumbnails visible at once before the strip scrolls.
    pub thumbnails_per_view: f64,
    pub thumbnails_gap: Size,
    /// The upper scale bound, `1.0` being the picture as first shown: scaled
    /// down into the stage, never above its natural size.
    pub max_zoom: f64,
}

impl LightboxDefaults {
    pub const DEFAULT: Self = Self {
        width: "90vw",
        stage_height: "70vh",
        thumbnail_size: "64px",
        thumbnails_per_view: 7.0,
        thumbnails_gap: Size::Xs,
        max_zoom: 3.0,
    };
}

impl ToCssDeclarations for LightboxDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            LIGHTBOX_WIDTH.declare(self.width),
            LIGHTBOX_STAGE_HEIGHT.declare(self.stage_height),
            LIGHTBOX_THUMBNAIL_SIZE.declare(self.thumbnail_size),
            LIGHTBOX_THUMBNAILS_GAP.declare(SizeCss::SPACING.value(self.thumbnails_gap)),
        ]
    }
}
