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
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_indexed_db(key, default) -> StoredAsync<T>" }
                    " keeps a value across reloads and app runs like "
                    Code { source: "use_local_storage" }
                    ", but loads and saves without blocking the page. Pick it over "
                    Code { source: "use_local_storage" }
                    " for values of more than a few kilobytes, such as a document, a draft or a list of rows: "
                    Code { source: "localStorage" }
                    " reads and writes on the main thread and holds about 5 MB, IndexedDB holds far more. For a small setting read at the first render, "
                    Code { source: "use_local_storage" }
                    " shows the stored value at once."
                }
                Text {
                    "The value arrives after the first render. Until "
                    Code { source: "is_loaded()" }
                    " is true, "
                    Code { source: "get()" }
                    " shows the default. A "
                    Code { source: "set" }
                    ", "
                    Code { source: "update" }
                    " or "
                    Code { source: "remove" }
                    " before the load lands wins and the stored value is dropped. Writes show at once and are saved afterwards; "
                    Code { source: "error()" }
                    " reports a save that failed. "
                    Code { source: "get()" }
                    ", "
                    Code { source: "is_stored()" }
                    ", "
                    Code { source: "is_loaded()" }
                    " and "
                    Code { source: "error()" }
                    " are reactive, and every handle of the same type on a key shows the same value."
                }
                Text {
                    "Web: one IndexedDB database, "
                    Code { source: "libero" }
                    ", with one store. A write or remove in one tab shows in the origin's other tabs at once. Blitz (the "
                    Code { source: "native" }
                    " feature), a desktop app with the "
                    Code { source: "desktop" }
                    " feature and Android: a file per key in the app's data directory, next to "
                    Code { source: "use_local_storage" }
                    "'s. Any other build keeps values in memory until you call "
                    Code { source: "libero::platform::set_storage_dir(path)" }
                    ". Blocked IndexedDB, as in some private modes, reports "
                    Code { source: "Unavailable" }
                    " and the value lasts the session."
                }
                Text {
                    "Values are stored as JSON, so a "
                    Code { source: "String" }
                    " is kept with its quotes; binary data is not supported. Keys are used verbatim. Each write hits the store, so pass a fast source such as a slider through "
                    Code { source: "use_debounced_value" }
                    " first."
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
