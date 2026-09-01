//! Four arrows, private to `Pagination`.
//!
//! `form/glyphs.rs` holds a chevron, a close, an upload and an eyedropper, all
//! `pub(super)` to the form module and none of them an arrow. These four share
//! nothing with those beyond being SVG, so promoting that module to
//! `components/common/` would move a file other units touch in exchange for no
//! actual reuse. They move up the day a second consumer wants them.

use dioxus::prelude::*;

/// One chevron, pointed by `rotate`. The four controls are the same glyph at
/// four angles, so there is one path rather than four that can drift apart.
#[component]
fn Chevron(rotate: &'static str) -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            style: "transform: rotate({rotate});",
            path { d: "M15 6l-6 6 6 6" }
        }
    }
}

#[component]
pub(super) fn PreviousIcon() -> Element {
    rsx! { Chevron { rotate: "0deg" } }
}

#[component]
pub(super) fn NextIcon() -> Element {
    rsx! { Chevron { rotate: "180deg" } }
}

/// A chevron against a bar, so first and last do not read as "one more step".
#[component]
fn ChevronToBar(rotate: &'static str) -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            style: "transform: rotate({rotate});",
            path { d: "M17 6l-6 6 6 6" }
            path { d: "M7 6v12" }
        }
    }
}

#[component]
pub(super) fn FirstIcon() -> Element {
    rsx! { ChevronToBar { rotate: "0deg" } }
}

#[component]
pub(super) fn LastIcon() -> Element {
    rsx! { ChevronToBar { rotate: "180deg" } }
}
