//! The focus ring's custom properties. Here, not beside `FocusRingDefaults`, because
//! `sx` writes the halo and sits below `theme` (todo 820).

use super::CssVar;

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
