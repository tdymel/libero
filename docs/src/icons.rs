use dioxus::prelude::*;

#[component]
pub fn ChevronIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M9 18l6-6-6-6" }
        }
    }
}

#[component]
pub fn CheckmarkIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M5 12l5 5L20 7" }
        }
    }
}

#[component]
pub fn FolderIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" }
        }
    }
}

#[component]
pub fn FileIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" }
            polyline { points: "14 2 14 8 20 8" }
        }
    }
}

#[component]
pub fn CodeIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "16 18 22 12 16 6" }
            polyline { points: "8 6 2 12 8 18" }
        }
    }
}

#[component]
pub fn AccessibilityIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "16", cy: "4", r: "1" }
            path { d: "m18 19 1-7-6 1" }
            path { d: "m5 8 3-3 5.5 3-2.36 3.5" }
            path { d: "M4.24 14.5a5 5 0 0 0 6.88 6" }
            path { d: "M13.76 17.5a5 5 0 0 0-6.88-6" }
        }
    }
}

#[component]
pub fn GitHubIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 16 16",
            fill: "currentColor",
            path {
                d: "M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82a7.4 7.4 0 0 1 2-.27c.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8Z"
            }
        }
    }
}

#[component]
pub fn MarkdownIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { x: "2", y: "5", width: "20", height: "14", rx: "2" }
            polyline { points: "6 15 6 9 9 12 12 9 12 15" }
            polyline { points: "16 9 16 15 19 15" }
        }
    }
}

#[component]
pub fn AlignLeftIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "3", y1: "6", x2: "21", y2: "6" }
            line { x1: "3", y1: "12", x2: "13", y2: "12" }
            line { x1: "3", y1: "18", x2: "17", y2: "18" }
        }
    }
}

#[component]
pub fn AlignCenterIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "3", y1: "6", x2: "21", y2: "6" }
            line { x1: "7", y1: "12", x2: "17", y2: "12" }
            line { x1: "5", y1: "18", x2: "19", y2: "18" }
        }
    }
}

#[component]
pub fn AlignRightIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "3", y1: "6", x2: "21", y2: "6" }
            line { x1: "11", y1: "12", x2: "21", y2: "12" }
            line { x1: "7", y1: "18", x2: "21", y2: "18" }
        }
    }
}

#[component]
pub fn DismissIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M18 6L6 18M6 6l12 12" }
        }
    }
}

#[component]
pub fn TruckIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { x: "1", y: "3", width: "15", height: "13" }
            polygon { points: "16 8 20 8 23 11 23 16 16 16 16 8" }
            circle { cx: "5.5", cy: "18.5", r: "2.5" }
            circle { cx: "18.5", cy: "18.5", r: "2.5" }
        }
    }
}

#[component]
pub fn CreditCardIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { x: "1", y: "4", width: "22", height: "16", rx: "2", ry: "2" }
            line { x1: "1", y1: "10", x2: "23", y2: "10" }
        }
    }
}

#[component]
pub fn ClipboardCheckIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { x: "8", y: "2", width: "8", height: "4", rx: "1" }
            path { d: "M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" }
            path { d: "m9 14 2 2 4-4" }
        }
    }
}

#[component]
pub fn SearchIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            circle { cx: "11", cy: "11", r: "7" }
            path { d: "M20 20l-4-4" }
        }
    }
}
