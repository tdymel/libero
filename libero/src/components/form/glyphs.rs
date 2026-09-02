use dioxus::prelude::*;

/// libero ships no icon set, so the glyphs a field cannot do without live
/// here. Shared rather than per component: `Select`'s trigger and
/// `Autocomplete`'s clear button draw the same x.
#[component]
pub(super) fn ChevronIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M6 9l6 6 6-6" }
        }
    }
}

// The x moved to `components/common/icons.rs`, which is where every glyph
// lives from now on; `Alert`'s close button was the third module to need it.
// Re-exported rather than re-pointed at eight call sites, so `glyphs::`
// stays the one name a field imports.
pub(super) use crate::components::common::CloseIcon;

/// The dropzone's prompt. A tray with an arrow going into it, which is the
/// shape every upload control has settled on.
#[component]
pub(super) fn UploadIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M12 16V4" }
            path { d: "M8 8l4-4 4 4" }
            path { d: "M4 16v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2" }
        }
    }
}

/// `ColorField`'s eyedropper button: a pipette, tip at the bottom left.
#[component]
pub(super) fn EyeDropperIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M11 7l6 6" }
            path { d: "M4 16l11.7 -11.7a1 1 0 0 1 1.4 0l2.6 2.6a1 1 0 0 1 0 1.4l-11.7 11.7h-4v-4z" }
        }
    }
}
