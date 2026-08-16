use dioxus::prelude::*;
use libero::{
    components::{Container, Flex, Text, Title},
    sx::sx,
};

#[component]
pub fn ContainerPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { variant: "h1", "Container" }
                Text {
                    "Centers content and caps its width at a breakpoint - wraps your main "
                    "content, not the whole page shell."
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Sizes" }
                Container {
                    size: "sm",
                    sx: sx().background("grey.1").padding("16px"),
                    Text { "size: \"sm\" caps this container's width." }
                }
            }
        }
    }
}
