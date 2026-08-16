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
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { variant: "h1", "Divider" }
                Text { "A horizontal or vertical rule, with an optional centered/positioned label." }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Horizontal" }
                Divider {}
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "With a label" }
                Divider { "OR" }
                Divider { label_position: "start", "Start" }
                Divider { label_position: "end", "End" }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Vertical" }
                Flex {
                    direction: "row",
                    gap: "12px",
                    sx: sx().height("48px"),
                    Text { "Left" }
                    Divider { vertical: true }
                    Text { "Right" }
                }
            }
        }
    }
}
