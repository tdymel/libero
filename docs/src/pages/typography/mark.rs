use crate::components::{Control, Demo, DemoValues, DocPage, DocSection};
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
            lead: rsx! {
                Text {
                    "Highlight "
                    Mark { "this chunk" }
                    " of the text. Renders a real "
                    Code { "<mark>" }
                    ", tinted with a light shade of the theme's "
                    Code { "warning" }
                    " color by default."
                }
            },
            DocSection {
                title: "Usage",
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
}
