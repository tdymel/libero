//! The focus ring's custom properties. Here, not beside `FocusRingDefaults`, because
//! `sx` writes the halo and sits below `theme` (todo 820).

use super::CssVar;

/// The dark stripe. A surface overrides it through `--lsx-focus-contrast`.
pub const FOCUS_RING_COLOR: CssVar = CssVar::new("--lsx-focus-ring-color");
/// The light halo on both sides of the stripe; a background overrides it (todo 630).
pub const FOCUS_RING_HALO: CssVar = CssVar::new("--lsx-focus-ring-halo");
/// The stripe's width.
pub const FOCUS_RING_WIDTH: CssVar = CssVar::new("--lsx-focus-ring-width");
/// The gap between the element and the stripe.
pub const FOCUS_RING_OFFSET: CssVar = CssVar::new("--lsx-focus-ring-offset");
/// The halo's width on each side of the stripe.
pub const FOCUS_RING_HALO_WIDTH: CssVar = CssVar::new("--lsx-focus-ring-halo-width");
/// The halo's `box-shadow` spread (offset + stripe + halo width), centring the stripe in it.
/// Derived, not a theme field.
pub const FOCUS_RING_HALO_SPREAD: CssVar = CssVar::new("--lsx-focus-ring-halo-spread");
