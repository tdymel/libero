use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::sx::{Sx, sx};
use crate::theme::{Color, CssVar, Size, SizeCss, Sizes};

/// The square edge per step; all geometry is a fraction of the active one.
pub const LOADER_SIZE_SCALE: SizeCss = SizeCss::new("--lsx-loader-size-");

/// The active step's edge, republished unsuffixed by [`LoaderDefaults::size_sx`].
pub const LOADER_SIZE: CssVar = CssVar::new("--lsx-loader-size");

/// The ink, per instance via `variables()`, never on `:root`.
pub const LOADER_COLOR: CssVar = CssVar::new("--lsx-loader-color");

/// One `@keyframes` per variant, each from the *hidden* end to the *visible* one:
/// `animation: none` would leave `bars` invisible under reduced motion.
pub const LOADER_KEYFRAMES: &str = concat!(
    "@keyframes lsx-loader-oval{from{transform:rotate(0deg);}to{transform:rotate(360deg);}}",
    "@keyframes lsx-loader-bars{from{transform:scale(0.6);opacity:0;}",
    "to{transform:scale(1);opacity:1;}}",
    // Dots loop back to full size instead of snapping there.
    "@keyframes lsx-loader-dots{0%,100%{transform:scale(1);opacity:1;}",
    "50%{transform:scale(0.6);opacity:0.5;}}"
);

str_enum! {
    /// Which shape the loader draws.
    pub enum LoaderVariant {
        /// A ring with a gap, rotating.
        #[default]
        Oval = "oval",
        /// Three bars rising in sequence.
        Bars = "bars",
        /// Three dots pulsing.
        Dots = "dots",
    }
}

/// Theme defaults for `Loader`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoaderDefaults {
    pub variant: LoaderVariant,
    pub size: Size,
    pub color: Color,
    /// px throughout: a spinner is a glyph, not text, so it ignores the reader's font size.
    pub sizes: Sizes<&'static str>,
}

impl LoaderDefaults {
    pub const DEFAULT: Self = Self {
        variant: LoaderVariant::Oval,
        size: Size::Md,
        color: Color::Primary,
        sizes: Sizes::new("18px", "22px", "36px", "44px", "58px", "72px"),
    };

    pub fn size_sx(size: Size) -> Sx {
        sx().var(LOADER_SIZE, LOADER_SIZE_SCALE.value(size))
    }

    /// No `per_radius`: no shape has a corner a caller would set.
    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for LoaderDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        Size::ALL
            .into_iter()
            .map(|size| LOADER_SIZE_SCALE.declare(size, self.sizes.get(size)))
            .collect()
    }
}
