use crate::components::{Control, Demo, DemoValues, DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Text},
    sx::sx,
};

const HREF: &str = "https://dioxuslabs.com";

#[component]
pub fn AnchorPage() -> Element {
    rsx! {
        DocPage {
            title: "Anchor",
            lead: rsx! {
                Text {
                    "Text styled and sized like Text, rendered as a real link - router-aware "
                    "via to, falling back to a plain href when no router is mounted."
                }
            },
            DocSection {
                title: "Usage",
                Demo {
                    component: "Anchor",
                    children_text: "Read the Dioxus docs",
                    controls: vec![
                        // `to` is required, so it always prints.
                        Control::toggle("underline", ["hover", "always", "never"]).code(
                            |control, values| {
                                let mut set = vec![
                                    format!("to: {HREF:?}"),
                                    "target: \"_blank\"".to_string(),
                                ];
                                let value = values.str("underline");
                                if value != control.default {
                                    set.push(format!("underline: {value:?}"));
                                }
                                set
                            },
                        ),
                        Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                            .default("md"),
                    ],
                    render: move |values: DemoValues| rsx! {
                        Anchor {
                            to: HREF,
                            target: "_blank",
                            underline: values.str("underline"),
                            size: values.str("size"),
                            "Read the Dioxus docs"
                        }
                    },
                }
            }
            DocSection {
                title: "Internal navigation",
                Text {
                    sx: sx().color("grey.6"),
                    "Uses the app's router directly - clicking this is a client-side navigation, not a full page reload.",
                }
                Anchor { to: crate::Route::GettingStarted {}, "Back to Getting Started" }
            }
        }
    }
}
