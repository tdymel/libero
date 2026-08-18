use dioxus::prelude::*;
use libero::components::{Flex, Title};

/// A docs page: its heading, a lead paragraph, and its `DocSection`s.
///
/// `lead` is an `Element` rather than a `String` because most leads embed
/// `Code` spans in their prose.
#[component]
pub fn DocPage(title: String, lead: Element, children: Element) -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "xxl",
            Flex {
                direction: "column",
                gap: "lg",
                Title { size: "xxl", "{title}" }
                {lead}
            }
            {children}
        }
    }
}
