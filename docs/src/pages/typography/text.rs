use crate::components::{Control, Demo, DemoValues, DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::Text;

#[component]
pub fn TextPage() -> Element {
    rsx! {
        DocPage {
            title: "Text",
            lead: rsx! {
                Text { "Body copy - renders a p by default, sized via the theme's text scale." }
            },
            DocSection {
                title: "Usage",
                Demo {
                    component: "Text",
                    children_text: "The quick brown fox jumps over the lazy dog.",
                    controls: vec![
                        Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                            .default("md"),
                        Control::toggle("component", ["p", "span", "div"]),
                    ],
                    render: move |values: DemoValues| rsx! {
                        Text {
                            size: values.str("size"),
                            component: values.str("component"),
                            "The quick brown fox jumps over the lazy dog."
                        }
                    },
                }
            }
        }
    }
}
