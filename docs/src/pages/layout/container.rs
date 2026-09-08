use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Code, Container, Text},
    sx::sx,
};

/// Printed as a `fixed` line, so the preview and the code block share it: a
/// container is invisible without a background to see its edges by.
// snippet: in Container { .. }
const SX: &str = r#"sx: sx().background("muted.1").padding_top("16px").padding_bottom("16px")"#;

#[component]
pub fn ContainerPage() -> Element {
    rsx! {
        DocPage {
            title: "Container",
            source: "libero/src/components/layout/container.rs",
            markdown: "/md/container.md",
            properties: vec![props("Container", vec![
                prop("component", "HtmlTag")
                    .default("div")
                    .doc("Which element to render as."),
                prop("size", "ThemeAwareValue")
                    .default("lg")
                    .doc("Max width, as a breakpoint (xs is 36rem, xxl 101rem) - the cap only bites once the surrounding area is wider than it."),
                prop("gutters", "ThemeAwareValue")
                    .default("md")
                    .doc("Horizontal padding, from the spacing scale."),
                prop("children", "Element").doc("The container's content."),
            ])],
            lead: rsx! {
                Text {
                    "Centers content and caps its width at a breakpoint - wraps your main "
                    "content, not the whole page shell. "
                    Code { source: "size" }
                    " names a breakpoint (xs is 36rem, xxl 101rem), so the cap only bites "
                    "once the surrounding area is wider than it; "
                    Code { source: "gutters" }
                    " is the horizontal padding, from the spacing scale."
                }
            },
            Demo {
                component: "Container",
                children_text: "Centered, width-capped content.",
                fixed: vec![SX.to_string()],
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("lg"),
                    Control::slider("gutters", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::toggle("component", ["div", "main", "section"]),
                ],
                render: move |values: DemoValues| rsx! {
                    Container {
                        size: values.str("size"),
                        gutters: values.str("gutters"),
                        component: values.str("component"),
                        sx: sx().background("muted.1").padding_top("16px").padding_bottom("16px"),
                        "Centered, width-capped content."
                    }
                },
            }
        }
    }
}
