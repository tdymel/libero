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

#[component]
pub(super) fn CloseIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M18 6L6 18" }
            path { d: "M6 6l12 12" }
        }
    }
}
