use dioxus::prelude::*;
use libero::components::{Flex, Title};

/// One titled section of a [`DocPage`](super::DocPage).
#[component]
pub fn DocSection(title: String, children: Element) -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "sm",
            Title { size: "xl", "{title}" }
            {children}
        }
    }
}
