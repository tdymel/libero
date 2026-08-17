use dioxus::prelude::*;
use libero::{
    components::{ActionIcon, Code, Flex, Text, Title},
    sx::sx,
};

use crate::icons::CheckmarkIcon;

#[component]
pub fn ActionIconPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { size: "xxl", "ActionIcon" }
                Text {
                    Code { "Icon" }
                    "'s sizing, color, and variant system, rendered as a real "
                    Code { "<button>" }
                    " with click handling and required a11y - for icon-only actions like a "
                    "copy, close, or delete button."
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { size: "xl", "Variants" }
                Flex {
                    direction: "row",
                    gap: "24px",
                    Flex {
                        direction: "column",
                        align: "center",
                        gap: "8px",
                        ActionIcon { variant: "filled", color: "primary", aria_label: "Confirm", CheckmarkIcon {} }
                        Text { size: "sm", "Filled" }
                    }
                    Flex {
                        direction: "column",
                        align: "center",
                        gap: "8px",
                        ActionIcon { variant: "outlined", color: "primary", aria_label: "Confirm", CheckmarkIcon {} }
                        Text { size: "sm", "Outlined" }
                    }
                    Flex {
                        direction: "column",
                        align: "center",
                        gap: "8px",
                        ActionIcon { variant: "transparent", color: "primary", aria_label: "Confirm", CheckmarkIcon {} }
                        Text { size: "sm", "Transparent" }
                    }
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { size: "xl", "No variant or color" }
                Text {
                    "With neither set, "
                    Code { "ActionIcon" }
                    " contributes no background/color of its own - it inherits "
                    "the surrounding text color instead of defaulting to a filled badge like "
                    Code { "Icon" }
                    " does, so it drops cleanly into a toolbar or a "
                    Code { "Code" }
                    " block header without fighting a default look. This is how "
                    Code { "Code" }
                    "'s own copy button is built."
                }
                ActionIcon { aria_label: "Confirm", CheckmarkIcon {} }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { size: "xl", "Sizes" }
                Flex {
                    direction: "row",
                    gap: "16px",
                    align: "center",
                    ActionIcon { color: "primary", size: "xs", aria_label: "Confirm", CheckmarkIcon {} }
                    ActionIcon { color: "primary", size: "sm", aria_label: "Confirm", CheckmarkIcon {} }
                    ActionIcon { color: "primary", size: "md", aria_label: "Confirm", CheckmarkIcon {} }
                    ActionIcon { color: "primary", size: "lg", aria_label: "Confirm", CheckmarkIcon {} }
                    ActionIcon { color: "primary", size: "xl", aria_label: "Confirm", CheckmarkIcon {} }
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { size: "xl", "Colors" }
                Flex {
                    direction: "row",
                    gap: "16px",
                    ActionIcon { color: "primary", aria_label: "Confirm", CheckmarkIcon {} }
                    ActionIcon { color: "success", aria_label: "Confirm", CheckmarkIcon {} }
                    ActionIcon { color: "error", aria_label: "Confirm", CheckmarkIcon {} }
                    ActionIcon { color: "warning", aria_label: "Confirm", CheckmarkIcon {} }
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { size: "xl", "Disabled" }
                ActionIcon {
                    variant: "filled",
                    color: "primary",
                    disabled: true,
                    aria_label: "Confirm",
                    CheckmarkIcon {}
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { size: "xl", "As a link" }
                Text {
                    sx: sx().color("grey.6"),
                    "Renders as a real anchor, or a router Link when to matches an internal route.",
                }
                ActionIcon {
                    variant: "outlined",
                    color: "primary",
                    to: "https://dioxuslabs.com",
                    target: "_blank",
                    aria_label: "Open Dioxus docs",
                    CheckmarkIcon {}
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { size: "xl", "Accessible name" }
                Text {
                    sx: sx().color("grey.6"),
                    "aria_label is required, not optional - an icon-only button has no visible "
                    "text for a screen reader to announce.",
                }
            }
        }
    }
}
