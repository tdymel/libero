use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, TextField},
    hooks::{UndoHistory, use_history},
};

#[component]
pub fn UndoableNote() -> Element {
    let note = use_history(|| UndoHistory::new(String::new()), 500);

    rsx! {
        Flex {
            direction: "column",
            align: "flex-start",
            gap: "sm",
            onkeydown: move |event: KeyboardEvent| {
                let modifiers = event.modifiers();
                let Key::Character(key) = event.key() else { return };
                // Ctrl+Y is Windows' redo; Cmd+Y is history on macOS, so it is left alone.
                let redo_y = modifiers.ctrl() && !modifiers.shift() && key.eq_ignore_ascii_case("y");
                let z = (modifiers.ctrl() || modifiers.meta()) && key.eq_ignore_ascii_case("z");
                if !redo_y && !z {
                    return;
                }
                event.prevent_default();
                if redo_y || modifiers.shift() {
                    note.redo();
                } else {
                    note.undo();
                }
            },
            TextField {
                label: "Note",
                value: note.value().to_string(),
                oninput: move |next| note.merge(next),
            }
            Flex { direction: "row", gap: "sm",
                Button {
                    variant: "outlined",
                    disabled: !note.can_undo(),
                    focusable_when_disabled: true,
                    onclick: move |_| {
                        note.undo();
                    },
                    "Undo"
                }
                Button {
                    variant: "outlined",
                    disabled: !note.can_redo(),
                    focusable_when_disabled: true,
                    onclick: move |_| {
                        note.redo();
                    },
                    "Redo"
                }
                Button {
                    variant: "text",
                    disabled: note.value().is_empty(),
                    focusable_when_disabled: true,
                    // One undo step, so Undo brings the text back.
                    onclick: move |_| note.push(String::new()),
                    "Clear"
                }
            }
        }
    }
}
