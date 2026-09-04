use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::sx::{Sx, sx};
use crate::theme::{Color, ColorShade, ColorValue, CssVar, Size, SizeCss, Sizes};

pub const STEPPER_MARKER_SIZE: SizeCss = SizeCss::new("--lsx-stepper-marker-");
pub const STEPPER_FONT_SIZE: SizeCss = SizeCss::new("--lsx-stepper-font-size-");
pub const STEPPER_DESCRIPTION_FONT_SIZE: SizeCss =
    SizeCss::new("--lsx-stepper-description-font-size-");
pub const STEPPER_GAP_SIZE: SizeCss = SizeCss::new("--lsx-stepper-gap-");
pub const STEPPER_SPACING_SIZE: SizeCss = SizeCss::new("--lsx-stepper-spacing-");

// The picked level, resolved on the root so the steps and their parts - which
// carry no size `data-state` of their own - inherit it.
pub const STEPPER_MARKER: CssVar = CssVar::new("--lsx-stepper-marker");
pub const STEPPER_DESCRIPTION_SIZE: CssVar = CssVar::new("--lsx-stepper-description-size");
pub const STEPPER_GAP: CssVar = CssVar::new("--lsx-stepper-gap");
pub const STEPPER_SPACING: CssVar = CssVar::new("--lsx-stepper-spacing");

pub const STEPPER_COLOR: CssVar = CssVar::new("--lsx-stepper-color");
pub const STEPPER_COLOR_CONTRAST: CssVar = CssVar::new("--lsx-stepper-color-contrast");
pub const STEPPER_PENDING: CssVar = CssVar::new("--lsx-stepper-pending");
pub const STEPPER_ERROR: CssVar = CssVar::new("--lsx-stepper-error");
pub const STEPPER_ERROR_CONTRAST: CssVar = CssVar::new("--lsx-stepper-error-contrast");
pub const STEPPER_CONNECTOR_COLOR: CssVar = CssVar::new("--lsx-stepper-connector-color");
pub const STEPPER_DESCRIPTION_COLOR: CssVar = CssVar::new("--lsx-stepper-description-color");
pub const STEPPER_LINE_WIDTH: CssVar = CssVar::new("--lsx-stepper-line-width");
pub const STEPPER_CONTENT_PADDING: CssVar = CssVar::new("--lsx-stepper-content-padding");

str_enum! {
    /// Where a step's label sits relative to its marker in a horizontal
    /// stepper. A vertical stepper always puts it beside the marker.
    #[state_prefix = "label"]
    pub enum StepLabelPosition {
        #[default]
        Side = "side",
        Below = "below",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepperSizeLevel {
    /// The marker's diameter.
    pub marker: &'static str,
    pub font_size: &'static str,
    pub description_font_size: &'static str,
    /// Between a marker and its label.
    pub gap: &'static str,
    /// Between two steps: the space either side of a horizontal connector,
    /// and the length of a vertical one.
    pub spacing: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepperDefaults {
    pub size: Size,
    pub sizes: Sizes<StepperSizeLevel>,
    pub label_position: StepLabelPosition,
    /// The active and completed markers, and the connector behind a completed
    /// step. A filled marker's glyph takes its contrast twin.
    pub color: ColorValue,
    /// The ring of a step not reached yet.
    pub pending_color: ColorValue,
    pub error_color: ColorValue,
    pub connector_color: ColorValue,
    pub description_color: ColorValue,
    /// The connector's thickness, and the marker ring's.
    pub line_width: u8,
    /// Around a vertical step's content.
    pub content_padding: Size,
    /// Read by a screen reader after a completed step's label. The marker's
    /// check is drawing only.
    pub completed_label: &'static str,
    /// Read after an errored step's label, so the error is not colour alone.
    pub error_label: &'static str,
}

/// The auto-contrast twin of a shade, so a filled marker's glyph comes from
/// the same ramp as its fill.
pub(crate) fn contrast_of(color: ColorValue) -> ColorValue {
    match color {
        ColorValue::Shade(color, shade) => ColorValue::Contrast(color, shade),
        other => other,
    }
}

impl StepperDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        sizes: Sizes::new(
            StepperSizeLevel {
                marker: "24px",
                font_size: "12px",
                description_font_size: "11px",
                gap: "6px",
                spacing: "8px",
            },
            StepperSizeLevel {
                marker: "28px",
                font_size: "13px",
                description_font_size: "12px",
                gap: "8px",
                spacing: "12px",
            },
            StepperSizeLevel {
                marker: "32px",
                font_size: "14px",
                description_font_size: "12px",
                gap: "10px",
                spacing: "16px",
            },
            StepperSizeLevel {
                marker: "36px",
                font_size: "16px",
                description_font_size: "14px",
                gap: "12px",
                spacing: "20px",
            },
            StepperSizeLevel {
                marker: "40px",
                font_size: "18px",
                description_font_size: "16px",
                gap: "14px",
                spacing: "24px",
            },
            StepperSizeLevel {
                marker: "48px",
                font_size: "20px",
                description_font_size: "18px",
                gap: "16px",
                spacing: "28px",
            },
        ),
        label_position: StepLabelPosition::Side,
        color: ColorValue::Shade(Color::Primary, ColorShade::S6),
        pending_color: ColorValue::Shade(Color::Grey, ColorShade::S3),
        error_color: ColorValue::Shade(Color::Error, ColorShade::S6),
        connector_color: ColorValue::Shade(Color::Grey, ColorShade::S3),
        description_color: ColorValue::Shade(Color::Grey, ColorShade::S7),
        line_width: 2,
        content_padding: Size::Md,
        completed_label: "Completed",
        error_label: "Error",
    };

    pub fn size_sx(size: Size) -> Sx {
        sx().font_size(STEPPER_FONT_SIZE.value(size))
            .var(STEPPER_MARKER, STEPPER_MARKER_SIZE.value(size))
            .var(
                STEPPER_DESCRIPTION_SIZE,
                STEPPER_DESCRIPTION_FONT_SIZE.value(size),
            )
            .var(STEPPER_GAP, STEPPER_GAP_SIZE.value(size))
            .var(STEPPER_SPACING, STEPPER_SPACING_SIZE.value(size))
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for StepperDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        // `.value()` on every colour, never the field itself: a raw
        // `primary.6` reaches `:root` as a bare token and silently drops every
        // shorthand that reads it. See `TimelineDefaults`.
        let mut declarations = vec![
            STEPPER_COLOR.declare(self.color.value()),
            STEPPER_COLOR_CONTRAST.declare(contrast_of(self.color).value()),
            STEPPER_PENDING.declare(self.pending_color.value()),
            STEPPER_ERROR.declare(self.error_color.value()),
            STEPPER_ERROR_CONTRAST.declare(contrast_of(self.error_color).value()),
            STEPPER_CONNECTOR_COLOR.declare(self.connector_color.value()),
            STEPPER_DESCRIPTION_COLOR.declare(self.description_color.value()),
            STEPPER_LINE_WIDTH.declare(format!("{}px", self.line_width)),
            STEPPER_CONTENT_PADDING.declare(SizeCss::SPACING.value(self.content_padding)),
        ];
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(STEPPER_MARKER_SIZE.declare(size, level.marker));
            declarations.push(STEPPER_FONT_SIZE.declare(size, level.font_size));
            declarations
                .push(STEPPER_DESCRIPTION_FONT_SIZE.declare(size, level.description_font_size));
            declarations.push(STEPPER_GAP_SIZE.declare(size, level.gap));
            declarations.push(STEPPER_SPACING_SIZE.declare(size, level.spacing));
        }
        declarations
    }
}
