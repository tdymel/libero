use crate::components::{Child, Control, Demo, DemoValues, DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Anchor, Text};

const HREF: &str = "https://dioxuslabs.com";
const EXTERNAL_LABEL: &str = "Read the Dioxus docs";
const INTERNAL_LABEL: &str = "Back to Getting Started";

/// A typed route reads differently from a URL, so the label follows `to`.
fn link_label(values: &DemoValues) -> String {
    match values.str("internal_route").as_str() {
        "true" => INTERNAL_LABEL.to_string(),
        _ => EXTERNAL_LABEL.to_string(),
    }
}

#[component]
pub fn AnchorPage() -> Element {
    rsx! {
        DocPage {
            title: "Anchor",
            lead: rsx! {
                Text {
                    "Text styled and sized like Text, rendered as a real link - router-aware "
                    "via to, falling back to a plain href when no router is mounted. A typed "
                    "route navigates through the app's router, so clicking it is a client-side "
                    "navigation, not a full page reload."
                }
            },
            DocSection {
                title: "Usage",
                Demo {
                    component: "Anchor",
                    children_text: "",
                    controls: vec![
                        // `to` is required, so it always prints - and it is
                        // what this switch varies.
                        Control::switch("internal_route").code(|_, values| {
                            match values.str("internal_route").as_str() {
                                "true" => vec!["to: Route::GettingStarted {}".to_string()],
                                _ => vec![
                                    format!("to: {HREF:?}"),
                                    "target: \"_blank\"".to_string(),
                                ],
                            }
                        }),
                        Control::toggle("underline", ["hover", "always", "never"]),
                        Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                            .default("md"),
                    ],
                    render: move |values: DemoValues| {
                        let underline = values.str("underline");
                        let size = values.str("size");
                        match values.str("internal_route").as_str() {
                            "true" => rsx! {
                                Anchor {
                                    to: crate::Route::GettingStarted {},
                                    underline,
                                    size,
                                    {INTERNAL_LABEL}
                                }
                            },
                            _ => rsx! {
                                Anchor {
                                    to: HREF,
                                    target: "_blank",
                                    underline,
                                    size,
                                    {EXTERNAL_LABEL}
                                }
                            },
                        }
                    },
                    child: Child(link_label),
                }
            }
        }
    }
}
