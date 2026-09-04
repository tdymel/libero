use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const SLIDER_TRACK_SIZE: SizeCss = SizeCss::new("--lsx-slider-track-size-");
pub const SLIDER_THUMB_SIZE: SizeCss = SizeCss::new("--lsx-slider-thumb-size-");
pub const SLIDER_FONT_SIZE: SizeCss = SizeCss::new("--lsx-slider-font-size-");

// The picked level, resolved on the root so the track/thumb/mark children -
// which carry no `data-state` of their own - can inherit it.
pub const SLIDER_TRACK: CssVar = CssVar::new("--lsx-slider-track");
pub const SLIDER_THUMB: CssVar = CssVar::new("--lsx-slider-thumb");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SliderSizeLevel {
    /// Track and filled-bar thickness.
    pub track_size: &'static str,
    pub thumb_size: &'static str,
    /// Label bubble and mark captions.
    pub font_size: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderDefaults {
    pub size: Size,
    pub sizes: Sizes<SliderSizeLevel>,
    /// Steps moved per arrow key press.
    pub step: f64,
    /// Steps moved per Shift+arrow, PageUp or PageDown.
    pub big_step: f64,
}

impl SliderDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        sizes: Sizes::new(
            SliderSizeLevel {
                track_size: "2px",
                thumb_size: "12px",
                font_size: "0.6875rem",
            },
            SliderSizeLevel {
                track_size: "3px",
                thumb_size: "14px",
                font_size: "0.75rem",
            },
            SliderSizeLevel {
                track_size: "4px",
                thumb_size: "16px",
                font_size: "0.8125rem",
            },
            SliderSizeLevel {
                track_size: "6px",
                thumb_size: "20px",
                font_size: "0.875rem",
            },
            SliderSizeLevel {
                track_size: "8px",
                thumb_size: "24px",
                font_size: "0.9375rem",
            },
            SliderSizeLevel {
                track_size: "10px",
                thumb_size: "28px",
                font_size: "1rem",
            },
        ),
        step: 1.0,
        big_step: 10.0,
    };

    pub fn size_sx(size: Size) -> Sx {
        sx().var(SLIDER_TRACK, SLIDER_TRACK_SIZE.value(size))
            .var(SLIDER_THUMB, SLIDER_THUMB_SIZE.value(size))
            .font_size(SLIDER_FONT_SIZE.value(size))
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for SliderDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = Vec::new();
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(SLIDER_TRACK_SIZE.declare(size, level.track_size));
            declarations.push(SLIDER_THUMB_SIZE.declare(size, level.thumb_size));
            declarations.push(SLIDER_FONT_SIZE.declare(size, level.font_size));
        }
        declarations
    }
}
