use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};

use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const TITLE_FONT_FAMILY: CssVar = CssVar::new("--lsx-title-font-family");

pub const TITLE_FONT_WEIGHT: SizeCss = SizeCss::new("--lsx-title-font-weight-");
pub const TITLE_FONT_SIZE: SizeCss = SizeCss::new("--lsx-title-font-size-");
pub const TITLE_LETTER_SPACING: SizeCss = SizeCss::new("--lsx-title-letter-spacing-");
pub const TITLE_LINE_HEIGHT: SizeCss = SizeCss::new("--lsx-title-line-height-");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TitleSize {
    pub font_weight: &'static str,
    pub font_size: &'static str, // in rem
    pub letter_spacing: &'static str,
    pub line_height: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TitleDefaults {
    pub font_family: &'static str,
    pub sizes: Sizes<TitleSize>,
}

impl TitleDefaults {
    /// Font-size/weight/letter-spacing/line-height for `size` - everything
    /// about a `Title` that actually varies by size. Font-family doesn't,
    /// so it's not part of this - see `TITLE_FONT_FAMILY`.
    pub fn size_sx(size: Size) -> Sx {
        sx().font_size(TITLE_FONT_SIZE.value(size))
            .font_weight(TITLE_FONT_WEIGHT.value(size))
            .letter_spacing(TITLE_LETTER_SPACING.value(size))
            .line_height(TITLE_LINE_HEIGHT.value(size))
    }

    /// Font-family plus every size's declarations at once, each size gated
    /// behind its own `[data-state~="size-md"]`-style selector (see
    /// `Size::state_name`) - one static/framework `Sx` that every `Title`
    /// instance shares regardless of which size it's actually using; only
    /// the `data-state` attribute (set per-instance) picks which size block
    /// applies, so choosing a size never costs a new dynamically-generated
    /// CSS class.
    pub fn theme_vars() -> Sx {
        sx().font_family(TITLE_FONT_FAMILY.value())
            .per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for TitleDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = vec![TITLE_FONT_FAMILY.declare(self.font_family)];
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(TITLE_FONT_WEIGHT.declare(size, level.font_weight));
            declarations.push(TITLE_FONT_SIZE.declare(size, level.font_size));
            declarations.push(TITLE_LETTER_SPACING.declare(size, level.letter_spacing));
            declarations.push(TITLE_LINE_HEIGHT.declare(size, level.line_height));
        }
        declarations
    }
}
