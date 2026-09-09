//! `HoverCard`.

use dioxus::prelude::*;
use libero::components::{Button, Flex, HoverCard, Text};

use crate::Routes;

pub const ROUTES: Routes = &[("/hover-card", || rsx! { HoverCardPage {} })];

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
