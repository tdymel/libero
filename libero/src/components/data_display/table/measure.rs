//! Boxes measured where a table cell fires no `onresize` (Blitz): the pin release's room.

use dioxus::prelude::*;

use super::pinning::PinSide;
use crate::hooks::{use_element, use_resize_fallback};

/// A box over a pinned header cell, reporting the cell's width as its `onresize` would.
#[component]
pub(super) fn PinProbe(
    header: String,
    side: PinSide,
    edge: bool,
    pin_width: Callback<(String, f64)>,
    pin_edge: Callback<(PinSide, f64)>,
) -> Element {
    let probe = use_element();
    use_resize_fallback(probe, move |event| {
        if let Ok(size) = event.get_border_box_size() {
            pin_width.call((header.clone(), size.width));
            if edge {
                pin_edge.call((side, size.width));
            }
        }
    });
    rsx! {
        div { "data-pin-box": true, aria_hidden: "true", onmounted: probe.mount() }
    }
}

/// A hidden box as wide as the leading columns, measured for the room left to pins.
#[component]
pub(super) fn LeadProbe(width: String, lead_px: Signal<f64>) -> Element {
    let probe = use_element();
    let measure = move |event: Event<ResizeData>| {
        let mut lead_px = lead_px;
        if let Ok(size) = event.get_border_box_size()
            && *lead_px.peek() != size.width
        {
            lead_px.set(size.width);
        }
    };
    use_resize_fallback(probe, measure);
    rsx! {
        div {
            aria_hidden: "true",
            style: "position:absolute;visibility:hidden;pointer-events:none;height:0;width:{width}",
            onmounted: probe.mount(),
            onresize: measure,
        }
    }
}
