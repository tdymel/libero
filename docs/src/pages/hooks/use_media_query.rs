use crate::components::{Demo, DemoFile, DemoValues, DocPage, DocSection, a11y};
use dioxus::prelude::*;
use libero::components::{Code, Text};

mod demo;
use demo::Layout;

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
                .example("A settings page that swaps its sidebar for a drawer on `use_media_query(\"(max-width: 48em)\")`: at 200% zoom the page narrows past the breakpoint, the drawer takes over, and every link stays reachable from its burger.")
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
            },

            Demo {
                component: "Layout",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { Layout {} },
                file: DemoFile(include_str!("use_media_query/demo.rs")),
            }

            DocSection {
                title: "First render",
                Text {
                    "Both answer "
                    Code { source: "false" }
                    " on the first render and on a server render, then the real answer once the component is mounted. "
                    "Where the platform cannot answer, as on native Blitz, they stay "
                    Code { source: "false" }
                    ". Passing a different query re-subscribes."
                }
            }
        }
    }
}
