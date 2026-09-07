use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, HtmlTag, Input, Text, Title};

#[component]
pub fn TitlePage() -> Element {
    rsx! {
        DocPage {
            title: "Title",
            source: "libero/src/components/typography/title.rs",
            markdown: "/md/title.md",
            properties: vec![props("Title", vec![
                prop("size", "Size")
                    .default("xxl")
                    .doc("Visual size, xs through xxl. Also picks the heading tag."),
                prop("component", "HtmlTag")
                    .default("follows size")
                    .doc("Overrides the heading tag, keeping a size's weight under a different level so h1 -> h2 -> h3 order survives."),
                prop("children", "Element").doc("The heading text."),
            ])],
            lead: rsx! {
                Text {
                    "A heading, h1 through h6 - "
                    Code { source: "component" }
                    " decouples the semantic tag from the visual size, for a11y heading order."
                }
            },
            Demo {
                    component: "Title",
                    children_text: "The quick brown fox",
                    controls: vec![
                        // `xl` is h2, the level a demo directly under the page's
                        // own `xxl` h1 answers to. `xxl` would open the page on a
                        // second h1. Both controls stay live: pinning `component`
                        // would kill the one control this page exists to show.
                        Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                            .default("xl"),
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
