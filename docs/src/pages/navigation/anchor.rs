use crate::components::{Child, Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Anchor, AnchorPart, Code, Text};

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
                prop("parts", "Parts<AnchorPart>")
                    .doc("Styles for the inner parts in the Style API tab, under `sx`."),
            ])
            .parts("AnchorPart", vec![
                (AnchorPart::NewTab, "The new-tab icon after the text, with `target: \"_blank\"` only."),
            ])],
            accessibility: a11y()
                .handles([
                    "A `target: \"_blank\"` link draws a small external icon and reads a hidden \"(opens in a new tab)\". `new_tab_hint: false` drops both, for a link whose text already says it.",
                    "In a filled, tonal or gradient `Alert`, a colored `Header` and a `Mark`, a link takes the text color, so `underline: \"hover\"` draws its underline at rest there too.",
                ])
                .must([
                    "Make the link text say where the link goes: it is the accessible name.",
                    "Keep the underline on a link inside a paragraph: with `underline: \"never\"` it stands out by color alone.",
                ]),
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
                    Control::toggle("underline", ["hover", "always", "never"])
                        .labels(["Hover", "Always", "Never"]),
                    Control::sizes("size")
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
        }
    }
}
