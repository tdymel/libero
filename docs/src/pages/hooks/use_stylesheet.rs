use crate::Route;
use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Box, Code, CodeBlock, Text},
    sx::sx,
    use_stylesheet,
};

const CALLOUT: &str = r#"#[component]
fn Callout(children: Element) -> Element {
    let class = use_stylesheet(&sx().background("primary.1").padding("md").border_radius("md"));

    rsx! {
        Box { class: class.unwrap_or_default(), {children} }
    }
}"#;

/// `CALLOUT`, rendered.
#[component]
fn Callout(children: Element) -> Element {
    let class = use_stylesheet(
        &sx()
            .background("primary.1")
            .padding("md")
            .border_radius("md"),
    );

    rsx! {
        Box { class: class.unwrap_or_default(), {children} }
    }
}

#[component]
pub fn UseStylesheetPage() -> Element {
    rsx! {
        DocPage {
            title: "use_stylesheet",
            source: "libero/src/hooks/stylesheet.rs",
            markdown: "/md/use_stylesheet.md",
            lead: rsx! {
                Text {
                    Code { source: "use_stylesheet(sheet) -> Option<String>" }
                    " registers a stylesheet of your own on the "
                    Code { source: "lsx-user-custom" }
                    " layer, above every other libero layer. Give it an "
                    Code { source: "sx" }
                    " and it returns the class to put on your element. "
                    Anchor { to: Route::StylingPage {}, "Styling" }
                    " explains the layers."
                }
            },

            DocSection {
                title: "Usage",
                Callout { Text { "Registered once, shared by every Callout." } }
                CodeBlock { source: CALLOUT, language: "rust" }
                Text {
                    "Raw CSS as a "
                    Code { source: "&str" }
                    " or a "
                    Code { source: "String" }
                    " works too. It has no single selector, so it returns "
                    Code { source: "None" }
                    ". Components that register the same sheet share one copy of it."
                }
            }
        }
    }
}
