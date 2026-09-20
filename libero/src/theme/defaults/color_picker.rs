use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const COLOR_PICKER_WIDTH_SIZE: SizeCss = SizeCss::new("--lsx-color-picker-width-");
pub const COLOR_PICKER_SATURATION_HEIGHT_SIZE: SizeCss =
    SizeCss::new("--lsx-color-picker-saturation-height-");
pub const COLOR_PICKER_THUMB_SIZE: SizeCss = SizeCss::new("--lsx-color-picker-thumb-size-");
pub const COLOR_PICKER_PREVIEW_SIZE: SizeCss = SizeCss::new("--lsx-color-picker-preview-size-");
pub const COLOR_PICKER_SPACING_SIZE: SizeCss = SizeCss::new("--lsx-color-picker-spacing-");
pub const COLOR_PICKER_SWATCH_SIZE: SizeCss = SizeCss::new("--lsx-color-picker-swatch-size-");

// The picked level, resolved on the root so panel, sliders and swatches inherit it.
pub const COLOR_PICKER_WIDTH: CssVar = CssVar::new("--lsx-color-picker-width");
pub const COLOR_PICKER_SATURATION_HEIGHT: CssVar =
    CssVar::new("--lsx-color-picker-saturation-height");
pub const COLOR_PICKER_THUMB: CssVar = CssVar::new("--lsx-color-picker-thumb");
pub const COLOR_PICKER_PREVIEW: CssVar = CssVar::new("--lsx-color-picker-preview");
pub const COLOR_PICKER_SPACING: CssVar = CssVar::new("--lsx-color-picker-spacing");
pub const COLOR_PICKER_SWATCH: CssVar = CssVar::new("--lsx-color-picker-swatch");

str_enum! {
    /// The CSS text a colour is written as; a `ColorCode` is the same in every form.
    pub enum ColorFormat {
        #[default]
        Hex = "hex",
        Hexa = "hexa",
        Rgb = "rgb",
        Rgba = "rgba",
        Hsl = "hsl",
        Hsla = "hsla",
    }
}

impl ColorFormat {
    /// Whether the form can write a translucent color.
    pub const fn has_alpha(self) -> bool {
        matches!(self, Self::Hexa | Self::Rgba | Self::Hsla)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorPickerSizeLevel {
    /// The picker's own width, unless `full_width`.
    pub width: &'static str,
    pub saturation_height: &'static str,
    /// The thumb's diameter, and the height of the hue and alpha tracks.
    pub thumb_size: &'static str,
    /// The swatch beside the sliders.
    pub preview_size: &'static str,
    /// Between the panel, the sliders and the swatches.
    pub spacing: &'static str,
    /// One preset swatch, fixed per step: seven fit `width`; `full_width` does not grow them.
    pub swatch_size: &'static str,
}

/// Theme defaults for `ColorPicker`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorPickerDefaults {
    pub size: Size,
    /// Caps swatches per row. `None` wraps them to the picker's width.
    pub swatches_per_row: Option<usize>,
    /// Corner radius of the swatches and the preview. Round by default.
    pub radius: Size,
    pub sizes: Sizes<ColorPickerSizeLevel>,
}

impl ColorPickerDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        swatches_per_row: None,
        radius: Size::Xxl,
        sizes: Sizes::new(
            ColorPickerSizeLevel {
                width: "180px",
                saturation_height: "100px",
                thumb_size: "8px",
                preview_size: "26px",
                // WCAG 2.5.8: a 20px swatch passes only at 24px centre to centre,
                // so neither value can shrink (pinned by a test).
                spacing: "4px",
                swatch_size: "20px",
            },
            ColorPickerSizeLevel {
                width: "200px",
                saturation_height: "110px",
                thumb_size: "12px",
                preview_size: "34px",
                spacing: "6px",
                swatch_size: "22px",
            },
            ColorPickerSizeLevel {
                width: "240px",
                saturation_height: "120px",
                thumb_size: "16px",
                preview_size: "42px",
                spacing: "8px",
                swatch_size: "26px",
            },
            ColorPickerSizeLevel {
                width: "280px",
                saturation_height: "140px",
                thumb_size: "20px",
                preview_size: "50px",
                spacing: "10px",
                swatch_size: "30px",
            },
            ColorPickerSizeLevel {
                width: "320px",
                saturation_height: "160px",
                thumb_size: "22px",
                preview_size: "54px",
                spacing: "12px",
                swatch_size: "34px",
            },
            ColorPickerSizeLevel {
                width: "360px",
                saturation_height: "180px",
                thumb_size: "26px",
                preview_size: "62px",
                spacing: "14px",
                swatch_size: "38px",
            },
        ),
    };

    pub fn size_sx(size: Size) -> Sx {
        sx().var(COLOR_PICKER_WIDTH, COLOR_PICKER_WIDTH_SIZE.value(size))
            .var(
                COLOR_PICKER_SATURATION_HEIGHT,
                COLOR_PICKER_SATURATION_HEIGHT_SIZE.value(size),
            )
            .var(COLOR_PICKER_THUMB, COLOR_PICKER_THUMB_SIZE.value(size))
            .var(COLOR_PICKER_PREVIEW, COLOR_PICKER_PREVIEW_SIZE.value(size))
            .var(COLOR_PICKER_SPACING, COLOR_PICKER_SPACING_SIZE.value(size))
            .var(COLOR_PICKER_SWATCH, COLOR_PICKER_SWATCH_SIZE.value(size))
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for ColorPickerDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = Vec::new();
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(COLOR_PICKER_WIDTH_SIZE.declare(size, level.width));
            declarations
                .push(COLOR_PICKER_SATURATION_HEIGHT_SIZE.declare(size, level.saturation_height));
            declarations.push(COLOR_PICKER_THUMB_SIZE.declare(size, level.thumb_size));
            declarations.push(COLOR_PICKER_PREVIEW_SIZE.declare(size, level.preview_size));
            declarations.push(COLOR_PICKER_SPACING_SIZE.declare(size, level.spacing));
            declarations.push(COLOR_PICKER_SWATCH_SIZE.declare(size, level.swatch_size));
        }
        declarations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn px(value: &str) -> f64 {
        value
            .strip_suffix("px")
            .and_then(|n| n.parse().ok())
            .unwrap()
    }

    /// WCAG 2.5.8: a swatch under 24px passes only if its neighbours' centres
    /// are 24px away. `xs` is the tightest step, and it sits exactly on it.
    #[test]
    fn the_swatches_sit_24px_apart_centre_to_centre_at_every_step() {
        for size in Size::ALL {
            let level = ColorPickerDefaults::DEFAULT.sizes.get(size);
            let pitch = px(level.swatch_size) + px(level.spacing);
            assert!(pitch >= 24.0, "{size:?}: swatches sit {pitch}px apart");
        }
    }
}
