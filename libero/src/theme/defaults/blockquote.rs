use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};

use crate::theme::{Color, CssVar, Size, SizeCss, Sizes, TEXT_FONT_SIZE, TEXT_LINE_HEIGHT};

/// The tint behind the quote and its accent bar, set per instance from `color`.
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
    /// The accent bar's thickness.
    pub border_width: &'static str,
}

/// Theme defaults for `Blockquote`, set on [`Theme`](crate::theme::Theme).
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

    /// Uses `TextDefaults`' font scale, so retuning `theme.text` keeps quotes matching prose.
    pub fn size_sx(size: Size) -> Sx {
        sx().padding(format!(
            "{} {}",
            BLOCKQUOTE_PADDING_Y.value(size),
            BLOCKQUOTE_PADDING_X.value(size)
        ))
        .border_left(format!(
            "{} solid {}",
            BLOCKQUOTE_BORDER_WIDTH.value(size),
            BLOCKQUOTE_BORDER_COLOR.value()
        ))
        .font_size(TEXT_FONT_SIZE.value(size))
        .line_height(TEXT_LINE_HEIGHT.value(size))
    }

    /// Only the two corners away from the accent bar, which stays on the left in
    /// every writing direction (`Sx` has no logical corners).
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
