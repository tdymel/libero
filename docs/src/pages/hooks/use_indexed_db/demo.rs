use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, TextField},
    hooks::use_indexed_db,
};

// demo-code: start
#[component]
pub fn KeptNotes() -> Element {
    let mut notes = use_indexed_db("libero-docs-notes", String::new);
    let status = match notes.error() {
        _ if !notes.is_loaded() => "Loading your notes".to_string(),
        Some(error) => format!("Not saved ({error:?}): kept until you leave"),
        None if notes.is_stored() => "Saved on this device".to_string(),
        None => String::new(),
    };

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            TextField {
                label: "Notes, kept after a reload",
                value: notes.get(),
                disabled: !notes.is_loaded(),
                oninput: move |next| notes.set(next),
            }
            div { role: "status", "{status}" }
            Button { variant: "outlined", onclick: move |_| notes.remove(), "Forget notes" }
        }
    }
}
// demo-code: end
