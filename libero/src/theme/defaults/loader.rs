use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::sx::{Sx, sx};
use crate::theme::{Color, CssVar, Size, SizeCss, Sizes};

/// The square edge, one declaration per step. The root resolves the active one
/// into [`LOADER_SIZE`]; every piece of geometry below is a fraction of that,
/// so a loader needs exactly one number to be fully specified.
pub const LOADER_SIZE_SCALE: SizeCss = SizeCss::new("--lsx-loader-size-");

/// The active step's edge, republished unsuffixed by [`LoaderDefaults::size_sx`],
/// which is the `Badge`/`ColorSwatch` pattern. The bar and dot children are
/// sized off it and inherit it, so they need to know nothing about which step
/// is on.
pub const LOADER_SIZE: CssVar = CssVar::new("--lsx-loader-size");

/// The ink. Per instance via `variables()`, never on `:root` - the `Mark`/`Image`
/// shape. Children inherit it from the root.
pub const LOADER_COLOR: CssVar = CssVar::new("--lsx-loader-color");

/// One `@keyframes` per variant, appended to the stylesheet beside
/// `RIPPLE_KEYFRAMES`.
///
/// Each one runs from the shape's *hidden* end to its *visible* end, which is
/// why the reduced-motion arm in the component cannot simply be
/// `animation: none` for every variant - see `Loader`'s own comment. Cancelling
/// an animation leaves the element on its static style, and for `bars` that is
/// the `from` frame: invisible.
pub const LOADER_KEYFRAMES: &str = concat!(
    "@keyframes lsx-loader-oval{from{transform:rotate(0deg);}to{transform:rotate(360deg);}}",
    "@keyframes lsx-loader-bars{from{transform:scale(0.6);opacity:0;}",
    "to{transform:scale(1);opacity:1;}}",
    "@keyframes lsx-loader-dots{from{transform:scale(1);opacity:1;}",
    "to{transform:scale(0.6);opacity:0.5;}}"
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoaderDefaults {
    pub variant: LoaderVariant,
    pub size: Size,
    pub color: Color,
    /// `xs`..`xl` are Mantine's own numbers; `xxl` continues the ramp, since
    /// our scale has a sixth step and theirs does not. px throughout: a
    /// spinner is a glyph at a fixed optical weight, not text - it does not
    /// follow a reader's font size the way `Badge`'s label does.
    pub sizes: Sizes<&'static str>,
}

impl LoaderDefaults {
    pub const DEFAULT: Self = Self {
        variant: LoaderVariant::Oval,
        size: Size::Md,
        color: Color::Primary,
        // `xs`..`xl` are Mantine's; `xxl` continues the ramp at the same
        // step, since our scale has a sixth level and theirs does not.
        sizes: Sizes::new("18px", "22px", "36px", "44px", "58px", "72px"),
    };

    pub fn size_sx(size: Size) -> Sx {
        sx().var(LOADER_SIZE, LOADER_SIZE_SCALE.value(size))
    }

    /// No `per_radius`: none of the three shapes has a corner a caller would
    /// set. A ring and a dot are round by construction and a bar's cap is part
    /// of the glyph, so a radius prop would have nothing to apply to -
    /// `Slider`'s pill-track lesson.
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
