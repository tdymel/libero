use crate::components::{Demo, DemoValues, DocPage, Wrap, a11y};
use dioxus::prelude::*;
use libero::{
    components::{Code, Flex, Text},
    hooks::{use_is_mobile, use_media_query},
};

/// The two hooks in one component, as `Layout` renders them.
fn code(_: &DemoValues, _: &str) -> String {
    r#"let mobile = use_is_mobile();
let wide = use_media_query("(min-width: 1024px)");

rsx! {
    Flex { direction: "column", gap: "sm",
        Text { if mobile() { "Mobile: under 768px" } else { "Not mobile" } }
        Text { if wide() { "Wide: 1024px or more" } else { "Narrower than 1024px" } }
    }
}"#
    .to_string()
}

#[component]
fn Layout() -> Element {
    let mobile = use_is_mobile();
    let wide = use_media_query("(min-width: 1024px)");

    rsx! {
        Flex { direction: "column", gap: "sm",
            Text {
                if mobile() {
                    "Mobile: under 768px"
                } else {
                    "Not mobile"
                }
            }
            Text {
                if wide() {
                    "Wide: 1024px or more"
                } else {
                    "Narrower than 1024px"
                }
            }
        }
    }
}

#[component]
pub fn UseMediaQueryPage() -> Element {
    rsx! {
        DocPage {
            title: "Media query",
            source: "libero/src/hooks/media_query.rs",
            markdown: "/md/use_media_query.md",
            accessibility: a11y()
                .handles([
                    "The answer follows the viewport and the reader's own settings live, so a zoomed page that narrows past a breakpoint switches layout.",
                ])
                .must([
                    "Keep the content and every control reachable in both layouts: a breakpoint may rearrange a page, never drop what a reader needs.",
                    "Prefer CSS `@media` rules for pure styling. Use the hook when the component tree itself differs, such as a drawer in place of a sidebar.",
                ])
                .limits([
                    "The first render answers `false` and the real answer lands after mount, so a layout chosen by the hook flashes its default once. Native Blitz has no media queries and always answers `false`.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_media_query(query: &str) -> ReadSignal<bool>" }
                    " answers whether a CSS media query matches, and keeps answering as the viewport or the reader's settings change. "
                    Code { source: "use_is_mobile() -> ReadSignal<bool>" }
                    " is "
                    Code { source: "(max-width: 767px)" }
                    ": the 768px breakpoint."
                }
                Text {
                    "Both answer "
                    Code { source: "false" }
                    " on the first render and on a server render, then the real answer once the component is mounted. "
                    "Where the platform cannot answer, as on native Blitz, they stay "
                    Code { source: "false" }
                    ". Passing a different query re-subscribes."
                }
            },

            Demo {
                component: "use_media_query",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { Layout {} },
                wrap: Wrap(code),
            }
        }
    }
}
