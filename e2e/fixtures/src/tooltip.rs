//! `Tooltip`.

use dioxus::prelude::*;
use libero::components::{Button, Flex, Tooltip};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/tooltip", || rsx! { TooltipPage {} }),
    ("/tooltip-wrapped", || rsx! { TooltipWrappedPage {} }),
    ("/tooltip-edge", || rsx! { TooltipEdgePage {} }),
];

/// A `Tooltip` on a direct-child trigger, between two buttons so Tab has
/// somewhere to come from and to go to. `bottom`, so the test knows where the
/// gap it bridges lies.
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
