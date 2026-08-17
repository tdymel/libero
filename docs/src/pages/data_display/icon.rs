use dioxus::prelude::*;
use libero::components::{Code, Flex, Icon, Text, Title};

use crate::icons::CheckmarkIcon;

#[component]
pub fn IconPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "xxl",
            Flex {
                direction: "column",
                gap: "lg",
                Title { size: "xxl", "Icon" }
                Text {
                    "Wraps an svg child in a sized, colored badge. "
                    Code { "color" }
                    " sets the container's CSS color, which any child svg using "
                    Code { "currentColor" }
                    " for its fill/stroke then inherits."
                }
            }

            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Variants" }
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

            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Sizes" }
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

            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Colors" }
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
