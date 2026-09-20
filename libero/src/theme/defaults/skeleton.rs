use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Color, ColorShade, ColorValue, CssVar, Size, SizeCss};

// Per instance, from props. Unset means the content's size, the wrapper case.
pub const SKELETON_HEIGHT: CssVar = CssVar::new("--lsx-skeleton-height");
pub const SKELETON_WIDTH: CssVar = CssVar::new("--lsx-skeleton-width");

/// The active corner, resolved on the root from the `radius-*` token.
pub const SKELETON_RADIUS: CssVar = CssVar::new("--lsx-skeleton-radius");

pub const SKELETON_COLOR: CssVar = CssVar::new("--lsx-skeleton-color");
pub const SKELETON_DURATION: CssVar = CssVar::new("--lsx-skeleton-duration");

/// Runs on the grey layer only: on the root it would fade the children too.
pub const SKELETON_KEYFRAMES: &str =
    "@keyframes lsx-skeleton-pulse{0%,100%{opacity:0.4;}50%{opacity:1;}}";
pub const SKELETON_ANIMATION: &str = "lsx-skeleton-pulse";

/// Theme defaults for `Skeleton`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkeletonDefaults {
    pub radius: Size,
    /// The visible placeholder.
    pub color: ColorValue,
    /// One full pulse, any CSS `<time>`.
    pub duration: &'static str,
    /// Runs the pulse.
    pub animate: bool,
}

impl SkeletonDefaults {
    pub const DEFAULT: Self = Self {
        radius: Size::Sm,
        color: ColorValue::Shade(Color::Muted, ColorShade::S3),
        duration: "1500ms",
        animate: true,
    };

    pub fn radius_sx(radius: Size) -> Sx {
        sx().var(SKELETON_RADIUS, SizeCss::RADIUS.value(radius))
    }

    /// No `per_size`: the dimensions are the content's, or `height`/`width`.
    pub fn theme_vars() -> Sx {
        sx().per_radius(Self::radius_sx)
    }
}

impl ToCssDeclarations for SkeletonDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            SKELETON_COLOR.declare(self.color.value()),
            SKELETON_DURATION.declare(self.duration),
        ]
    }
}
