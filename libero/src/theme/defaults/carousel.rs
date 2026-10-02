use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::theme::{Color, ColorShade, ColorValue, CssVar, Size, SizeCss};

pub const CAROUSEL_GAP: CssVar = CssVar::new("--lsx-carousel-gap");
pub const CAROUSEL_PER_VIEW: CssVar = CssVar::new("--lsx-carousel-per-view");
pub const CAROUSEL_RADIUS: CssVar = CssVar::new("--lsx-carousel-radius");
pub const CAROUSEL_CONTROL_SIZE: CssVar = CssVar::new("--lsx-carousel-control-size");
pub const CAROUSEL_CONTROLS_OFFSET: CssVar = CssVar::new("--lsx-carousel-controls-offset");
pub const CAROUSEL_INDICATOR_LENGTH: CssVar = CssVar::new("--lsx-carousel-indicator-length");
/// The current dot's length: a second channel, the two colours are close in luminance.
pub const CAROUSEL_INDICATOR_CURRENT_LENGTH: CssVar =
    CssVar::new("--lsx-carousel-indicator-current-length");
pub const CAROUSEL_INDICATOR_THICKNESS: CssVar = CssVar::new("--lsx-carousel-indicator-thickness");
pub const CAROUSEL_INDICATORS_GAP: CssVar = CssVar::new("--lsx-carousel-indicators-gap");
pub const CAROUSEL_INDICATOR_COLOR: CssVar = CssVar::new("--lsx-carousel-indicator-color");
pub const CAROUSEL_INDICATOR_CURRENT_COLOR: CssVar =
    CssVar::new("--lsx-carousel-indicator-current-color");
pub const CAROUSEL_CONTROL_BACKGROUND: CssVar = CssVar::new("--lsx-carousel-control-background");
pub const CAROUSEL_CONTROL_HOVER_BACKGROUND: CssVar =
    CssVar::new("--lsx-carousel-control-hover-background");
pub const CAROUSEL_CONTROL_COLOR: CssVar = CssVar::new("--lsx-carousel-control-color");

str_enum! {
    /// Where a snapped slide rests in the viewport: CSS `scroll-snap-align`.
    #[state_prefix = "align"]
    pub enum CarouselAlign {
        Start = "start",
        #[default]
        Center = "center",
        End = "end",
    }
}

/// Theme defaults for `Carousel`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarouselDefaults {
    /// Slides visible at once. Fractional peeks the next one.
    pub per_view: f64,
    pub gap: Size,
    pub align: CarouselAlign,
    /// Corner radius of a slide.
    pub radius: Size,
    pub controls: bool,
    pub indicators: bool,
    pub control_size: &'static str,
    pub controls_offset: Size,
    pub indicator_length: &'static str,
    /// The current dot's length, so position is not carried by hue alone.
    pub indicator_current_length: &'static str,
    pub indicator_thickness: &'static str,
    pub indicators_gap: &'static str,
    /// An idle dot. SC 1.4.11 asks 3:1: `muted.6` is 3.32:1 on white, `muted.5` about 2.0:1.
    pub indicator_color: ColorValue,
    pub indicator_current_color: ColorValue,
    /// The previous/next and pause buttons' fill, over the slides.
    pub control_background: ColorValue,
    /// A previous/next button's fill under the pointer; pause has no hover arm.
    pub control_hover_background: ColorValue,
    /// The controls' glyph and focus-ring colour; change it with `control_background`.
    pub control_color: ColorValue,
    /// Milliseconds between automatic advances, when `autoplay` is on.
    pub autoplay_delay: u32,
}

impl ToCssDeclarations for CarouselDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            CAROUSEL_GAP.declare(SizeCss::SPACING.value(self.gap)),
            // Unitless on purpose: the slide-size formula divides by it.
            CAROUSEL_PER_VIEW.declare(self.per_view.to_string()),
            CAROUSEL_RADIUS.declare(SizeCss::RADIUS.value(self.radius)),
            CAROUSEL_CONTROL_SIZE.declare(self.control_size),
            CAROUSEL_CONTROLS_OFFSET.declare(SizeCss::SPACING.value(self.controls_offset)),
            CAROUSEL_INDICATOR_LENGTH.declare(self.indicator_length),
            CAROUSEL_INDICATOR_CURRENT_LENGTH.declare(self.indicator_current_length),
            CAROUSEL_INDICATOR_THICKNESS.declare(self.indicator_thickness),
            CAROUSEL_INDICATORS_GAP.declare(self.indicators_gap),
            CAROUSEL_INDICATOR_COLOR.declare(self.indicator_color.value()),
            CAROUSEL_INDICATOR_CURRENT_COLOR.declare(self.indicator_current_color.value()),
            CAROUSEL_CONTROL_BACKGROUND.declare(self.control_background.value()),
            CAROUSEL_CONTROL_HOVER_BACKGROUND.declare(self.control_hover_background.value()),
            CAROUSEL_CONTROL_COLOR.declare(self.control_color.value()),
        ]
    }
}

impl CarouselDefaults {
    pub const DEFAULT: Self = Self {
        per_view: 1.0,
        gap: Size::Md,
        align: CarouselAlign::Center,
        radius: Size::Sm,
        controls: true,
        indicators: false,
        control_size: "28px",
        controls_offset: Size::Sm,
        indicator_length: "24px",
        indicator_current_length: "40px",
        indicator_thickness: "5px",
        indicators_gap: "8px",
        indicator_color: ColorValue::Shade(Color::Muted, ColorShade::S6),
        indicator_current_color: ColorValue::Shade(Color::Primary, ColorShade::S6),
        control_background: ColorValue::Shade(Color::Surface, ColorShade::S1),
        control_hover_background: ColorValue::Shade(Color::Muted, ColorShade::S1),
        control_color: ColorValue::Shade(Color::Muted, ColorShade::S7),
        autoplay_delay: 4000,
    };
}
