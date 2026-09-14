//! `Collapse`, for the motion checks.

use dioxus::prelude::*;
use libero::components::{Button, Collapse, Flex, Text};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/collapse", || rsx! { CollapsePage {} }),
    ("/collapse-kept", || rsx! { KeptPage {} }),
];

/// `keep_mounted` (the default): the closed panel's button stays in the DOM,
/// so only `visibility` keeps it out of the tab order and the tree.
#[component]
fn KeptPage() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Flex { id: "kept-frame", direction: "column", gap: "md", max_width: "320px",
            Button {
                id: "toggle-kept",
                variant: "outlined",
                aria_expanded: open(),
                aria_controls: "kept",
                onclick: move |_| open.toggle(),
                "Options"
            }
            Collapse { id: "kept", open: open(),
                Button { id: "inside", "Inside" }
            }
            Button { id: "after", variant: "outlined", "After" }
        }
    }
}

/// The motion fixture: a `Collapse` whose open and close both run a
/// transition, so a reduced-motion test has something to switch off.
/// `keep_mounted: false`, so closing also exercises the unmount that
/// `use_presence` ties to the exit.
#[component]
fn CollapsePage() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button {
                id: "toggle-details",
                variant: "outlined",
                onclick: move |_| open.toggle(),
                "Shipping details"
            }
            Collapse { id: "details", open: open(), keep_mounted: false,
                Text { id: "details-text", "Shipping is calculated at checkout." }
            }
        }
    }
}
