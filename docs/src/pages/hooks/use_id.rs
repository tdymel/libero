use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Box, Button, Code, CodeBlock, Flex, Text},
    hooks::use_id,
};

const DISCLOSURE: &str = r#"#[component]
fn Disclosure(title: String, children: Element) -> Element {
    let panel = use_id();
    let mut open = use_signal(|| false);

    rsx! {
        Button {
            variant: "standard",
            aria_expanded: open(),
            aria_controls: panel(),
            onclick: move |_| open.toggle(),
            "{title}"
        }
        Box { id: panel(), hidden: !open(), {children} }
    }
}

#[component]
fn Faq() -> Element {
    rsx! {
        Disclosure { title: "Shipping", Text { "Two to four working days." } }
        Disclosure { title: "Returns", Text { "Free within 30 days." } }
    }
}"#;

/// `DISCLOSURE`, rendered.
#[component]
fn Disclosure(title: String, children: Element) -> Element {
    let panel = use_id();
    let mut open = use_signal(|| false);

    rsx! {
        Button {
            variant: "standard",
            aria_expanded: open(),
            aria_controls: panel(),
            onclick: move |_| open.toggle(),
            "{title}"
        }
        Box { id: panel(), hidden: !open(), {children} }
    }
}

#[component]
pub fn UseIdPage() -> Element {
    rsx! {
        DocPage {
            title: "use_id",
            source: "libero/src/hooks/id.rs",
            markdown: "/md/use_id.md",
            lead: rsx! {
                Text {
                    Code { source: "use_id() -> Signal<String>" }
                    " returns an id that is unique in the process and stays the same for "
                    "the component's lifetime. Use it for the aria wiring between one "
                    "instance's elements, where a fixed string would clash as soon as the "
                    "component renders twice."
                }
            },

            DocSection {
                title: "Usage",
                Flex {
                    direction: "column",
                    align: "flex-start",
                    gap: "xs",
                    Disclosure { title: "Shipping", Text { "Two to four working days." } }
                    Disclosure { title: "Returns", Text { "Free within 30 days." } }
                }
                CodeBlock { source: DISCLOSURE, language: "rust" }
            }

            DocSection {
                title: "Accessibility",
                Text {
                    "An id is how "
                    Code { source: "aria_controls" }
                    ", "
                    Code { source: "aria_labelledby" }
                    ", "
                    Code { source: "aria_describedby" }
                    " and a label's "
                    Code { source: "r#for" }
                    " find their element. Each disclosure above names its own panel, so "
                    "a screen reader pairs every button with the right one."
                }
            }
        }
    }
}
