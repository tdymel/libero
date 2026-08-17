use dioxus::prelude::*;
use libero::{
    components::{Box, Divider, Flex, Text, Title},
    sx::sx,
};

#[component]
pub fn FlexPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "xxl",
            Flex {
                direction: "column",
                gap: "lg",
                Title { size: "xxl", "Flex" }
                Text { "A flexbox container - direction, gap, align, justify and wrap, all theme-aware." }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Row with gap" }
                Flex {
                    direction: "row",
                    gap: "md",
                    Box { sx: sx().padding("8px 16px").background("primary.1"), "One" }
                    Box { sx: sx().padding("8px 16px").background("primary.1"), "Two" }
                    Box { sx: sx().padding("8px 16px").background("primary.1"), "Three" }
                }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Column" }
                Flex {
                    direction: "column",
                    gap: "sm",
                    Box { sx: sx().padding("8px 16px").background("primary.1"), "One" }
                    Box { sx: sx().padding("8px 16px").background("primary.1"), "Two" }
                }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Divider" }
                Text { "Rendered between each child - not before the first or after the last." }
                Flex {
                    direction: "column",
                    gap: "sm",
                    divider: rsx! { Divider {} },
                    Text { "First section" }
                    Text { "Second section" }
                    Text { "Third section" }
                }
            }
        }
    }
}
