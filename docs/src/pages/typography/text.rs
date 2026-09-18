use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, Text};

#[component]
pub fn TextPage() -> Element {
    rsx! {
        DocPage {
            title: "Text",
            source: "libero/src/components/typography/text.rs",
            markdown: "/md/text.md",
            properties: vec![props("Text", vec![
                prop("size", "Size")
                    .default("md")
                    .doc("Visual size, `xs` to `xxl`."),
                prop("component", "HtmlTag")
                    .default("p")
                    .doc("The element to render."),
                prop("children", "Element").default("required").doc("The text."),
            ])],
            lead: rsx! {
                Text {
                    "Body copy in a "
                    Code { source: "<p>" }
                    ", sized from the theme's text scale. "
                    Code { source: "component" }
                    " changes the element without changing the look. For headings, use "
                    Code { source: "Title" }
                    "."
                }
            },
            Demo {
                component: "Text",
                children_text: "The quick brown fox jumps over the lazy dog.",
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::toggle("component", ["p", "span", "div"])
                        .labels(["P", "Span", "Div"]),
                ],
                render: move |values: DemoValues| rsx! {
                    Text {
                        size: values.str("size"),
                        component: values.str("component"),
                        "The quick brown fox jumps over the lazy dog."
                    }
                },
            }
            DocSection { title: "Accessibility",
                Text {
                    "Use "
                    Code { source: "component: \"span\"" }
                    " for text inside a sentence. A large size is only styling, so it never "
                    "makes a heading."
                }
            }
        }
    }
}
