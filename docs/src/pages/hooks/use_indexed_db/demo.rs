use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, TextField},
    hooks::{StorageError, use_debounced_callback, use_indexed_db, use_timeout},
};

// demo-code: start
#[component]
pub fn KeptNotes() -> Element {
    let mut notes = use_indexed_db("libero-docs-notes", String::new);
    // The field follows every key; the store gets the text once typing pauses.
    let mut typed = use_signal(|| None::<String>);
    let save = use_debounced_callback(
        move |()| {
            if let Some(text) = typed.peek().clone() {
                notes.set(text);
            }
        },
        400,
    );
    // Leaving the page drops a pending save: keep the last keys typed too.
    use_drop(move || {
        if let Ok(Some(text)) = typed.try_peek().map(|text| text.clone()) {
            notes.set(text);
        }
    });
    // A fast load shows no loading text at all.
    let mut slow = use_signal(|| false);
    let wait = use_timeout(move || slow.set(true), 500);
    use_hook(move || wait.start());
    // Rendered from the start, so a failed save is announced once.
    let status = match notes.error() {
        _ if !notes.is_loaded() && slow() => "Loading your notes",
        _ if !notes.is_loaded() => "",
        Some(StorageError::Invalid) => "The kept notes could not be read: starting empty",
        Some(StorageError::Full) => {
            "Not saved, this device's storage is full: kept until you leave"
        }
        Some(StorageError::Unavailable) => "Not saved, storage is off here: kept until you leave",
        None if notes.is_stored() => "Saved on this device",
        None => "",
    };

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            TextField {
                label: "Notes, kept after a reload",
                value: typed().unwrap_or_else(|| notes.get()),
                disabled: !notes.is_loaded(),
                aria_busy: !notes.is_loaded(),
                oninput: move |next| {
                    typed.set(Some(next));
                    save.call(());
                },
            }
            div { role: "status", "{status}" }
            Button {
                variant: "outlined",
                disabled: !notes.is_loaded(),
                focusable_when_disabled: true,
                onclick: move |_| {
                    typed.set(None);
                    notes.remove();
                },
                "Forget notes"
            }
        }
    }
}
// demo-code: end
