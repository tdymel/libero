use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Code, Flex, Icon, Text};

use crate::icons::CheckmarkIcon;

#[component]
pub fn IconPage() -> Element {
    rsx! {
        DocPage {
            title: "Icon",
            lead: rsx! {
                Text {
                    "Wraps an svg child in a sized, colored badge. "
                    Code { source: "color" }
                    " sets the container's CSS color, which any child svg using "
                    Code { source: "currentColor" }
                    " for its fill/stroke then inherits."
                }
            },
            DocSection {
                title: "Variants",
                Flex {
                    direction: "row",
                    gap: "xxl",
                    Flex {
                        direction: "column",
                        align: "center",
                        gap: "sm",
                        Icon { variant: "filled", color: "primary", CheckmarkIcon {} }
                        Text { size: "sm", "Filled" }
                    }
                    Flex {
                        direction: "column",
                        align: "center",
                        gap: "sm",
                        Icon { variant: "outlined", color: "primary", CheckmarkIcon {} }
                        Text { size: "sm", "Outlined" }
                    }
                    Flex {
                        direction: "column",
                        align: "center",
                        gap: "sm",
                        Icon { variant: "transparent", color: "primary", CheckmarkIcon {} }
                        Text { size: "sm", "Transparent" }
                    }
                }
            }

            DocSection {
                title: "Sizes",
                Flex {
                    direction: "row",
                    gap: "lg",
                    align: "center",
                    Icon { size: "xs", CheckmarkIcon {} }
                    Icon { size: "sm", CheckmarkIcon {} }
                    Icon { size: "md", CheckmarkIcon {} }
                    Icon { size: "lg", CheckmarkIcon {} }
                    Icon { size: "xl", CheckmarkIcon {} }
                }
            }

            DocSection {
                title: "Colors",
                Flex {
                    direction: "row",
                    gap: "lg",
                    Icon { color: "primary", CheckmarkIcon {} }
                    Icon { color: "success", CheckmarkIcon {} }
                    Icon { color: "error", CheckmarkIcon {} }
                    Icon { color: "warning", CheckmarkIcon {} }
                }
            }
        }
    }
}
