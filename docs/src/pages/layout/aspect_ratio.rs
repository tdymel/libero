use dioxus::prelude::*;
use libero::{
    components::{AspectRatio, Flex, Text, Title},
    sx::sx,
};

#[component]
pub fn AspectRatioPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "xxl",
            Flex {
                direction: "column",
                gap: "lg",
                Title { size: "xxl", "AspectRatio" }
                Text { "Enforces a width-to-height ratio on its child, cropping it to fill the box." }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Ratios" }
                Flex {
                    gap: "md",
                    AspectRatio {
                        ratio: 16.0 / 9.0,
                        sx: sx().width("240px"),
                        Flex {
                            sx: sx().background("primary").color("white"),
                            align: "center",
                            justify: "center",
                            "16 / 9"
                        }
                    }
                    AspectRatio {
                        ratio: 1.0,
                        sx: sx().width("160px"),
                        Flex {
                            sx: sx().background("secondary").color("white"),
                            align: "center",
                            justify: "center",
                            "1 / 1"
                        }
                    }
                }
            }
        }
    }
}
