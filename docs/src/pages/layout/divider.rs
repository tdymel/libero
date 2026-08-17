use dioxus::prelude::*;
use libero::{
    components::{Divider, Flex, Text, Title},
    sx::sx,
};

#[component]
pub fn DividerPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "xxl",
            Flex {
                direction: "column",
                gap: "lg",
                Title { size: "xxl", "Divider" }
                Text { "A horizontal or vertical rule, with an optional centered/positioned label." }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Horizontal" }
                Divider {}
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "With a label" }
                Divider { "OR" }
                Divider { label_position: "start", "Start" }
                Divider { label_position: "end", "End" }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Vertical" }
                Flex {
                    direction: "row",
                    gap: "md",
                    sx: sx().height("48px"),
                    Text { "Left" }
                    Divider { orientation: "vertical" }
                    Text { "Right" }
                }
            }
        }
    }
}
