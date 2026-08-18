use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};

use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const TEXT_FONT_FAMILY: CssVar = CssVar::new("--lsx-text-font-family");

pub const TEXT_FONT_WEIGHT: SizeCss = SizeCss::new("--lsx-text-font-weight-");
pub const TEXT_FONT_SIZE: SizeCss = SizeCss::new("--lsx-text-font-size-");
pub const TEXT_LETTER_SPACING: SizeCss = SizeCss::new("--lsx-text-letter-spacing-");
pub const TEXT_LINE_HEIGHT: SizeCss = SizeCss::new("--lsx-text-line-height-");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextSize {
    pub font_weight: &'static str,
    pub font_size: &'static str, // in rem
    pub letter_spacing: &'static str,
    pub line_height: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextDefaults {
    pub font_family: &'static str,
    pub sizes: Sizes<TextSize>,
}

impl TextDefaults {
    /// Font-size/weight/letter-spacing/line-height for `size` - everything
    /// about a `Text` that actually varies by size. Font-family doesn't, so
    /// it's not part of this - see `TEXT_FONT_FAMILY`.
    pub fn size_sx(size: Size) -> Sx {
        sx().font_size(TEXT_FONT_SIZE.value(size))
            .font_weight(TEXT_FONT_WEIGHT.value(size))
            .letter_spacing(TEXT_LETTER_SPACING.value(size))
            .line_height(TEXT_LINE_HEIGHT.value(size))
    }

    /// Font-family plus every size's declarations at once, each size gated
    /// behind its own `[data-state~="size-md"]`-style selector (see
    /// `Size::state_name`) - one static/framework `Sx` that every `Text`
    /// instance shares regardless of which size it's actually using; only
    /// the `data-state` attribute (set per-instance) picks which size block
    /// applies, so choosing a size never costs a new dynamically-generated
    /// CSS class.
    pub fn theme_vars() -> Sx {
        Size::ALL
            .into_iter()
            .fold(sx().font_family(TEXT_FONT_FAMILY.value()), |base, size| {
                base.when(size.state_name(), Self::size_sx(size))
            })
    }
}

impl ToCssDeclarations for TextDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = vec![TEXT_FONT_FAMILY.declare(self.font_family)];
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(TEXT_FONT_WEIGHT.declare(size, level.font_weight));
            declarations.push(TEXT_FONT_SIZE.declare(size, level.font_size));
            declarations.push(TEXT_LETTER_SPACING.declare(size, level.letter_spacing));
            declarations.push(TEXT_LINE_HEIGHT.declare(size, level.line_height));
        }
        declarations
    }
}
