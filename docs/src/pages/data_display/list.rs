use dioxus::prelude::*;
use libero::components::{Code, Flex, List, ListItem, Text, Title};

#[component]
pub fn ListPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { variant: "h1", "List" }
                Text {
                    "Renders a "
                    Code { "ul" }
                    "/"
                    Code { "li" }
                    " pair with the browser's default list styling removed - nested lists "
                    "indent relative to their own content."
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Example" }
                List {
                    ListItem { "First item" }
                    ListItem { "Second item" }
                    ListItem {
                        "Third item, with a nested list"
                        List {
                            ListItem { "Nested one" }
                            ListItem { "Nested two" }
                        }
                    }
                }
            }
        }
    }
}
