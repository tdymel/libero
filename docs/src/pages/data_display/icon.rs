use dioxus::prelude::*;
use libero::components::{Code, Flex, Icon, Text, Title};

use crate::icons::CheckmarkIcon;

#[component]
pub fn IconPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { variant: "h1", "Icon" }
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
                gap: "8px",
                Title { variant: "h2", "Variants" }
                Flex {
                    direction: "row",
                    gap: "24px",
                    Flex {
                        direction: "column",
                        align: "center",
                        gap: "8px",
                        Icon { variant: "filled", color: "primary", CheckmarkIcon {} }
                        Text { size: "sm", "Filled" }
                    }
                    Flex {
                        direction: "column",
                        align: "center",
                        gap: "8px",
                        Icon { variant: "outlined", color: "primary", CheckmarkIcon {} }
                        Text { size: "sm", "Outlined" }
                    }
                    Flex {
                        direction: "column",
                        align: "center",
                        gap: "8px",
                        Icon { variant: "transparent", color: "primary", CheckmarkIcon {} }
                        Text { size: "sm", "Transparent" }
                    }
                }
            }

            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Sizes" }
                Flex {
                    direction: "row",
                    gap: "16px",
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
                gap: "8px",
                Title { variant: "h2", "Colors" }
                Flex {
                    direction: "row",
                    gap: "16px",
                    Icon { color: "primary", CheckmarkIcon {} }
                    Icon { color: "success", CheckmarkIcon {} }
                    Icon { color: "error", CheckmarkIcon {} }
                    Icon { color: "warning", CheckmarkIcon {} }
                }
            }
        }
    }
}
