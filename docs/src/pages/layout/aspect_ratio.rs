use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{AspectRatio, Flex, Text},
    sx::sx,
};

#[component]
pub fn AspectRatioPage() -> Element {
    rsx! {
        DocPage {
            title: "AspectRatio",
            lead: rsx! {
                Text { "Enforces a width-to-height ratio on its child, cropping it to fill the box." }
            },
            DocSection {
                title: "Ratios",
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
