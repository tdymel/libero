use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Flex, Image, Text},
    sx::sx,
};

#[component]
pub fn ImagePage() -> Element {
    rsx! {
        DocPage {
            title: "Image",
            lead: rsx! {
                Text {
                    "An img with a fallback source on load error, optional rounded corners, "
                    "and an optional click-to-zoom overlay."
                }
            },
            DocSection {
                title: "Fit & radius",
                Flex {
                    direction: "row",
                    gap: "lg",
                    Image {
                        src: crate::LOGO,
                        alt: "Libero logo",
                        radius: "md",
                        sx: sx().width("96px").height("96px").background("grey.1"),
                    }
                    Image {
                        src: crate::LOGO,
                        alt: "Libero logo",
                        fit: "contain",
                        radius: "50%",
                        sx: sx().width("96px").height("96px").background("grey.1"),
                    }
                }
            }
            DocSection {
                title: "Zoomable",
                Text { sx: sx().color("grey.6"), "Click to open a zoomed overlay." }
                Image {
                    src: crate::LOGO,
                    alt: "Libero logo",
                    zoomable: true,
                    sx: sx().width("96px").height("96px").background("grey.1"),
                }
            }
        }
    }
}
