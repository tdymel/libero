use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::components::Text;

#[component]
pub fn TextPage() -> Element {
    rsx! {
        DocPage {
            title: "Text",
            properties: vec![props("Text", vec![
                prop("size", "Size")
                    .default("md")
                    .doc("Visual size, xs through xxl."),
                prop("component", "HtmlTag")
                    .default("p")
                    .doc("Which element to render as."),
                prop("children", "Element").doc("The text content."),
            ])],
            lead: rsx! {
                Text { "Body copy - renders a p by default, sized via the theme's text scale." }
            },
            Demo {
                component: "Text",
                children_text: "The quick brown fox jumps over the lazy dog.",
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::toggle("component", ["p", "span", "div"]),
                ],
                render: move |values: DemoValues| rsx! {
                    Text {
                        size: values.str("size"),
                        component: values.str("component"),
                        "The quick brown fox jumps over the lazy dog."
                    }
                },
            }
        }
    }
}
