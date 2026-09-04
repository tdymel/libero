use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Color, CssVar, Size, SizeCss, Sizes};

pub const INDICATOR_SIZE: SizeCss = SizeCss::new("--lsx-indicator-size-");
pub const INDICATOR_FONT_SIZE: SizeCss = SizeCss::new("--lsx-indicator-font-size-");

/// The active step's diameter and label size, republished unsuffixed by
/// [`IndicatorDefaults::size_sx`] - the `Badge` pattern. The base rule reads
/// these two and needs to know nothing about which size token is on.
pub const INDICATOR_BOX: CssVar = CssVar::new("--lsx-indicator-box");
pub const INDICATOR_FONT: CssVar = CssVar::new("--lsx-indicator-font");

pub const INDICATOR_RADIUS: CssVar = CssVar::new("--lsx-indicator-radius");
pub const INDICATOR_BORDER_WIDTH: CssVar = CssVar::new("--lsx-indicator-border-width");
pub const INDICATOR_PROCESSING_DURATION: CssVar =
    CssVar::new("--lsx-indicator-processing-duration");

/// The `processing` ping, appended to the stylesheet beside `LOADER_KEYFRAMES`.
///
/// One name, unlike `RIPPLE_KEYFRAMES`' two: the ping runs for as long as the
/// state is on and never has to restart.
pub const INDICATOR_KEYFRAMES: &str = concat!(
    "@keyframes lsx-indicator-processing{from{opacity:0.6;transform:scale(0);}",
    "to{opacity:0;transform:scale(2.8);}}"
);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IndicatorSizeLevel {
    /// The dot's diameter, and the height and minimum width of a labelled one.
    pub size: &'static str,
    pub font_size: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IndicatorDefaults {
    pub size: Size,
    /// A dot that is not about something the reader should look at is not
    /// worth drawing, so the default is the error role - Mantine's too.
    pub color: Color,
    /// A CSS length, off the radius scale on purpose: a dot is round at every
    /// diameter, which no fixed step gives. `Badge`'s pill, same reason.
    pub radius: &'static str,
    /// Above this a count renders as `{max}+`. A house convention, set once
    /// here rather than on every call site.
    pub max: u32,
    /// The `with_border` ring.
    pub border_width: &'static str,
    pub processing_duration: &'static str,
    /// px throughout, label included: the box is a fixed px height, and a
    /// label that followed the reader's text size would outgrow it.
    pub sizes: Sizes<IndicatorSizeLevel>,
}

impl IndicatorDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        color: Color::Error,
        radius: "9999px",
        max: 99,
        border_width: "2px",
        processing_duration: "1000ms",
        sizes: Sizes::new(
            IndicatorSizeLevel {
                size: "6px",
                font_size: "8px",
            },
            IndicatorSizeLevel {
                size: "8px",
                font_size: "9px",
            },
            IndicatorSizeLevel {
                size: "10px",
                font_size: "10px",
            },
            IndicatorSizeLevel {
                size: "14px",
                font_size: "11px",
            },
            IndicatorSizeLevel {
                size: "18px",
                font_size: "12px",
            },
            IndicatorSizeLevel {
                size: "22px",
                font_size: "14px",
            },
        ),
    };

    pub fn size_sx(size: Size) -> Sx {
        sx().var(INDICATOR_BOX, INDICATOR_SIZE.value(size))
            .var(INDICATOR_FONT, INDICATOR_FONT_SIZE.value(size))
    }

    pub fn theme_vars() -> Sx {
        sx().height(INDICATOR_BOX.value())
            .min_width(INDICATOR_BOX.value())
            .line_height(INDICATOR_BOX.value())
            .font_size(INDICATOR_FONT.value())
            .border_radius(INDICATOR_RADIUS.overridable())
            .per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for IndicatorDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = Vec::new();
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(INDICATOR_SIZE.declare(size, level.size));
            declarations.push(INDICATOR_FONT_SIZE.declare(size, level.font_size));
        }
        declarations.push(INDICATOR_RADIUS.declare(self.radius));
        declarations.push(INDICATOR_BORDER_WIDTH.declare(self.border_width));
        declarations.push(INDICATOR_PROCESSING_DURATION.declare(self.processing_duration));
        declarations
    }
}
