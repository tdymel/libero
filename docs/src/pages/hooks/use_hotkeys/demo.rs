use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Kbd, Text, TextField},
    hooks::{Hotkey, use_element, use_hotkeys},
    sx::sx,
};

#[derive(Clone, Copy, PartialEq)]
enum Action {
    Search,
    Help,
    Bold,
}

/// "opened 1 time", "opened 2 times".
fn times(count: u32) -> String {
    match count {
        1 => "1 time".into(),
        n => format!("{n} times"),
    }
}

#[component]
pub fn Shortcuts() -> Element {
    let mut searches = use_signal(|| 0u32);
    let mut helps = use_signal(|| 0u32);
    let mut bolds = use_signal(|| 0u32);
    let mut last = use_signal(|| None::<Action>);
    let mut run = move |action: Action| {
        match action {
            Action::Search => searches += 1,
            Action::Help => helps += 1,
            Action::Bold => bolds += 1,
        }
        last.set(Some(action));
    };
    let editor = use_element();
    use_hotkeys([
        Hotkey::new("f2", move || run(Action::Search)),
        Hotkey::new("f8", move || run(Action::Help)).include_editable(true),
        Hotkey::new("mod+b", move || run(Action::Bold))
            .include_editable(true)
            .within(editor),
    ]);
    // One line, naming only the last action, so a press is announced once.
    let said = match last() {
        None => "No shortcut pressed yet".to_string(),
        Some(Action::Search) => format!("Search opened {}", times(searches())),
        Some(Action::Help) => format!("Help opened {}", times(helps())),
        Some(Action::Bold) => format!("Bold pressed {}", times(bolds())),
    };

    rsx! {
        Flex { direction: "column", gap: "sm",
            // The same actions without a keyboard. Each shows its shortcut; `aria-keyshortcuts` tells a
            // screen reader, so the key cap stays out of the button's name.
            Flex { gap: "xs", wrap: "wrap",
                Button {
                    variant: "outlined",
                    "aria-keyshortcuts": "F2",
                    onclick: move |_| run(Action::Search),
                    "Search"
                    Kbd { aria_hidden: "true", sx: sx().margin_inline_start("xs"), "F2" }
                }
                Button {
                    variant: "outlined",
                    "aria-keyshortcuts": "F8",
                    onclick: move |_| run(Action::Help),
                    "Help"
                    Kbd { aria_hidden: "true", sx: sx().margin_inline_start("xs"), "F8" }
                }
                Button {
                    variant: "outlined",
                    "aria-keyshortcuts": "Control+B Meta+B",
                    onclick: move |_| run(Action::Bold),
                    "Bold"
                    Kbd { aria_hidden: "true", sx: sx().margin_inline_start("xs"), "Ctrl+B" }
                }
            }
            Text { role: "status", "{said}" }
            TextField { label: "Press F8 while typing" }
            div { onmounted: editor.mount(), ..editor.attributes(),
                TextField { label: "Press Ctrl+B (Cmd+B on a Mac) in here" }
            }
        }
    }
}
