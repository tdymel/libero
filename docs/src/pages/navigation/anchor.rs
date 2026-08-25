use crate::components::{Child, Control, Demo, DemoValues, DocPage, prop, props};
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
            source: "libero/src/components/navigation/anchor.rs",
            markdown: "/md/anchor.md",
            properties: vec![props("Anchor", vec![
                prop("size", "Size").default("md").doc("Text size."),
                prop("to", "NavigationTarget")
                    .doc("A path/URL or a typed route (`Route::Foo {}`). With a router mounted and `target` unset or `\"_blank\"`, an internal target gets SPA navigation; otherwise a plain `href`."),
                prop("target", "String").doc("The anchor's `target` attribute."),
                prop("underline", "AnchorUnderline")
                    .default("hover")
                    .doc("When the underline draws: always, hover, or never."),
                prop("children", "Element").doc("The link's content."),
            ])],
            lead: rsx! {
                Text {
                    "Text styled and sized like Text, rendered as a real link - router-aware "
                    "via to, falling back to a plain href when no router is mounted. A typed "
                    "route navigates through the app's router, so clicking it is a client-side "
                    "navigation, not a full page reload."
                }
            },
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
