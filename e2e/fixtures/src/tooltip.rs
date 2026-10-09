//! `Tooltip`.

use dioxus::prelude::*;
use libero::components::{Button, Flex, Tooltip};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/tooltip", || rsx! { TooltipPage {} }),
    ("/tooltip-wrapped", || rsx! { TooltipWrappedPage {} }),
    ("/tooltip-edge", || rsx! { TooltipEdgePage {} }),
    ("/tooltip-start", || rsx! { TooltipStartPage {} }),
    ("/tooltip/quick", || rsx! { TooltipQuickPage {} }),
    ("/tooltip-long", || rsx! { TooltipLongPage {} }),
    ("/tooltip-toggle", || rsx! { TooltipTogglePage {} }),
    ("/tooltip-gap", || rsx! { TooltipGapPage {} }),
];

/// A label with no break opportunity, forced open, for reflow at 320px (todo 2384).
#[component]
fn TooltipLongPage() -> Element {
    rsx! {
        div { style: "padding-top: 80px;",
            Tooltip {
                label: rsx! { "Versandkostenberechnungsgrundlagenverordnungsentwurfsbearbeitungsstelle" },
                label_id: "save-tip",
                open: true,
                Button { id: "save", "aria-describedby": "save-tip", "Save" }
            }
        }
    }
}

/// 10 ms delays, a bottom bubble 200px off the left edge so it centres
/// without clamping, and `#away` to hover off to.
#[component]
fn TooltipQuickPage() -> Element {
    rsx! {
        Button { id: "before", "Before" }
        div { height: "40px" }
        div { margin_left: "200px",
            Tooltip {
                label: rsx! { "Saves the draft" },
                label_id: "save-tip",
                side: "bottom",
                open_delay: 10,
                close_delay: 10,
                Button { id: "save", "aria-describedby": "save-tip", "Save" }
            }
        }
        div { height: "200px" }
        p { id: "away", "Away" }
    }
}

/// A forced-open bottom bubble at a custom `gap: "20px"` (todo 2722).
#[component]
fn TooltipGapPage() -> Element {
    rsx! {
        div { margin_left: "200px", padding_top: "40px",
            Tooltip {
                label: rsx! { "Saves the draft" },
                label_id: "save-tip",
                side: "bottom",
                gap: "20px",
                open: true,
                Button { id: "save", "aria-describedby": "save-tip", "Save" }
            }
        }
    }
}

/// The trigger's press disables the tooltip, `#enable` turns it back on (todo 2446).
#[component]
fn TooltipTogglePage() -> Element {
    let mut disabled = use_signal(|| false);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Tooltip {
                label: rsx! { "Saves the draft" },
                label_id: "save-tip",
                side: "bottom",
                open_delay: 0,
                close_delay: 0,
                disabled: disabled(),
                Button { id: "save", onclick: move |_| disabled.set(true), "Save" }
            }
            Button { id: "enable", onclick: move |_| disabled.set(false), "Enable" }
        }
    }
}

/// A `Tooltip` on a direct-child trigger between two buttons. `bottom`, so the test knows
/// where the gap it bridges lies.
#[component]
fn TooltipPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            Tooltip { label: rsx! { "Saves the draft" }, label_id: "save-tip", side: "bottom",
                Button { id: "save", "aria-describedby": "save-tip", "Save" }
            }
            Button { id: "after", "After" }
        }
    }
}

/// The trigger one element too deep: focus inside the wrapper still opens it.
#[component]
fn TooltipWrappedPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            Tooltip { label: rsx! { "Saves the draft" }, label_id: "save-tip", side: "bottom",
                div {
                    Button { id: "save", "aria-describedby": "save-tip", "Save" }
                }
            }
            Button { id: "after", "After" }
        }
    }
}

/// A `top` tooltip at the top of the page, inside a box that clips: no room
/// above, so it flips below, and it escapes the clip.
#[component]
fn TooltipEdgePage() -> Element {
    rsx! {
        div { id: "clip", style: "overflow: hidden; height: 40px; width: 120px;",
            Tooltip { label: rsx! { "Saves the draft" }, label_id: "save-tip", side: "top",
                Button { id: "save", "aria-describedby": "save-tip", "Save" }
            }
        }
    }
}

/// A `start` tooltip in the middle of the page, room on both sides: the test
/// sets `dir` and reads which side it lands on (todo 711).
#[component]
fn TooltipStartPage() -> Element {
    rsx! {
        div { style: "display: flex; justify-content: center; padding-top: 120px;",
            Tooltip { label: rsx! { "Saves the draft" }, label_id: "save-tip", side: "start",
                Button { id: "save", "aria-describedby": "save-tip", "Save" }
            }
        }
    }
}
