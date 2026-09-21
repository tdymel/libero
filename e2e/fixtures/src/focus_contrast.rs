//! The focus ring's `--lsx-focus-contrast` fallback, and the one a hex publishes.

use dioxus::prelude::*;
use libero::{
    components::{Alert, Anchor, Box, Button, Flex, Paper},
    sx::sx,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/focus-contrast", || rsx! { FocusContrastPage {} }),
    ("/focus-contrast/hex", || rsx! { HexPage {} }),
    ("/focus-contrast/fills", || rsx! { FillsPage {} }),
];

/// Todo 53: a `--lsx-focus-contrast` naming an undeclared referent is invalid at computed-value
/// time, not unset, so whether the ring falls back to primary can only be measured.
#[component]
fn FocusContrastPage() -> Element {
    rsx! {
        document::Style {
            ":root {{ --lsx-focus-contrast: var(--nothing-declares-this); }}"
        }
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "ring-probe", "Probe" }
        }
    }
}

/// Todo 630: dark fills in light, where a white stripe met the page's white halo.
#[component]
fn FillsPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Paper { sx: sx().background("primary.9").padding("md"),
                Anchor { id: "paper-link", to: "#", sx: sx().color("#FFFFFF"), "a link on a dark paper" }
            }
            Alert { id: "filled-alert", color: "info", variant: "filled", title: "Saved",
                onclose: |_| {},
                "The message."
            }
        }
    }
}

/// Todo 604: a link on a caller's hex background, whose ring is read off the hex.
#[component]
fn HexPage() -> Element {
    rsx! {
        Box { sx: sx().background("#1e3a8a").padding("md").max_width("320px"),
            Anchor { id: "hex-link", to: "#", sx: sx().color("#FFFFFF"), "a link on navy" }
        }
    }
}
