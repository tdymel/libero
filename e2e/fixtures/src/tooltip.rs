//! `Tooltip`.

use dioxus::prelude::*;
use libero::components::{Button, Flex, Tooltip};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/tooltip", || rsx! { TooltipPage {} }),
    ("/tooltip-wrapped", || rsx! { TooltipWrappedPage {} }),
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

/// The trigger one element too deep: hover shows the bubble, Tab does not,
/// and a debug build says so in the console.
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
