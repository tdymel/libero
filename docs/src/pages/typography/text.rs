use dioxus::prelude::*;
use libero::{
    components::{Flex, Text, Title},
    sx::sx,
};

#[component]
pub fn TextPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { size: "xxl", "Text" }
                Text { "Body copy - renders a p by default, sized via the theme's text scale." }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { size: "xl", "Sizes" }
                Flex {
                    direction: "column",
                    gap: "8px",
                    Text { size: "xs", "Extra small" }
                    Text { size: "sm", "Small" }
                    Text { size: "md", "Medium (default)" }
                    Text { size: "lg", "Large" }
                    Text { size: "xl", "Extra large" }
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { size: "xl", "As a span" }
                Text {
                    "Inline text with "
                    Text { component: "span", sx: sx().font_weight("700"), "bold inline text" }
                    " in the middle."
                }
            }
        }
    }
}
