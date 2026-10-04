use crate::components::{Demo, DemoFile, DemoValues, DocPage, DocSection, a11y};
use dioxus::prelude::*;
use libero::components::{Code, Text};

mod demo;
use demo::LiveSearch;

#[component]
pub fn UseDebouncePage() -> Element {
    rsx! {
        DocPage {
            title: "Debounce and throttle",
            source: "libero/src/hooks/debounce.rs",
            markdown: "/md/use_debounce.md",
            accessibility: a11y()
                .handles([
                    "A pending update is dropped when the component unmounts.",
                ])
                .must([
                    "Announce results that arrive late from a debounced search in a live region (`role=\"status\"`); a screen reader user gets no other sign that the list changed.",
                    "Keep the field itself bound to the live signal, as the demo does: delaying the text a person is typing makes the field lag behind their keys.",
                ])
                .example("A product search that fetches on `use_debounced_value` of the query, 300 ms after the last key: the field follows every key, and a `role=\"status\"` line says \"12 results\" once the late results arrive."),
            lead: rsx! {
                Text {
                    Code { source: "use_debounced_value(value: ReadSignal<T>, ms: u64) -> ReadSignal<T>" }
                    " follows a signal once it has stopped changing for "
                    Code { source: "ms" }
                    ": the right copy for a search request that should not fire per key. "
                    Code { source: "use_throttled_value" }
                    " takes the same arguments but follows at once, then at most once per "
                    Code { source: "ms" }
                    ", ending on the last value: the right copy for a pointer position or a scroll offset."
                }
            },

            Demo {
                component: "LiveSearch",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { LiveSearch {} },
                file: DemoFile(include_str!("use_debounce/demo.rs")),
            }

            DocSection {
                title: "Callbacks",
                Text {
                    Code { source: "use_debounced_callback(callback, ms) -> Callback<A>" }
                    " and "
                    Code { source: "use_throttled_callback" }
                    " do the same for a call: the returned callback takes the argument "
                    "and runs yours later, with the last one. A throttled callback runs "
                    "its first call at once."
                }
            }

            DocSection {
                title: "Signals",
                Text {
                    "Pass a signal as "
                    Code { source: "query.into()" }
                    ". The first value shows at once, and a change that is undone inside "
                    "the delay never shows. A change lands from an effect, one render "
                    "after its source. It runs on the same timer on the web and "
                    "natively; a server render never follows a change."
                }
            }
        }
    }
}
