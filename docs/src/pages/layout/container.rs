use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
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
                    .doc("The element to render."),
                prop("size", "ThemeAwareValue")
                    .default("lg")
                    .doc("Max width, a breakpoint (`xs` is 36rem, `xxl` 101rem) or a CSS length."),
                prop("gutters", "ThemeAwareValue")
                    .default("md")
                    .doc("Horizontal padding, a spacing step or a CSS length."),
                prop("children", "Element").doc("The container's content."),
            ])],
            accessibility: a11y().must([
                "Use `component: \"main\"` or `\"section\"` when the region is a landmark.",
                "Name a `section` (`aria-label` or `aria-labelledby`) for it to count as a landmark.",
            ]),
            lead: rsx! {
                Text {
                    "Centers content and caps its width at a breakpoint. Wrap your main "
                    "content in it, not the whole page shell. The cap shows only once the "
                    "space around it is wider than "
                    Code { source: "size" }
                    "."
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
                    Control::toggle("component", ["div", "main", "section"])
                        .labels(["Div", "Main", "Section"]),
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
