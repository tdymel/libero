use crate::components::{Control, Demo, DemoValues, DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Code, HtmlTag, Input, Text, Title};

#[component]
pub fn TitlePage() -> Element {
    rsx! {
        DocPage {
            title: "Title",
            lead: rsx! {
                Text {
                    "A heading, h1 through h6 - "
                    Code { source: "component" }
                    " decouples the semantic tag from the visual size, for a11y heading order."
                }
            },
            DocSection {
                title: "Usage",
                Demo {
                    component: "Title",
                    children_text: "The quick brown fox",
                    controls: vec![
                        Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                            .default("xxl"),
                        Control::slider(
                            "component",
                            ["auto", "h1", "h2", "h3", "h4", "h5", "h6"],
                        ),
                    ],
                    render: move |values: DemoValues| rsx! {
                        Title {
                            size: values.str("size"),
                            component: match values.str("component").as_str() {
                                // The default follows `size`, so leave the prop unset.
                                "auto" => Input::None,
                                tag => Input::Value(HtmlTag::from(tag)),
                            },
                            "The quick brown fox"
                        }
                    },
                }
            }
        }
    }
}
