use crate::components::{
    Control, Demo, DemoValues, DocPage, UNSET, a11y, gradient_controls, gradient_value, prop, props,
};
use dioxus::prelude::*;
use libero::components::{Code, Input, Text};

fn no_gradient(values: &DemoValues) -> bool {
    values.str("gradient") != "true"
}

#[component]
pub fn TextPage() -> Element {
    let [gradient_to, gradient_deg] = gradient_controls(no_gradient);
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
                prop("color", "ThemeAwareValue")
                    .doc("The text color: a theme color name in its text shade, or any CSS color. Unset, the text inherits. Under a gradient, its first stop."),
                prop("gradient", "Gradient")
                    .doc("Paints the glyphs with a gradient from `color` to a second stop, as `(\"info\", 90)` or `Gradient::default().to(\"info\").deg(90)`; `Gradient::default()` is the theme's. Keep it to large display text: the contrast of a literal CSS stop is yours to check, and a debug build warns when a hex stop reads under 4.5:1 on the page background. Solid in its first stop in forced colours and in native windows."),
                prop("children", "Element").default("required").doc("The text."),
            ])],
            accessibility: a11y()
                .handles(["A large size is only styling, so it never makes a heading."])
                .must([
                    "Use `component: \"span\"` for text inside a sentence.",
                    "For a heading, use `Title`.",
                    "Keep `gradient` to large display text and check the contrast of a literal CSS stop. A debug build warns when a hex stop reads under 4.5:1 on the page background.",
                ]),
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
                    Control::color("color").with_unset(),
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
                    Text {
                        size: values.str("size"),
                        component: values.str("component"),
                        color: match values.str("color").as_str() {
                            UNSET => Input::None,
                            color => Input::from(color),
                        },
                        gradient: (values.str("gradient") == "true").then(|| gradient_value(&values)),
                        "The quick brown fox jumps over the lazy dog."
                    }
                },
            }
        }
    }
}
