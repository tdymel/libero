use crate::components::{Demo, DemoValues, DocPage, Wrap, a11y};
use dioxus::prelude::*;
use libero::{
    components::{Code, Flex, Kbd, Text},
    hooks::{Hotkey, use_hotkeys},
};

/// The hook in one component, as `Shortcuts` renders it.
fn code(_: &DemoValues, _: &str) -> String {
    r#"let mut searches = use_signal(|| 0);
let mut helps = use_signal(|| 0);
use_hotkeys([
    Hotkey::new("mod+k", move || searches += 1),
    Hotkey::new("alt+h", move || helps += 1).include_editable(true),
]);

rsx! {
    Flex { direction: "column", gap: "sm",
        Text { "Search opened {searches} times" }
        Text { "Help opened {helps} times" }
        input { placeholder: "Type here" }
    }
}"#
    .to_string()
}

#[component]
fn Shortcuts() -> Element {
    let mut searches = use_signal(|| 0);
    let mut helps = use_signal(|| 0);
    use_hotkeys([
        Hotkey::new("mod+k", move || searches += 1),
        Hotkey::new("alt+h", move || helps += 1).include_editable(true),
    ]);

    rsx! {
        Flex { direction: "column", gap: "sm",
            Text { "Search opened {searches} times" }
            Text { "Help opened {helps} times" }
            input { placeholder: "Type here" }
        }
    }
}

#[component]
pub fn UseHotkeysPage() -> Element {
    rsx! {
        DocPage {
            title: "use_hotkeys",
            source: "libero/src/hooks/hotkeys.rs",
            markdown: "/md/use_hotkeys.md",
            accessibility: a11y()
                .key(["Cmd+K", "Ctrl+K"], "Runs the first demo shortcut: Cmd on macOS, Ctrl elsewhere.")
                .key(["Alt+H"], "Runs the second one, also while typing in the field.")
                .handles([
                    "A shortcut is not taken from a text field, a `textarea`, a `select` or editable content unless you ask with `include_editable`.",
                    "A press never fires mid-composition, so an IME is not cut off, and a held key runs the handler once.",
                    "A debug build warns once about a chord that shadows Tab, Escape, the arrows or a browser shortcut such as Ctrl+L.",
                    "Listening stops when the component unmounts.",
                ])
                .must([
                    "Offer the same action another way: a visible control or menu item. A shortcut alone excludes anyone who cannot press the chord (WCAG 2.1.4, 2.5.1).",
                    "Tell the reader the chord exists, on the control it triggers or in a help dialog.",
                ])
                .limits([
                    "A WebView prevents a chord's default action only from the second press of it.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_hotkeys(bindings)" }
                    " runs a handler when its chord is pressed anywhere in the document, "
                    "even with focus on the page body or in a portal. Each binding is "
                    Code { source: "Hotkey::new(chord, handler)" }
                    ". A chord is modifiers and a key joined by "
                    Code { source: "+" }
                    ": "
                    Code { source: "ctrl" }
                    ", "
                    Code { source: "alt" }
                    ", "
                    Code { source: "shift" }
                    ", "
                    Code { source: "meta" }
                    " (or "
                    Code { source: "cmd" }
                    ") and "
                    Code { source: "mod" }
                    ", then a character or a key name such as "
                    Code { source: "f8" }
                    ", "
                    Code { source: "escape" }
                    " or "
                    Code { source: "space" }
                    ". "
                    Code { source: "mod" }
                    " is Cmd on Apple platforms and Ctrl elsewhere. Modifiers match exactly, "
                    "so "
                    Kbd { "mod+k" }
                    " ignores "
                    Kbd { "mod+shift+k" }
                    "."
                }
                Text {
                    "Presses in text entry are skipped unless the binding calls "
                    Code { source: ".include_editable(true)" }
                    ". "
                    Code { source: ".when(guard)" }
                    " lets a press through only while the guard answers "
                    Code { source: "true" }
                    ". A press a binding takes has its default action prevented. "
                    "Native Blitz hears presses that bubble out of the app, and a server render binds nothing."
                }
            },

            Demo {
                component: "use_hotkeys",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { Shortcuts {} },
                wrap: Wrap(code),
            }
        }
    }
}
