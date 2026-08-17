use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Flex, Text, Title},
    sx::sx,
};

#[component]
pub fn ButtonPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "xxl",
            Flex {
                direction: "column",
                gap: "lg",
                Title { size: "xxl", "Button" }
                Text {
                    "A clickable control, or a router-aware link when "
                    Code { "to" }
                    " is set."
                }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Variants" }
                Flex {
                    direction: "row",
                    gap: "md",
                    Button { variant: "filled", "Filled" }
                    Button { variant: "outlined", "Outlined" }
                    Button { variant: "text", "Text" }
                }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Sizes" }
                Flex {
                    direction: "row",
                    gap: "md",
                    align: "center",
                    Button { variant: "filled", size: "xs", "Extra small" }
                    Button { variant: "filled", size: "sm", "Small" }
                    Button { variant: "filled", size: "md", "Medium" }
                    Button { variant: "filled", size: "lg", "Large" }
                    Button { variant: "filled", size: "xl", "Extra large" }
                }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Colors" }
                Flex {
                    direction: "row",
                    gap: "md",
                    Button { variant: "filled", color: "primary", "Primary" }
                    Button { variant: "filled", color: "success", "Success" }
                    Button { variant: "filled", color: "error", "Error" }
                    Button { variant: "filled", color: "warning", "Warning" }
                }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Disabled" }
                Flex {
                    direction: "row",
                    gap: "md",
                    Button { variant: "filled", disabled: true, "Filled" }
                    Button { variant: "outlined", disabled: true, "Outlined" }
                }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "As a link" }
                Text {
                    sx: sx().color("grey.6"),
                    "Renders as a real anchor, or a router Link when to matches an internal route.",
                }
                Button {
                    variant: "outlined",
                    to: "https://dioxuslabs.com",
                    target: "_blank",
                    "Open Dioxus docs"
                }
            }
        }
    }
}
