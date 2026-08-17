use dioxus::prelude::*;
use libero::{
    components::{Anchor, Flex, Text, Title},
    sx::sx,
};

#[component]
pub fn AnchorPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "xxl",
            Flex {
                direction: "column",
                gap: "lg",
                Title { size: "xxl", "Anchor" }
                Text {
                    "Text styled and sized like Text, rendered as a real link - router-aware "
                    "via to, falling back to a plain href when no router is mounted."
                }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Underline" }
                Flex {
                    direction: "row",
                    gap: "lg",
                    Anchor { to: "https://dioxuslabs.com", underline: "always", "Always" }
                    Anchor { to: "https://dioxuslabs.com", underline: "hover", "Hover (default)" }
                    Anchor { to: "https://dioxuslabs.com", underline: "never", "Never" }
                }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Sizes" }
                Flex {
                    direction: "row",
                    gap: "lg",
                    align: "baseline",
                    Anchor { to: "https://dioxuslabs.com", size: "xs", "Extra small" }
                    Anchor { to: "https://dioxuslabs.com", size: "md", "Medium" }
                    Anchor { to: "https://dioxuslabs.com", size: "xl", "Extra large" }
                }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Internal navigation" }
                Text {
                    sx: sx().color("grey.6"),
                    "Uses the app's router directly - clicking this is a client-side navigation, not a full page reload.",
                }
                Anchor { to: crate::Route::GettingStarted {}, "Back to Getting Started" }
            }
        }
    }
}
