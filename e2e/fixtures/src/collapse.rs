//! `Collapse`, for the motion checks.

use dioxus::prelude::*;
use libero::components::{Button, Collapse, Flex, Text};

use crate::Routes;

pub const ROUTES: Routes = &[("/collapse", || rsx! { CollapsePage {} })];

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
