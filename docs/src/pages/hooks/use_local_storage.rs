use crate::components::{Demo, DemoFile, DemoValues, DocPage, a11y};
use dioxus::prelude::*;
use libero::components::{Code, Text};

mod demo;
use demo::KeptDraft;

#[component]
pub fn UseLocalStoragePage() -> Element {
    rsx! {
        DocPage {
            title: "Local storage",
            source: "libero/src/hooks/storage.rs",
            markdown: "/md/use_local_storage.md",
            accessibility: a11y()
                .handles([
                    "It announces nothing and moves no focus: the value shows where you render it.",
                    "A failed save never loses the value for the session; error() says why.",
                ])
                .must([
                    "Say what is kept and for how long next to the control, as the demo's label does.",
                    "Offer a way to forget a kept value.",
                    "Announce a failed save once in a status region where the user relies on it, as the demo does.",
                    "Keep personal data out of local storage unless the user agreed: it stays on the device, readable by every script of the site.",
                ])
                .example("A comment box keeps its draft: after a reload the text is back, a \"Discard draft\" button forgets it, and a status line says \"Draft not saved\" once when the store is full.")
                .limits([
                    "A kept value lives in one browser or app install: another device starts from the default.",
                    "Off the web a value is a file per key in the app's data directory; any other build keeps values in memory until you call `libero::platform::set_storage_dir(path)`, and other windows and processes read a value at their mount.",
                    "Values are stored as JSON, so a `String` is kept with its quotes. `lsx-` keys are libero's own, read at start: change the colour scheme live through `use_color_scheme`. Pass a fast source such as a slider through `use_debounced_value` first.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_local_storage(key, default) -> Stored<T>" }
                    " keeps a value across reloads and app runs, "
                    Code { source: "use_session_storage(key, default)" }
                    " for the tab or the window's run. Read "
                    Code { source: "get()" }
                    ", "
                    Code { source: "is_stored()" }
                    " and "
                    Code { source: "error()" }
                    "; all are reactive. "
                    Code { source: "set(value)" }
                    ", "
                    Code { source: "update(change)" }
                    " and "
                    Code { source: "remove()" }
                    " write through; every handle of the same type on a key shows the same value."
                }
                Text {
                    Code { source: "use_local_storage_with(key, default, options)" }
                    " takes "
                    Code { source: "StorageOptions" }
                    ": in a hydrating fullstack app set "
                    Code { source: "read_after_mount: true" }
                    ", and "
                    Code { source: "format: StorageFormat::Text" }
                    " keeps bare text."
                }
            },

            Demo {
                component: "KeptDraft",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { KeptDraft {} },
                file: DemoFile(include_str!("use_local_storage/demo.rs")),
            }
        }
    }
}
