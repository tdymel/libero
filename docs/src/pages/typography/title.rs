use crate::components::{
    Control, Demo, DemoValues, DocPage, a11y, gradient_controls, gradient_value, prop, props,
};
use dioxus::prelude::*;
use libero::components::{Code, HtmlTag, Input, Text, Title};

fn no_gradient(values: &DemoValues) -> bool {
    values.str("gradient") != "true"
}

#[component]
pub fn TitlePage() -> Element {
    let [gradient_to, gradient_deg] = gradient_controls(no_gradient);
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
                prop("gradient", "Gradient")
                    .doc("Paints the glyphs with a gradient from the theme's first stop to a second, as `(\"info\", 90)` or `Gradient::default().to(\"info\").deg(90)`; `Gradient::default()` is the theme's. The contrast of a literal CSS stop is yours to check, and a debug build warns when a hex stop reads under 4.5:1 on the page background. Solid in its first stop in forced colors and in native windows."),
                prop("children", "Element").default("required").doc("The heading text."),
            ])],
            accessibility: a11y()
                .handles(["`size` picks the heading tag, `xxl` as `h1` down to `xs` as `h6`, unless `component` is set."])
                .must([
                    "Keep one `h1` per page and skip no levels.",
                    "A `lg` heading in a section under the page's `h1` needs `component: \"h2\"`, or the document jumps from `h1` to `h3`.",
                    "Check the contrast of a literal CSS stop in `gradient`. A debug build warns when a hex stop reads under 4.5:1 on the page background.",
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
                        Control::sizes("size")
                            .default("xxl"),
                        Control::slider(
                            "component",
                            ["auto", "h1", "h2", "h3", "h4", "h5", "h6"],
                        ),
                        // The theme's own second stop and angle print as `Gradient::default()`.
                        Control::switch("gradient").code(|_, values| {
                            let default = values.str("gradient_to") == "secondary"
                                && values.str("gradient_deg") == "45";
                            match values.str("gradient").as_str() {
                                "true" if default => vec!["gradient: Gradient::default()".to_string()],
                                _ => vec![],
                            }
                        }),
                        gradient_to,
                        gradient_deg,
                    ],
                    render: move |values: DemoValues| rsx! {
                        Title {
                            size: values.str("size"),
                            component: match values.str("component").as_str() {
                                // The default follows `size`, so leave the prop unset.
                                "auto" => Input::None,
                                tag => Input::Value(HtmlTag::from(tag)),
                            },
                            gradient: (values.str("gradient") == "true").then(|| gradient_value(&values)),
                            "The quick brown fox"
                        }
                },
            }
        }
    }
}
