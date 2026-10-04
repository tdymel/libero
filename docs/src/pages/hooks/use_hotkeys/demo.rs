use dioxus::prelude::*;
use libero::{
    components::{Flex, Text, TextField},
    hooks::{Hotkey, use_element, use_hotkeys},
};

#[component]
pub fn Shortcuts() -> Element {
    let mut searches = use_signal(|| 0);
    let mut helps = use_signal(|| 0);
    let mut bolds = use_signal(|| 0);
    let editor = use_element();
    use_hotkeys([
        Hotkey::new("f2", move || searches += 1),
        Hotkey::new("f8", move || helps += 1).include_editable(true),
        Hotkey::new("mod+b", move || bolds += 1)
            .include_editable(true)
            .within(editor),
    ]);

    rsx! {
        Flex { direction: "column", gap: "sm",
            Text { "Search opened {searches} times" }
            Text { "Help opened {helps} times" }
            TextField { label: "Press F8 while typing" }
            div { onmounted: editor.mount(), ..editor.attributes(),
                TextField { label: "Press Ctrl+B (Cmd+B on a Mac) in here" }
            }
            Text { "Bold pressed {bolds} times" }
        }
    }
}
