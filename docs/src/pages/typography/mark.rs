use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
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
            accessibility: a11y()
                .handles([
                    "Each highlight is a real `<mark>`.",
                    "For a theme color, a shade or a hex, the text takes the tint's contrast color, so it stays readable.",
                    "A link inside is underlined in the text's color, unless its `underline` is `never`, and its focus ring clears 3:1 against the tint.",
                    "In forced colors the tint gives way to the system highlight colors, `Mark` and `MarkText`.",
                ])
                .must([
                    "With a CSS color name such as `gold`, the text keeps the page's color: check its contrast.",
                    "Say in the text why a highlight matters. Not every screen reader announces `<mark>`.",
                ]),
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
