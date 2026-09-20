use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, Text};
use libero::theme::Gradient;

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
                prop("gradient", "Gradient")
                    .doc("Paints the glyphs with a gradient, `Gradient::default()` for the theme's. Keep it to large display text: the contrast of a literal CSS stop is yours to check. Solid in its first stop in forced colours and in native windows."),
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
                    Control::switch("gradient").code(|_, values| match values.str("gradient").as_str() {
                        "true" => vec!["gradient: Gradient::default()".to_string()],
                        _ => vec![],
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    Text {
                        size: values.str("size"),
                        component: values.str("component"),
                        gradient: (values.str("gradient") == "true").then(Gradient::default),
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
