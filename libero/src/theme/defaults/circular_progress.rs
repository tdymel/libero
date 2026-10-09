use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Color, ColorCss, ColorShade, CssVar, Size, SizeCss, Sizes};

/// The ring's outer edge per step.
pub const CIRCULAR_PROGRESS_SIZE_SCALE: SizeCss = SizeCss::new("--lsx-circular-progress-size-");

/// The active step's edge, republished unsuffixed by [`CircularProgressDefaults::size_sx`].
pub const CIRCULAR_PROGRESS_SIZE: CssVar = CssVar::new("--lsx-circular-progress-size");

/// The unfilled ring. Themed once, not per instance.
pub const CIRCULAR_PROGRESS_TRACK: CssVar = CssVar::new("--lsx-circular-progress-track");
/// How long the arc eases to a new value.
pub const CIRCULAR_PROGRESS_TRANSITION: CssVar = CssVar::new("--lsx-circular-progress-transition");

// Per instance, from props; read with `value_or`.
pub const CIRCULAR_PROGRESS_COLOR: CssVar = CssVar::new("--lsx-circular-progress-color");
/// The ring's width as a share of the edge, e.g. `0.1`.
pub const CIRCULAR_PROGRESS_THICKNESS: CssVar = CssVar::new("--lsx-circular-progress-thickness");

/// Theme defaults for `CircularProgress`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CircularProgressDefaults {
    /// The arc when a call site names no color.
    pub color: Color,
    pub size: Size,
    pub thickness: Size,
    /// Muted step of the unfilled ring.
    pub track_shade: ColorShade,
    /// How long the arc eases to a new value.
    pub transition: &'static str,
    /// rem throughout, so a ring holding a label grows with text zoom (WCAG 1.4.4); `Loader`
    /// stays px.
    pub sizes: Sizes<&'static str>,
    /// The ring's width per step, in percent of the edge, so it scales with `size`.
    pub thicknesses: Sizes<u8>,
}

impl CircularProgressDefaults {
    pub const DEFAULT: Self = Self {
        color: Color::Primary,
        size: Size::Md,
        thickness: Size::Md,
        track_shade: ColorShade::S2,
        transition: "100ms",
        sizes: Sizes::new(
            "1.125rem", "1.375rem", "2.25rem", "2.75rem", "3.625rem", "4.5rem",
        ),
        thicknesses: Sizes::new(6, 8, 10, 12, 14, 16),
    };

    pub fn size_sx(size: Size) -> Sx {
        sx().var(
            CIRCULAR_PROGRESS_SIZE,
            CIRCULAR_PROGRESS_SIZE_SCALE.value(size),
        )
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for CircularProgressDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations: Vec<_> = Size::ALL
            .into_iter()
            .map(|size| CIRCULAR_PROGRESS_SIZE_SCALE.declare(size, self.sizes.get(size)))
            .collect();
        declarations.push(CIRCULAR_PROGRESS_TRACK.declare(ColorCss::MUTED.value(self.track_shade)));
        declarations.push(CIRCULAR_PROGRESS_TRANSITION.declare(self.transition));
        declarations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A ring holding a label grows with text zoom (todo 2763): rem, 18px to 72px at 16px.
    #[test]
    fn the_edges_are_rem_and_match_the_old_px_at_a_16px_root() {
        let px: Vec<f32> = Size::ALL
            .into_iter()
            .map(|size| {
                let rem = CircularProgressDefaults::DEFAULT.sizes.get(size);
                rem.strip_suffix("rem").unwrap().parse::<f32>().unwrap() * 16.0
            })
            .collect();
        assert_eq!(px, [18.0, 22.0, 36.0, 44.0, 58.0, 72.0]);
    }
}
