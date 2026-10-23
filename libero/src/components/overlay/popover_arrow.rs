use dioxus::prelude::*;

use crate::{
    CssLayer,
    hooks::{ArrowPoint, use_css},
    sx::{StaticSx, Sx, sx},
};

/// A square turned 45° whose half outside the box is the arrow; the box covers or
/// matches the rest. `pointer-events: none`, so the pointer stays on what lies under it.
pub(crate) fn arrow_sx() -> Sx {
    sx().transform("rotate(45deg)").pointer_events("none")
}

/// How far a `size` square's tip reaches past the box's edge: half its diagonal. Added
/// to the gap, so `gap` runs from the tip to the trigger.
pub(crate) fn arrow_reach(size: f64) -> f64 {
    size * std::f64::consts::FRAC_1_SQRT_2
}

/// A floating box's arrow: a child of a box that never scrolls, else a sibling
/// before it, since a scrolling box clips a child.
#[component]
pub(crate) fn PopoverArrow(
    sx: &'static StaticSx,
    /// From `PopoverHandle::arrow`, in the coordinates `sx`'s `position` reads; `None` draws nothing.
    point: Option<ArrowPoint>,
    /// The square's side, in px.
    size: f64,
    /// Declarations appended to the inline style, beating `sx`.
    #[props(default)]
    style: String,
) -> Element {
    let class = use_css(Some(sx), CssLayer::Framework);
    let Some(point) = point else {
        return rsx! {};
    };
    let half = size / 2.0;
    let style = format!(
        "left:{}px;top:{}px;width:{size}px;height:{size}px;{style}",
        point.x - half,
        point.y - half
    );
    rsx! {
        span { class, style, "data-slot": "arrow", aria_hidden: "true" }
    }
}
