//! `Button`, as a button and as a router link.

use dioxus::prelude::*;
use libero::components::{Button, Flex, Text};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/button", || rsx! { ButtonPage {} }),
    ("/button/landing", || rsx! { ButtonLanding {} }),
];

/// A plain button and a busy one, each counting its activations, and a
/// link-mode button whose route is one only this page leads to.
#[component]
fn ButtonPage() -> Element {
    let mut plain = use_signal(|| 0u32);
    let mut busy = use_signal(|| 0u32);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "plain", onclick: move |_| plain += 1, "Plain" }
            Button { id: "busy", loading: true, onclick: move |_| busy += 1, "Busy" }
            Text { id: "presses", "data-plain": "{plain}", "data-busy": "{busy}",
                "Plain {plain}, busy {busy}"
            }
            Button { id: "to-landing", to: "/button/landing", "Go to landing" }
            // Todo 452: one per label rule, for the hover contrast test.
            Button { id: "outlined", variant: "outlined", "Outlined" }
            Button { id: "standard-error", variant: "standard", color: "error", "Standard" }
            Button { id: "elevated", variant: "elevated", color: "secondary", "Elevated" }
            Button { id: "filled-muted", color: "muted", "Filled" }
            Button { id: "disabled-link", to: "/button/landing", disabled: true, "Disabled link" }
        }
    }
}

#[component]
fn ButtonLanding() -> Element {
    rsx! { Text { id: "landing", "Landed" } }
}
