use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Flex, Text},
    sx::sx,
};

#[component]
pub fn AnchorPage() -> Element {
    rsx! {
        DocPage {
            title: "Anchor",
            lead: rsx! {
                Text {
                    "Text styled and sized like Text, rendered as a real link - router-aware "
                    "via to, falling back to a plain href when no router is mounted."
                }
            },
            DocSection {
                title: "Underline",
                Flex {
                    direction: "row",
                    gap: "lg",
                    Anchor { to: "https://dioxuslabs.com", underline: "always", "Always" }
                    Anchor { to: "https://dioxuslabs.com", underline: "hover", "Hover (default)" }
                    Anchor { to: "https://dioxuslabs.com", underline: "never", "Never" }
                }
            }
            DocSection {
                title: "Sizes",
                Flex {
                    direction: "row",
                    gap: "lg",
                    align: "baseline",
                    Anchor { to: "https://dioxuslabs.com", size: "xs", "Extra small" }
                    Anchor { to: "https://dioxuslabs.com", size: "md", "Medium" }
                    Anchor { to: "https://dioxuslabs.com", size: "xl", "Extra large" }
                }
            }
            DocSection {
                title: "Internal navigation",
                Text {
                    sx: sx().color("grey.6"),
                    "Uses the app's router directly - clicking this is a client-side navigation, not a full page reload.",
                }
                Anchor { to: crate::Route::GettingStarted {}, "Back to Getting Started" }
            }
        }
    }
}
