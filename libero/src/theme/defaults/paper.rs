use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Color, ColorShade, ColorValue, CssVar, NamedColorCss, Size, SizeCss};

/// The library's one definition of a surface colour. Anything that paints
/// itself as a sheet of paper - `Paper`, `Dialog`, and every skeleton or
/// indicator that has to disappear against one - reads this rather than
/// spelling `white` again.
pub const PAPER_BACKGROUND: CssVar = CssVar::new("--lsx-paper-background");
pub const PAPER_BORDER_COLOR: CssVar = CssVar::new("--lsx-paper-border-color");
/// What reads against [`PAPER_BACKGROUND`]. Published as `--lsx-focus-contrast`
/// by [`PaperDefaults::theme_vars`], so a focus ring inside a surface contrasts
/// against it.
pub const PAPER_CONTRAST: CssVar = CssVar::new("--lsx-paper-contrast");
pub const PAPER_RADIUS: CssVar = CssVar::new("--lsx-paper-radius");
pub const PAPER_SHADOW: CssVar = CssVar::new("--lsx-paper-shadow");

/// What a surface looks like when nobody says otherwise.
///
/// `radius` and `shadow` are steps on the shared `SizeCss::RADIUS`/`SHADOW`
/// scales rather than a per-size scale of their own - a surface has one of
/// each, not six.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaperDefaults {
    pub radius: Size,
    pub shadow: Size,
    /// A CSS colour, not a palette token: it is the page's paper, and the
    /// palette has no shade that means "the surface itself" yet. Adding a
    /// dark arm later is a change to this value, not to any component.
    pub background: &'static str,
    /// What reads against `background`. It is a field rather than something
    /// derived because `background` is arbitrary CSS: `sx`'s `background()`
    /// publishes `--lsx-focus-contrast` only when it recognises a named
    /// colour, and a `var()` reference is opaque to it. Change one and change
    /// the other.
    pub contrast: ColorValue,
    pub border_color: ColorValue,
}

impl PaperDefaults {
    pub const DEFAULT: Self = Self {
        radius: Size::Md,
        shadow: Size::Sm,
        background: "#fff",
        contrast: ColorValue::Shade(Color::Ink, ColorShade::S1),
        border_color: ColorValue::Shade(Color::Grey, ColorShade::S3),
    };

    fn radius_sx(radius: Size) -> Sx {
        sx().border_radius(SizeCss::RADIUS.value(radius))
    }

    fn shadow_sx(shadow: Size) -> Sx {
        sx().box_shadow(SizeCss::SHADOW.value(shadow))
    }

    /// The surface colour, for a component that paints itself as one without
    /// being a `Paper` - a field frame, `Button`'s `Elevated` arm. Always take
    /// this, never `background(PAPER_BACKGROUND.value())` alone: the var is
    /// opaque to `sx`, so the `--lsx-focus-contrast` that `background("white")`
    /// used to publish for free has to be declared by hand, or every focus
    /// ring on the surface falls back to the primary shade.
    pub(crate) fn background_sx() -> Sx {
        sx().background(PAPER_BACKGROUND.value()).var(
            CssVar::Owned(NamedColorCss::FOCUS_CONTRAST.name().to_string()),
            PAPER_CONTRAST.value(),
        )
    }

    /// The base declarations name the themed vars, so a surface that sets no
    /// `radius`/`shadow` carries no `data-state` at all; the folds below only
    /// come into play once a caller names a step.
    pub fn theme_vars() -> Sx {
        Self::background_sx()
            .border_radius(PAPER_RADIUS.value())
            .box_shadow(PAPER_SHADOW.value())
            .per_radius(Self::radius_sx)
            .per_shadow(Self::shadow_sx)
    }
}

impl ToCssDeclarations for PaperDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            PAPER_RADIUS.declare(SizeCss::RADIUS.value(self.radius)),
            PAPER_SHADOW.declare(SizeCss::SHADOW.value(self.shadow)),
            PAPER_BACKGROUND.declare(self.background),
            PAPER_CONTRAST.declare(self.contrast.value()),
            PAPER_BORDER_COLOR.declare(self.border_color.value()),
        ]
    }
}
