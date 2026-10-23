use crate::components::{Demo, DemoFile, DemoValues, DocPage, a11y};
use dioxus::prelude::*;
use libero::components::{Code, Text};

mod demo;
use demo::KeptNotes;

#[component]
pub fn UseIndexedDbPage() -> Element {
    rsx! {
        DocPage {
            title: "IndexedDB",
            source: "libero/src/hooks/indexed_db.rs",
            markdown: "/md/use_indexed_db.md",
            accessibility: a11y()
                .handles([
                    "It announces nothing and moves no focus: the value shows where you render it.",
                    "A failed save never loses the value for the session; error() says why.",
                ])
                .must([
                    "Show a loading state until is_loaded() is true, or disable the control: the default shows first, and typing over it before the load lands replaces the stored value.",
                    "Say what is kept and for how long next to the control, as the demo's label does.",
                    "Offer a way to forget a kept value.",
                    "Announce a failed save once in a status region where the user relies on it, as the demo does.",
                    "Keep personal data out of IndexedDB unless the user agreed: it stays on the device, readable by every script of the site.",
                ])
                .example("A notes box keeps its text: while it loads the field is disabled and a status line says \"Loading your notes\", after a reload the text is back, and a \"Forget notes\" button clears it.")
                .limits([
                    "A kept value lives in one browser or app install: another device starts from the default.",
                    "Web: one IndexedDB database, `libero`, with one store; a write in one tab shows in the origin's other tabs at once. Blitz, a desktop app with the `desktop` feature and Android keep a file per key in the app's data directory. Any other build keeps values in memory until you call `libero::platform::set_storage_dir(path)`.",
                    "Blocked IndexedDB, as in some private modes, reports `Unavailable` and the value lasts the session.",
                    "Values are stored as JSON, so a `String` is kept with its quotes; binary data is not supported. Pass a fast source such as a slider through `use_debounced_value` first.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_indexed_db(key, default) -> StoredAsync<T>" }
                    " keeps a value across reloads and app runs like "
                    Code { source: "use_local_storage" }
                    ", but loads and saves without blocking the page. Pick it for values of more than a few kilobytes, such as a document or a list of rows: "
                    Code { source: "localStorage" }
                    " blocks the main thread and holds about 5 MB."
                }
                Text {
                    "The value arrives after the first render: until "
                    Code { source: "is_loaded()" }
                    " is true, "
                    Code { source: "get()" }
                    " shows the default. A write before the load lands wins. "
                    Code { source: "error()" }
                    " reports a save that failed."
                }
            },

            Demo {
                component: "KeptNotes",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { KeptNotes {} },
                file: DemoFile(include_str!("use_indexed_db/demo.rs")),
            }
        }
    }
}
