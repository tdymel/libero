use crate::components::{Child, Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::components::{Anchor, Code, Text};

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
                    .default("required")
                    .doc("A path, a URL or a typed route (`Route::Foo {}`). With a router mounted and `target` unset or `\"_blank\"`, an internal target navigates without a page reload. A `javascript:` URL runs script on click, so check the scheme of any URL from user data."),
                prop("target", "String").doc("The link's `target` attribute. `\"_blank\"` adds a small external icon and a hidden \"(opens in a new tab)\"."),
                prop("new_tab_hint", "bool")
                    .default("true")
                    .doc("`false` drops the icon and the hidden text a `\"_blank\"` target adds."),
                prop("underline", "AnchorUnderline")
                    .default("hover")
                    .doc("When the underline draws, `always`, `hover` or `never`."),
                prop("children", "Element").default("required").doc("The link's content."),
            ])],
            lead: rsx! {
                Text {
                    "A link styled and sized like "
                    Code { source: "Text" }
                    ". A typed route in "
                    Code { source: "to" }
                    " navigates through the app's router without a page reload. Without a "
                    "router it renders a plain "
                    Code { source: "href" }
                    "."
                }
            },
            // snippet: item #[derive(Clone, PartialEq, Routable)] enum Route { #[route("/")] GettingStarted {} }
            // snippet: item #[component] fn GettingStarted() -> Element { rsx! {} }
            Demo {
                component: "Anchor",
                children_text: "",
                controls: vec![
                    // `to` is required, so it always prints. This switch varies it.
                    Control::switch("internal_route").code(|_, values| {
                        match values.str("internal_route").as_str() {
                            "true" => vec!["to: Route::GettingStarted {}".to_string()],
                            _ => vec![
                                format!("to: {HREF:?}"),
                                "target: \"_blank\"".to_string(),
                            ],
                        }
                    }),
                    Control::switch("new_tab_hint").default("true").code(|_, values| {
                        match (values.str("internal_route").as_str(), values.str("new_tab_hint").as_str()) {
                            ("true", _) | (_, "true") => vec![],
                            _ => vec!["new_tab_hint: false".to_string()],
                        }
                    }),
                    Control::toggle("underline", ["hover", "always", "never"]),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                ],
                render: move |values: DemoValues| {
                    let underline = values.str("underline");
                    let size = values.str("size");
                    let new_tab_hint = values.str("new_tab_hint") == "true";
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
                                new_tab_hint,
                                underline,
                                size,
                                {EXTERNAL_LABEL}
                            }
                        },
                    }
                },
                child: Child(link_label),
            }
            DocSection { title: "Accessibility",
                Text {
                    "The link text is the accessible name, so make it say where the link goes. A "
                    Code { source: "target: \"_blank\"" }
                    " link also reads \"(opens in a new tab)\". With "
                    Code { source: "underline: \"never\"" }
                    ", a link inside a paragraph stands out by color alone."
                }
            }
        }
    }
}
