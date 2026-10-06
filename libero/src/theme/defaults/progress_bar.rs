use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{ColorCss, ColorShade, CssVar, Size, SizeCss, Sizes};

/// Track thickness, one declaration per step.
pub const PROGRESS_BAR_THICKNESS: SizeCss = SizeCss::new("--lsx-progress-bar-thickness-");

// The picked level, resolved on the root so the fill inherits it.
pub const PROGRESS_BAR_SIZE: CssVar = CssVar::new("--lsx-progress-bar-size");
pub const PROGRESS_BAR_RADIUS: CssVar = CssVar::new("--lsx-progress-bar-radius");

/// The unfilled groove. Themed once, not per instance.
pub const PROGRESS_BAR_TRACK: CssVar = CssVar::new("--lsx-progress-bar-track");
/// How long the fill eases to a new value. App-wide, so there is no `transition` prop.
pub const PROGRESS_BAR_TRANSITION: CssVar = CssVar::new("--lsx-progress-bar-transition");

// Per instance, from props. Read with `value_or`, never `overridable`: no `-override`
// twin is written (the todo 49 trap).
pub const PROGRESS_BAR_COLOR: CssVar = CssVar::new("--lsx-progress-bar-color");
/// The drawn fraction as a percentage, e.g. `42%`.
pub const PROGRESS_BAR_FILL: CssVar = CssVar::new("--lsx-progress-bar-fill");

/// The indeterminate sweep: a [`INDETERMINATE_WIDTH`] fill translated (no reflow) from
/// `-100%` to `400%` of its own box, so it enters and leaves completely. The `-rtl` twin
/// mirrors it, as the fill starts at the right there (todo 2401).
pub const PROGRESS_BAR_KEYFRAMES: &str = concat!(
    "@keyframes lsx-progress-bar-indeterminate{",
    "from{transform:translateX(-100%);}",
    "to{transform:translateX(400%);}}",
    "@keyframes lsx-progress-bar-indeterminate-rtl{",
    "from{transform:translateX(100%);}",
    "to{transform:translateX(-400%);}}"
);

/// The `data-state` that runs it, and the animation's name.
pub const PROGRESS_BAR_INDETERMINATE_STATE: &str = "indeterminate";
pub const PROGRESS_BAR_ANIMATION: &str = "lsx-progress-bar-indeterminate";
pub(crate) const PROGRESS_BAR_ANIMATION_RTL: &str = "lsx-progress-bar-indeterminate-rtl";

/// Width of the sweeping fill, as a share of the track.
pub const INDETERMINATE_WIDTH: &str = "25%";

/// Theme defaults for `ProgressBar`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProgressBarDefaults {
    pub size: Size,
    /// Corner of track and fill. Past half the height it is a pill, so on the 8px
    /// `md` track only `xs` differs.
    pub radius: Size,
    /// Muted step of the unfilled groove.
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
        declarations.push(PROGRESS_BAR_TRACK.declare(ColorCss::MUTED.value(self.track_shade)));
        declarations.push(PROGRESS_BAR_TRANSITION.declare(self.transition));
        declarations
    }
}
