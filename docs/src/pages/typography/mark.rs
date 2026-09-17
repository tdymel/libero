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
            source: "libero/src/components/typography/mark.rs",
            markdown: "/md/mark.md",
            properties: vec![props("Mark", vec![
                prop("color", "ThemeAwareValue")
                    .default("warning, tinted")
                    .doc("A theme color name gets a light shade, and an explicit shade such as `error.4` stays as it is. Any CSS color works too."),
                prop("children", "Element").default("required").doc("The highlighted content."),
            ])],
            lead: rsx! {
                Text {
                    "Highlights "
                    Mark { "a chunk" }
                    " of text in a real "
                    Code { source: "<mark>" }
                    ", tinted with a light shade of the theme's "
                    Code { source: "warning" }
                    " color by default. The text color follows the tint, so it stays readable."
                }
            },
            Demo {
                component: "Mark",
                children_text: "this chunk",
                controls: vec![
                    Control::color("color")
                        .default(theme.mark.color.as_str()),
                ],
                render: move |values: DemoValues| rsx! {
                    Mark { color: values.str("color"), "this chunk" }
                },
            }
        }
    }
}
