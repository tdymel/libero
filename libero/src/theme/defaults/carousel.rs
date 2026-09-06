use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::theme::{Color, ColorShade, ColorValue, CssVar, Size, SizeCss};

pub const CAROUSEL_GAP: CssVar = CssVar::new("--lsx-carousel-gap");
pub const CAROUSEL_PER_VIEW: CssVar = CssVar::new("--lsx-carousel-per-view");
pub const CAROUSEL_RADIUS: CssVar = CssVar::new("--lsx-carousel-radius");
pub const CAROUSEL_CONTROL_SIZE: CssVar = CssVar::new("--lsx-carousel-control-size");
pub const CAROUSEL_CONTROLS_OFFSET: CssVar = CssVar::new("--lsx-carousel-controls-offset");
pub const CAROUSEL_INDICATOR_LENGTH: CssVar = CssVar::new("--lsx-carousel-indicator-length");
/// The current dot's length. A second channel beside the colour, since the two
/// colours are close in luminance.
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
    /// Where a snapped slide comes to rest inside the viewport - CSS
    /// `scroll-snap-align`.
    #[state_prefix = "align"]
    pub enum CarouselAlign {
        Start = "start",
        #[default]
        Center = "center",
        End = "end",
    }
}

/// Lives here rather than in the component because an enum prop with a themed
/// default has to sit below `components` in the layer order - the
/// `QrRobustness` precedent.
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
    /// An idle dot. It is a button carrying the only visible position
    /// affordance, so SC 1.4.11 asks 3:1 of it: `grey.6` is 3.32:1 on white,
    /// `grey.5` about 2.0:1 and `grey.4` 1.49:1. Retheme it with that in hand.
    pub indicator_color: ColorValue,
    pub indicator_current_color: ColorValue,
    /// The previous/next and pause buttons' fill. They sit over the slides,
    /// not on the page, so this is their own surface rather than the paper's.
    pub control_background: ColorValue,
    /// A previous/next button's fill under the pointer. The pause button has
    /// no hover arm.
    pub control_hover_background: ColorValue,
    /// The glyph on every control. It is also the controls' focus-ring
    /// colour: a var is opaque to `sx`, so `background()` no longer publishes
    /// the ring's `--lsx-focus-contrast`, and this is what reads against
    /// `control_background`. Change the two together.
    pub control_color: ColorValue,
    /// Milliseconds between automatic advances, when `autoplay` is on.
    pub autoplay_delay: u32,
    /// English literals, the `DateDefaults` precedent. The library has no i18n
    /// mechanism yet (todo 28); a caller overrides these on the theme, or
    /// passes `aria_label`/`slide_label` per instance.
    pub label: &'static str,
    pub previous_label: &'static str,
    pub next_label: &'static str,
    /// `{n}` is replaced with the slide number.
    pub indicator_label: &'static str,
    /// `{n}` of `{m}` - a slide group's accessible name.
    pub slide_label: &'static str,
    /// What the live region reads when the slide settles.
    pub status_label: &'static str,
    /// The autoplay button's name. It stays the same whether the slideshow
    /// runs or not: `aria-pressed` carries the state, so it never reads
    /// "Play, pressed".
    pub pause_label: &'static str,
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
        indicator_color: ColorValue::Shade(Color::Grey, ColorShade::S6),
        indicator_current_color: ColorValue::Shade(Color::Primary, ColorShade::S6),
        control_background: ColorValue::Shade(Color::White, ColorShade::S1),
        control_hover_background: ColorValue::Shade(Color::Grey, ColorShade::S1),
        control_color: ColorValue::Shade(Color::Grey, ColorShade::S7),
        autoplay_delay: 4000,
        label: "Carousel",
        previous_label: "Previous slide",
        next_label: "Next slide",
        indicator_label: "Go to slide {n}",
        slide_label: "{n} of {m}",
        status_label: "Slide {n} of {m}",
        pause_label: "Pause slideshow",
    };

    /// `"{n} of {m}"` with both holes filled. One-based, because it is read
    /// aloud.
    pub(crate) fn format_label(template: &str, index: usize, count: usize) -> String {
        template
            .replace("{n}", &(index + 1).to_string())
            .replace("{m}", &count.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_label_template_fills_both_holes_one_based() {
        assert_eq!(
            CarouselDefaults::format_label("Slide {n} of {m}", 2, 7),
            "Slide 3 of 7"
        );
        assert_eq!(
            CarouselDefaults::format_label("Go to slide {n}", 0, 7),
            "Go to slide 1"
        );
    }

    /// A template with no hole is a legitimate override, not an error.
    #[test]
    fn a_template_without_holes_is_returned_unchanged() {
        assert_eq!(CarouselDefaults::format_label("Bild", 4, 9), "Bild");
    }
}
