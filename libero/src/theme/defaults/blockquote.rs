use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};

use crate::theme::{Color, CssVar, Size, SizeCss, Sizes, TEXT_FONT_SIZE, TEXT_LINE_HEIGHT};

/// The tint behind the quote, and the accent bar down its edge. Both are set
/// per instance from the resolved `color`, so they are plain vars rather than
/// an `-override` pair.
pub const BLOCKQUOTE_BACKGROUND: CssVar = CssVar::new("--lsx-blockquote-background");
pub const BLOCKQUOTE_BORDER_COLOR: CssVar = CssVar::new("--lsx-blockquote-border-color");
/// Body text on the tint - the tint's `-contrast` twin, never an accent shade.
pub const BLOCKQUOTE_COLOR: CssVar = CssVar::new("--lsx-blockquote-color");
pub const BLOCKQUOTE_CITE_OPACITY: CssVar = CssVar::new("--lsx-blockquote-cite-opacity");

pub const BLOCKQUOTE_PADDING_Y: SizeCss = SizeCss::new("--lsx-blockquote-padding-y-");
pub const BLOCKQUOTE_PADDING_X: SizeCss = SizeCss::new("--lsx-blockquote-padding-x-");
pub const BLOCKQUOTE_BORDER_WIDTH: SizeCss = SizeCss::new("--lsx-blockquote-border-width-");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockquoteSizeLevel {
    pub padding_y: &'static str,
    pub padding_x: &'static str,
    /// The accent bar's thickness. Scales with `size` so the frame stays
    /// proportional at `xxl`, rather than Mantine's flat 3px.
    pub border_width: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockquoteDefaults {
    pub size: Size,
    pub radius: Size,
    pub color: Color,
    /// How far the attribution is dimmed below the quote itself.
    pub cite_opacity: &'static str,
    pub sizes: Sizes<BlockquoteSizeLevel>,
}

impl BlockquoteDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        radius: Size::Sm,
        color: Color::Primary,
        cite_opacity: "0.65",
        sizes: Sizes::new(
            BlockquoteSizeLevel {
                padding_y: "0.5rem",
                padding_x: "0.75rem",
                border_width: "2px",
            },
            BlockquoteSizeLevel {
                padding_y: "0.75rem",
                padding_x: "1rem",
                border_width: "2px",
            },
            BlockquoteSizeLevel {
                padding_y: "1rem",
                padding_x: "1.5rem",
                border_width: "3px",
            },
            BlockquoteSizeLevel {
                padding_y: "1.25rem",
                padding_x: "2rem",
                border_width: "3px",
            },
            BlockquoteSizeLevel {
                padding_y: "1.5rem",
                padding_x: "2.5rem",
                border_width: "4px",
            },
            BlockquoteSizeLevel {
                padding_y: "2rem",
                padding_x: "3rem",
                border_width: "5px",
            },
        ),
    };

    /// The font scale is `TextDefaults`', not one of our own: "scales the
    /// text" should mean the same scale everywhere, and a caller who retunes
    /// `theme.texts` gets quotes that still match their prose.
    pub fn size_sx(size: Size) -> Sx {
        sx().padding(format!(
            "{} {}",
            BLOCKQUOTE_PADDING_Y.value(size),
            BLOCKQUOTE_PADDING_X.value(size)
        ))
        // The shorthand rather than `border-left-width`, which has no `Sx`
        // builder: adding one would be a shared-file edit for a single
        // caller, and the colour var inherits from the instance either way.
        .border_left(format!(
            "{} solid {}",
            BLOCKQUOTE_BORDER_WIDTH.value(size),
            BLOCKQUOTE_BORDER_COLOR.value()
        ))
        .font_size(TEXT_FONT_SIZE.value(size))
        .line_height(TEXT_LINE_HEIGHT.value(size))
    }

    /// Only the two corners away from the accent bar. Rounding all four
    /// rounds off the bar itself, which reads as a mistake rather than as a
    /// radius.
    ///
    /// Physical corners, because `Sx` has no logical twins - the bar is on the
    /// left in every writing direction, which the docs lead says out loud.
    pub fn radius_sx(radius: Size) -> Sx {
        sx().border_top_right_radius(SizeCss::RADIUS.value(radius))
            .border_bottom_right_radius(SizeCss::RADIUS.value(radius))
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx).per_radius(Self::radius_sx)
    }
}

impl ToCssDeclarations for BlockquoteDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = vec![BLOCKQUOTE_CITE_OPACITY.declare(self.cite_opacity)];
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(BLOCKQUOTE_PADDING_Y.declare(size, level.padding_y));
            declarations.push(BLOCKQUOTE_PADDING_X.declare(size, level.padding_x));
            declarations.push(BLOCKQUOTE_BORDER_WIDTH.declare(size, level.border_width));
        }
        declarations
    }
}
