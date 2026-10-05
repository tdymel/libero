use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Text, TextField},
    hooks::{use_local_storage, use_session_storage},
};

// demo-code: start
#[component]
pub fn KeptDraft() -> Element {
    let mut draft = use_local_storage("libero-docs-draft", String::new);
    let mut presses = use_session_storage("libero-docs-presses", || 0_u32);
    let saved = match draft.error() {
        Some(error) => format!("Not saved ({error:?}): kept until you leave"),
        None if draft.is_stored() => "Saved on this device".to_string(),
        None => String::new(),
    };

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            TextField {
                label: "Draft, kept after a reload",
                value: draft.get(),
                oninput: move |next| draft.set(next),
            }
            div { role: "status", "{saved}" }
            Flex { direction: "row", gap: "sm",
                Button { variant: "outlined", onclick: move |_| draft.remove(), "Forget draft" }
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
