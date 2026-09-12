//! `HoverCard`.

use dioxus::prelude::*;
use libero::components::{Button, ColorCode, ColorField, Flex, HoverCard, Text};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/hover-card", || rsx! { HoverCardPage {} }),
    (
        "/hover-card-color-field",
        || rsx! { ColorFieldInCardPage {} },
    ),
    ("/hover-card-disable", || rsx! { DisableWhileOpenPage {} }),
];

/// A card forced open, and a button that disables it (todo 449).
#[component]
fn DisableWhileOpenPage() -> Element {
    let mut disabled = use_signal(|| false);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "disable", onclick: move |_| disabled.set(true), "Disable" }
            HoverCard {
                aria_label: "Ada Lovelace",
                id: "card",
                open: true,
                disabled: disabled(),
                content: rsx! { Text { "Wrote the first algorithm meant for a machine." } },
                Button { id: "trigger", "Ada Lovelace" }
            }
        }
    }
}

/// A card with two controls, between two buttons so Tab has somewhere to come
/// from and to go to past the trigger. Its delays are values nothing else on
/// the page schedules, so the test can hold exactly those two timers.
#[component]
fn HoverCardPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            HoverCard {
                aria_label: "Ada Lovelace",
                open_delay: 707,
                close_delay: 808,
                content: rsx! {
                    Flex { direction: "column", gap: "xs",
                        Text { "Wrote the first algorithm meant for a machine." }
                        Button { id: "card-first", variant: "outlined", "Profile" }
                        Button { id: "card-last", variant: "outlined", "Follow" }
                    }
                },
                Button { id: "trigger", variant: "outlined", "Ada Lovelace" }
            }
            Button { id: "after", "After" }
        }
    }
}

/// A read-only `ColorField` in the card: its dropdown never shows, so Escape
/// in it is the card's (todo 446).
#[component]
fn ColorFieldInCardPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            HoverCard {
                aria_label: "Theme",
                id: "card",
                content: rsx! {
                    ColorField {
                        label: "Accent",
                        value: "#ff0000".parse::<ColorCode>().unwrap(),
                        readonly: true,
                    }
                },
                Button { id: "trigger", "Theme" }
            }
            Button { id: "after", "After" }
        }
    }
}
