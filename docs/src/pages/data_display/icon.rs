use dioxus::prelude::*;
use libero::components::{Code, Flex, Icon, Text, Title};

fn checkmark() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M5 12l5 5L20 7" }
        }
    }
}

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
                        Icon { variant: "filled", color: "primary", {checkmark()} }
                        Text { size: "sm", "Filled" }
                    }
                    Flex {
                        direction: "column",
                        align: "center",
                        gap: "8px",
                        Icon { variant: "outlined", color: "primary", {checkmark()} }
                        Text { size: "sm", "Outlined" }
                    }
                    Flex {
                        direction: "column",
                        align: "center",
                        gap: "8px",
                        Icon { variant: "transparent", color: "primary", {checkmark()} }
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
                    Icon { size: "xs", {checkmark()} }
                    Icon { size: "sm", {checkmark()} }
                    Icon { size: "md", {checkmark()} }
                    Icon { size: "lg", {checkmark()} }
                    Icon { size: "xl", {checkmark()} }
                }
            }

            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Colors" }
                Flex {
                    direction: "row",
                    gap: "16px",
                    Icon { color: "primary", {checkmark()} }
                    Icon { color: "success", {checkmark()} }
                    Icon { color: "error", {checkmark()} }
                    Icon { color: "warning", {checkmark()} }
                }
            }
        }
    }
}
