use crate::css::{CssDeclaration, ToCssDeclarations};

use crate::theme::{Color, ColorShade, ColorValue, CssVar};

/// The dark stripe. Overridden per surface by `--lsx-focus-contrast`, which a
/// surface publishes when it knows what reads against itself.
pub const FOCUS_RING_COLOR: CssVar = CssVar::new("--lsx-focus-ring-color");
/// The light halo drawn on both sides of the stripe. Overridden, beside
/// `--lsx-focus-contrast`, by the background that publishes it (todo 630).
pub const FOCUS_RING_HALO: CssVar = CssVar::new("--lsx-focus-ring-halo");
pub const FOCUS_RING_WIDTH: CssVar = CssVar::new("--lsx-focus-ring-width");
pub const FOCUS_RING_OFFSET: CssVar = CssVar::new("--lsx-focus-ring-offset");
pub const FOCUS_RING_HALO_WIDTH: CssVar = CssVar::new("--lsx-focus-ring-halo-width");
/// How far the halo's `box-shadow` spreads: out past the offset, the stripe
/// and the halo's own width, so the stripe lands in the middle of it.
/// Derived, not a field - it moves with whichever of the three a theme changes.
pub const FOCUS_RING_HALO_SPREAD: CssVar = CssVar::new("--lsx-focus-ring-halo-spread");
/// An element's own resting `box-shadow`, restated as a value the ring can
/// compose back in. The ring's halo *is* a `box-shadow`, and a
/// `:focus-visible` arm that sets the property drops whatever the resting
/// rule put there - an `Elevated` button would lose its elevation for as long
/// as it held focus. Set it wherever a focusable carries a resting shadow;
/// [`shadow_sx`](crate::components::common::shadow_sx) writes both at once.
pub const OWN_SHADOW: CssVar = CssVar::new("--lsx-own-shadow");

/// The library's one focus indicator: a dark stripe with a light halo on both
/// sides of it.
///
/// A single-tone ring cannot be guaranteed any contrast, because the surface
/// it is painted on belongs to the caller. `Splitter`'s divider is the proof -
/// its focusable is a 10px invisible hit overlay over a 1px bar, so the ring
/// is painted entirely over the caller's two panes. Deriving the ring from the
/// component's own colour fails the other way, giving a white ring over light
/// panes the moment a caller darkens the line.
///
/// Two tones carry their own contrast instead: whatever the surround is, the
/// stripe has the halo next to it, and the pair reads at the ratio between
/// `color` and `halo` rather than at the ratio against a surface nobody
/// controls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FocusRingDefaults {
    /// The stripe. Dark on purpose: the halo only rescues a *dark* surround,
    /// so the stripe is what has to read against a light one.
    pub color: ColorValue,
    /// What the stripe reads against.
    pub halo: ColorValue,
    /// The stripe's thickness, in px.
    pub width: u8,
    /// The gap between the element and the stripe, in px. The halo fills it.
    pub offset: u8,
    /// How far the halo reaches past the stripe, in px.
    pub halo_width: u8,
}

impl FocusRingDefaults {
    /// The two tones are the theme's own black and white rather than
    /// literals, and rather than a palette shade.
    ///
    /// `Color::Ink`/`Color::Surface` have no ramp, so both resolve to the one
    /// var each - `--lsx-ink` and `--lsx-surface`, which *are* `theme.ink`
    /// and `theme.surface` - and the shade below is ignored. That is what makes
    /// the pair follow the theme: a scheme that redefines the two ends of the
    /// page redefines the ring with them, and the tones stay adjacent because
    /// they invert together.
    pub const DEFAULT: Self = Self {
        color: ColorValue::Shade(Color::Ink, ColorShade::DEFAULT),
        halo: ColorValue::Shade(Color::Surface, ColorShade::DEFAULT),
        width: 2,
        offset: 2,
        halo_width: 2,
    };
}

impl ToCssDeclarations for FocusRingDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            FOCUS_RING_COLOR.declare(self.color.value()),
            FOCUS_RING_HALO.declare(self.halo.value()),
            FOCUS_RING_WIDTH.declare(format!("{}px", self.width)),
            FOCUS_RING_OFFSET.declare(format!("{}px", self.offset)),
            FOCUS_RING_HALO_WIDTH.declare(format!("{}px", self.halo_width)),
            FOCUS_RING_HALO_SPREAD.declare(format!(
                "calc({} + {} + {})",
                FOCUS_RING_OFFSET.value(),
                FOCUS_RING_WIDTH.value(),
                FOCUS_RING_HALO_WIDTH.value()
            )),
        ]
    }
}
