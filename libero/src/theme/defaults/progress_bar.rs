use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{ColorCss, ColorShade, CssVar, Size, SizeCss, Sizes};

/// Track thickness, one declaration per step.
pub const PROGRESS_BAR_THICKNESS: SizeCss = SizeCss::new("--lsx-progress-bar-thickness-");

// The picked level, resolved on the root so the fill - which carries no
// `data-state` of its own - inherits it.
pub const PROGRESS_BAR_SIZE: CssVar = CssVar::new("--lsx-progress-bar-size");
pub const PROGRESS_BAR_RADIUS: CssVar = CssVar::new("--lsx-progress-bar-radius");

/// The unfilled groove. Themed once, not per instance.
pub const PROGRESS_BAR_TRACK: CssVar = CssVar::new("--lsx-progress-bar-track");
/// How long the fill takes to ease to a new value. Themed once; app-wide
/// chrome rather than a per-call knob, so there is no `transition` prop.
pub const PROGRESS_BAR_TRANSITION: CssVar = CssVar::new("--lsx-progress-bar-transition");

// Per instance, written from props by `progress_bar_variables`. Both are read
// with `value_or`, never `overridable` - nothing writes an `-override` twin
// here, which is the trap todo 49 records against `Carousel`.
pub const PROGRESS_BAR_COLOR: CssVar = CssVar::new("--lsx-progress-bar-color");
/// The drawn fraction as a percentage, e.g. `42%`.
pub const PROGRESS_BAR_FILL: CssVar = CssVar::new("--lsx-progress-bar-fill");

/// The indeterminate sweep.
///
/// A fixed-width fill translated across the track rather than a `width`
/// animation, so it composites on the GPU and never reflows. The fill is
/// [`INDETERMINATE_WIDTH`] of the track, and `translateX` percentages are
/// relative to the fill's own box: `-100%` parks its right edge on the track's
/// left edge, and `400%` carries its left edge one full track width past the
/// right - so the sweep enters and leaves completely.
pub const PROGRESS_BAR_KEYFRAMES: &str = concat!(
    "@keyframes lsx-progress-bar-indeterminate{",
    "from{transform:translateX(-100%);}",
    "to{transform:translateX(400%);}}"
);

/// The `data-state` that runs it, and the animation's name.
pub const PROGRESS_BAR_INDETERMINATE_STATE: &str = "indeterminate";
pub const PROGRESS_BAR_ANIMATION: &str = "lsx-progress-bar-indeterminate";

/// Width of the sweeping fill, as a share of the track.
pub const INDETERMINATE_WIDTH: &str = "25%";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProgressBarDefaults {
    pub size: Size,
    /// Corner of both track and fill.
    ///
    /// Note that a track is a full pill as soon as the radius reaches half its
    /// height, so most of the scale is visually inert on a thin bar: on the
    /// default `md` track (8px) only `xs` differs, and even the tallest track
    /// (`xxl`, 20px) separates only as far as `lg`. Shipped as specified
    /// anyway; the docs page says so.
    pub radius: Size,
    /// Grey step of the unfilled groove.
    pub track_shade: ColorShade,
    /// How long the fill eases to a new value.
    pub transition: &'static str,
    /// Track thickness per size step.
    pub sizes: Sizes<&'static str>,
}

impl ProgressBarDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        radius: Size::Xl,
        track_shade: ColorShade::S2,
        transition: "100ms",
        // Track heights. A bar is a full pill once the radius reaches half
        // of these, which is why most of the radius scale is inert here -
        // see `ProgressBarDefaults::radius`.
        sizes: Sizes::new("3px", "5px", "8px", "12px", "16px", "20px"),
    };

    pub fn size_sx(size: Size) -> Sx {
        sx().var(PROGRESS_BAR_SIZE, PROGRESS_BAR_THICKNESS.value(size))
    }

    pub fn radius_sx(radius: Size) -> Sx {
        sx().var(PROGRESS_BAR_RADIUS, SizeCss::RADIUS.value(radius))
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx).per_radius(Self::radius_sx)
    }
}

impl ToCssDeclarations for ProgressBarDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = Vec::new();
        for size in Size::ALL {
            declarations.push(PROGRESS_BAR_THICKNESS.declare(size, self.sizes.get(size)));
        }
        declarations.push(PROGRESS_BAR_TRACK.declare(ColorCss::GREY.value(self.track_shade)));
        declarations.push(PROGRESS_BAR_TRANSITION.declare(self.transition));
        declarations
    }
}
