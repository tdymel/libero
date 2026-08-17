use dioxus::prelude::*;
use libero::{
    components::{Box, Flex, Text, Title},
    sx::sx,
};

#[component]
pub fn FlexPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { size: "xxl", "Flex" }
                Text { "A flexbox container - direction, gap, align, justify and wrap, all theme-aware." }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { size: "xl", "Row with gap" }
                Flex {
                    direction: "row",
                    gap: "12px",
                    Box { sx: sx().padding("8px 16px").background("primary.1"), "One" }
                    Box { sx: sx().padding("8px 16px").background("primary.1"), "Two" }
                    Box { sx: sx().padding("8px 16px").background("primary.1"), "Three" }
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { size: "xl", "Column" }
                Flex {
                    direction: "column",
                    gap: "8px",
                    Box { sx: sx().padding("8px 16px").background("primary.1"), "One" }
                    Box { sx: sx().padding("8px 16px").background("primary.1"), "Two" }
                }
            }
        }
    }
}
