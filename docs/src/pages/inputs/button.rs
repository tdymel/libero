use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Flex, Text},
    sx::sx,
};

#[component]
pub fn ButtonPage() -> Element {
    rsx! {
        DocPage {
            title: "Button",
            lead: rsx! {
                Text {
                    "A clickable control, or a router-aware link when "
                    Code { "to" }
                    " is set."
                }
            },
            DocSection {
                title: "Variants",
                Flex {
                    direction: "row",
                    gap: "md",
                    Button { variant: "filled", "Filled" }
                    Button { variant: "outlined", "Outlined" }
                    Button { variant: "text", "Text" }
                }
            }
            DocSection {
                title: "Sizes",
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
            DocSection {
                title: "Colors",
                Flex {
                    direction: "row",
                    gap: "md",
                    Button { variant: "filled", color: "primary", "Primary" }
                    Button { variant: "filled", color: "success", "Success" }
                    Button { variant: "filled", color: "error", "Error" }
                    Button { variant: "filled", color: "warning", "Warning" }
                }
            }
            DocSection {
                title: "Disabled",
                Flex {
                    direction: "row",
                    gap: "md",
                    Button { variant: "filled", disabled: true, "Filled" }
                    Button { variant: "outlined", disabled: true, "Outlined" }
                }
            }
            DocSection {
                title: "Form type",
                Text {
                    sx: sx().color("grey.6"),
                    "A plain "
                    Code { "<button>" }
                    " submits an enclosing form. Ours defaults to "
                    Code { "type=\"button\"" }
                    " instead - set it yourself for a submit or reset button.",
                }
                Flex {
                    direction: "row",
                    gap: "md",
                    Button { variant: "filled", r#type: "submit", "Submit" }
                    Button { variant: "outlined", r#type: "reset", "Reset" }
                }
            }
            DocSection {
                title: "As a link",
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
