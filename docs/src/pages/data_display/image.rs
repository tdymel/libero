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
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { variant: "h1", "Image" }
                Text {
                    "An img with a fallback source on load error, optional rounded corners, "
                    "and an optional click-to-zoom overlay."
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Fit & radius" }
                Flex {
                    direction: "row",
                    gap: "16px",
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
                gap: "8px",
                Title { variant: "h2", "Zoomable" }
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
