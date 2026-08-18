use dioxus::prelude::*;
use libero::{
    components::{Box, Center, Flex, Text, Title},
    sx::sx,
};

#[component]
pub fn CenterPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "xxl",
            Flex {
                direction: "column",
                gap: "lg",
                Title { size: "xxl", "Center" }
                Text { "Centers its child both horizontally and vertically." }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Block" }
                Center {
                    sx: sx().width("100%").height("120px").background("primary.1"),
                    Box { sx: sx().padding("8px 16px").background("primary"), "Centered" }
                }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Inline" }
                Text { "`inline` uses `inline-flex` instead of `flex`, so the box doesn't stretch to fill its parent's width." }
                Center {
                    inline: true,
                    sx: sx().background("primary.1"),
                    Box { sx: sx().padding("8px 16px").background("primary"), "Centered" }
                }
            }
        }
    }
}
