use dioxus::prelude::*;
use libero::{
    components::{Flex, Image, Text, Title},
    sx::sx,
};

#[component]
pub fn ImagePage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "xxl",
            Flex {
                direction: "column",
                gap: "lg",
                Title { size: "xxl", "Image" }
                Text {
                    "An img with a fallback source on load error, optional rounded corners, "
                    "and an optional click-to-zoom overlay."
                }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Fit & radius" }
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
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Zoomable" }
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
