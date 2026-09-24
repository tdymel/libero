use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size, SizeCss, Sizes};
use crate::tokens::Color;

pub const RATING_GLYPH_SIZE: SizeCss = SizeCss::new("--lsx-rating-glyph-size-");
pub const RATING_GAP_SIZE: SizeCss = SizeCss::new("--lsx-rating-gap-size-");

// Resolved on the root; every symbol inherits it.
pub const RATING_GLYPH: CssVar = CssVar::new("--lsx-rating-glyph");
pub const RATING_GAP: CssVar = CssVar::new("--lsx-rating-gap");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RatingSizeLevel {
    /// One symbol's width and height.
    pub glyph_size: &'static str,
    /// Space between two symbols; half of it pads each side, inside the hit area.
    pub gap: &'static str,
}

/// Theme defaults for `Rating`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RatingDefaults {
    pub size: Size,
    pub sizes: Sizes<RatingSizeLevel>,
    /// The filled symbols' colour.
    pub color: Color,
}

impl RatingDefaults {
    /// `Md`: a 24px symbol plus 4px gap, so a whole symbol is a 28px target (WCAG 2.5.8).
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        sizes: Sizes::new(
            RatingSizeLevel {
                glyph_size: "16px",
                gap: "2px",
            },
            RatingSizeLevel {
                glyph_size: "20px",
                gap: "4px",
            },
            RatingSizeLevel {
                glyph_size: "24px",
                gap: "4px",
            },
            RatingSizeLevel {
                glyph_size: "28px",
                gap: "6px",
            },
            RatingSizeLevel {
                glyph_size: "32px",
                gap: "6px",
            },
            RatingSizeLevel {
                glyph_size: "40px",
                gap: "8px",
            },
        ),
        color: Color::Warning,
    };

    pub fn size_sx(size: Size) -> Sx {
        sx().var(RATING_GLYPH, RATING_GLYPH_SIZE.value(size))
            .var(RATING_GAP, RATING_GAP_SIZE.value(size))
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for RatingDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = Vec::new();
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(RATING_GLYPH_SIZE.declare(size, level.glyph_size));
            declarations.push(RATING_GAP_SIZE.declare(size, level.gap));
        }
        declarations
    }
}
