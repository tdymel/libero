use crate::Route;
use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap, a11y};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Box, Code, Text},
    sx::sx,
    use_stylesheet,
};

/// The hook call and the box that wears its class, as `Callout` renders them.
fn code(values: &DemoValues, _: &str) -> String {
    format!(
        "let class = use_stylesheet(\n    \
             &sx().background({:?}).padding({:?}).border_radius({:?}),\n\
         );\n\n\
         rsx! {{\n    \
             Box {{ class: class.unwrap_or_default(),\n        \
                 Text {{ \"Registered once, shared by every Callout.\" }}\n    \
             }}\n\
         }}",
        values.str("background"),
        values.str("padding"),
        values.str("border_radius"),
    )
}

#[component]
fn Callout(background: String, padding: String, border_radius: String) -> Element {
    let class = use_stylesheet(
        &sx()
            .background(background)
            .padding(padding)
            .border_radius(border_radius),
    );

    rsx! {
        Box { class: class.unwrap_or_default(),
            Text { "Registered once, shared by every Callout." }
        }
    }
}

#[component]
pub fn UseStylesheetPage() -> Element {
    rsx! {
        DocPage {
            title: "Stylesheet",
            source: "libero/src/hooks/stylesheet.rs",
            markdown: "/md/use_stylesheet.md",
            accessibility: a11y()
                .handles([
                    "Nothing on screen: it registers CSS and returns a class.",
                ])
                .must([
                    "Keep a visible focus indicator: a rule here outranks libero's own focus ring, so `outline: none` on a control removes it (WCAG 2.4.7).",
                    "Check the contrast of a literal colour you set: 4.5:1 for text, 3:1 for borders and icons (WCAG 1.4.3, 1.4.11). A theme colour such as `primary.1` follows the theme set.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_stylesheet(sheet) -> Option<String>" }
                    " registers a stylesheet of your own on the "
                    Code { source: "lsx-user-custom" }
                    " layer, above every other libero layer. Give it an "
                    Code { source: "sx" }
                    " and it returns the class to put on your element. "
                    Anchor { to: Route::StylingPage {}, "Styling" }
                    " explains the layers."
                }
            },
            Demo {
                component: "use_stylesheet",
                children_text: "",
                controls: vec![
                    Control::toggle("background", ["primary.1", "success.1", "warning.1"])
                        .labels(["Primary 1", "Success 1", "Warning 1"])
                        .default("primary.1"),
                    Control::sizes("padding").default("md"),
                    Control::sizes("border_radius").default("md"),
                ],
                render: move |values: DemoValues| rsx! {
                    Callout {
                        background: values.str("background"),
                        padding: values.str("padding"),
                        border_radius: values.str("border_radius"),
                    }
                },
                wrap: Wrap(code),
            }

            DocSection {
                title: "Raw CSS",
                Text {
                    "Raw CSS as a "
                    Code { source: "&str" }
                    " or a "
                    Code { source: "String" }
                    " works too. It has no single selector, so it returns "
                    Code { source: "None" }
                    ". Components that register the same sheet share one copy of it."
                }
            }
        }
    }
}
