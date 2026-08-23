use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Code, Mark, Text},
    use_theme,
};

#[component]
pub fn MarkPage() -> Element {
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "Mark",
            properties: vec![props("Mark", vec![
                prop("color", "ThemeAwareValue")
                    .default("warning, tinted")
                    .doc("Any theme color or literal value; a bare theme color is tinted to a light shade."),
                prop("children", "Element").doc("The highlighted content."),
            ])],
            lead: rsx! {
                Text {
                    "Highlight "
                    Mark { "this chunk" }
                    " of the text. Renders a real "
                    Code { source: "<mark>" }
                    ", tinted with a light shade of the theme's "
                    Code { source: "warning" }
                    " color by default."
                }
            },
            Demo {
                component: "Mark",
                children_text: "this chunk",
                controls: vec![
                    Control::color("color", ["warning", "primary", "success", "error", "info"])
                        .default(theme.mark.color.as_str()),
                ],
                render: move |values: DemoValues| rsx! {
                    Mark { color: values.str("color"), "this chunk" }
                },
            }
        }
    }
}
