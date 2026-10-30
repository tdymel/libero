use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Text, TextField},
    hooks::{StorageError, use_debounced_callback, use_local_storage, use_session_storage},
};

// demo-code: start
#[component]
pub fn KeptDraft() -> Element {
    let mut draft = use_local_storage("libero-docs-draft", String::new);
    let mut presses = use_session_storage("libero-docs-presses", || 0_u32);
    // The field follows every key; the store gets the text once typing pauses.
    let mut typed = use_signal(|| None::<String>);
    let save = use_debounced_callback(
        move |()| {
            if let Some(text) = typed.peek().clone() {
                draft.set(text);
            }
        },
        400,
    );
    // Rendered from the start, so a failed save is announced once.
    let saved = match draft.error() {
        Some(StorageError::Invalid) => "The kept draft could not be read: starting empty",
        Some(StorageError::Full) => {
            "Not saved, this device's storage is full: kept until you leave"
        }
        Some(StorageError::Unavailable) => "Not saved, storage is off here: kept until you leave",
        None if draft.is_stored() => "Saved on this device",
        None => "",
    };

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            TextField {
                label: "Draft, kept after a reload",
                value: typed().unwrap_or_else(|| draft.get()),
                oninput: move |next| {
                    typed.set(Some(next));
                    save.call(());
                },
            }
            div { role: "status", "{saved}" }
            Flex { direction: "row", gap: "sm",
                Button {
                    variant: "outlined",
                    onclick: move |_| {
                        typed.set(None);
                        draft.remove();
                    },
                    "Forget draft"
                }
                Button {
                    variant: "outlined",
                    onclick: move |_| presses.update(|count| *count += 1),
                    "Count in this tab"
                }
            }
            Text { size: "sm", "Pressed {presses.get()} times in this tab" }
        }
    }
}
// demo-code: end
