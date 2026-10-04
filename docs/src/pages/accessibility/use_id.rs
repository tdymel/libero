use crate::components::{Demo, DemoFile, DemoValues, DocPage, Wrap, a11y};
use dioxus::prelude::*;
use libero::components::{Code, Flex, Text};

mod demo;
use demo::Disclosure;

/// The page's own source: the usage prints from its live demo.
const PAGE: DemoFile = DemoFile(include_str!("use_id.rs"));

#[component]
pub fn UseIdPage() -> Element {
    rsx! {
        DocPage {
            title: "Unique ID",
            source: "libero/src/hooks/id.rs",
            markdown: "/md/use_id.md",
            accessibility: a11y()
                .handles(["The id is unique within the app, so each instance's wiring stays its own. Each disclosure in the demo names its own panel, so a screen reader pairs every button with the right one."])
                .must(["Pass the id to `aria_controls`, `aria_labelledby`, `aria_describedby` or a label's `r#for`: an id is how they find their element."])
                .example("A disclosure in an FAQ: the button takes `aria_controls: id()` and the panel `id: id()`, so each question opens and names its own answer, however many disclosures the page has."),
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
                component: "Disclosure",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! {
                    // demo-code: usage start
                    Flex { direction: "column", align: "flex-start", gap: "xs",
                        Disclosure { title: "Shipping", Text { "Two to four working days." } }
                        Disclosure { title: "Returns", Text { "Free within 30 days." } }
                    }
                    // demo-code: usage end
                },
                wrap: Wrap(|_, _| PAGE.section("usage")),
                file: DemoFile(include_str!("use_id/demo.rs")),
            }
        }
    }
}
