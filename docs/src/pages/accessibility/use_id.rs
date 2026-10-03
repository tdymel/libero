use crate::components::{Demo, DemoValues, DocPage, Wrap, a11y};
use dioxus::prelude::*;
use libero::{
    components::{Box, Button, Code, Flex, Text},
    hooks::use_id,
};

/// The id is per instance, so the snippet is the component and two of it.
fn code(_: &DemoValues, _: &str) -> String {
    r#"#[component]
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

Flex { direction: "column", align: "flex-start", gap: "xs",
    Disclosure { title: "Shipping", Text { "Two to four working days." } }
    Disclosure { title: "Returns", Text { "Free within 30 days." } }
}"#
    .to_string()
}

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
            title: "Unique ID",
            source: "libero/src/hooks/id.rs",
            markdown: "/md/use_id.md",
            accessibility: a11y()
                .handles(["The id is unique within the app, so each instance's wiring stays its own. Each disclosure in the demo names its own panel, so a screen reader pairs every button with the right one."])
                .must(["Pass the id to `aria_controls`, `aria_labelledby`, `aria_describedby` or a label's `r#for`: an id is how they find their element."]),
            lead: rsx! {
                Text {
                    Code { source: "use_id() -> Signal<String>" }
                    " returns an id that is unique within the app and stays the same for "
                    "the component's lifetime. Use it for the aria wiring between one "
                    "instance's elements, where a fixed string would clash as soon as the "
                    "component renders twice."
                }
            },
            Demo {
                component: "use_id",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! {
                    Flex { direction: "column", align: "flex-start", gap: "xs",
                        Disclosure { title: "Shipping", Text { "Two to four working days." } }
                        Disclosure { title: "Returns", Text { "Free within 30 days." } }
                    }
                },
                wrap: Wrap(code),
            }
        }
    }
}
