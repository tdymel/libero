use dioxus::prelude::*;
use libero::components::{Anchor, Button, Divider, Flex, Text, Title};

use crate::{GITHUB, Route};

/// The way in once more, and the page's footer line.
#[component]
pub fn Closing() -> Element {
    rsx! {
        section { "aria-labelledby": "closing-title",
            Flex { direction: "column", gap: "lg", align: "center",
                Title { size: "xl", component: "h2", id: "closing-title", "Ready when you are" }
                Button { to: Route::GettingStarted {}, size: "lg", "Get started" }
            }
        }
        footer {
            Flex { direction: "column", gap: "lg",
                Divider {}
                Text { size: "sm", color: "muted.7",
                    "MIT or Apache-2.0. Source on "
                    Anchor { to: GITHUB, target: "_blank", "GitHub" }
                    ". Built with "
                    Anchor { to: "https://dioxuslabs.com", target: "_blank", "Dioxus" }
                    "."
                }
            }
        }
    }
}
