use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
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
                    .doc("Visual size, `xs` to `xxl`. Also picks the tag unless `component` is set."),
                prop("component", "HtmlTag")
                    .default("follows size")
                    .doc("The heading tag. The size's look stays."),
                prop("children", "Element").default("required").doc("The heading text."),
            ])],
            accessibility: a11y()
                .handles(["`size` picks the heading tag, `xxl` as `h1` down to `xs` as `h6`, unless `component` is set."])
                .must([
                    "Keep one `h1` per page and skip no levels.",
                    "A `lg` heading in a section under the page's `h1` needs `component: \"h2\"`, or the document jumps from `h1` to `h3`.",
                ]),
            lead: rsx! {
                Text {
                    "A heading, "
                    Code { source: "h1" }
                    " to "
                    Code { source: "h6" }
                    ". "
                    Code { source: "size" }
                    " sets the look and the tag: "
                    Code { source: "xxl" }
                    " is "
                    Code { source: "h1" }
                    ", "
                    Code { source: "xl" }
                    " is "
                    Code { source: "h2" }
                    ", down to "
                    Code { source: "xs" }
                    " as "
                    Code { source: "h6" }
                    ". Set "
                    Code { source: "component" }
                    " when the look and the level disagree."
                }
            },
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
