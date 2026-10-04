use crate::components::{Demo, DemoValues, DocPage, DocSection, Wrap, a11y};
use dioxus::prelude::*;
use libero::{
    components::{Code, Flex, Kbd, Text, TextField},
    hooks::{Hotkey, use_element, use_hotkeys},
};

/// The hook in one component, as `Shortcuts` renders it.
// snippet: mirrors Shortcuts
fn code(_: &DemoValues, _: &str) -> String {
    r#"let mut searches = use_signal(|| 0);
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
}"#
    .to_string()
}

#[component]
fn Shortcuts() -> Element {
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

#[component]
pub fn UseHotkeysPage() -> Element {
    rsx! {
        DocPage {
            title: "Hotkeys",
            source: "libero/src/hooks/hotkeys.rs",
            markdown: "/md/use_hotkeys.md",
            accessibility: a11y()
                .key(["F2"], "Runs the first demo shortcut. Not Ctrl+K: the docs search has it.")
                .key(["F8"], "Runs the second one, also while typing in the field.")
                .key(["Cmd+B", "Ctrl+B"], "Runs the third one, only with focus in the second field.")
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
                .example("A bold shortcut, `Hotkey::new(\"mod+b\", ..)`, next to a Bold button whose tooltip says \"Ctrl+B\": the button does the same for anyone who cannot press the chord, and the tooltip tells everyone else it exists.")
                .limits([
                    "A WebView prevents a chord's default action only from the second press of it.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_hotkeys(bindings)" }
                    " runs a handler when its chord is pressed anywhere in the document, "
                    "even with focus on the page body or in a portal. Each binding is "
                    Code { source: "Hotkey::new(chord, handler)" }
                    "."
                }
            },

            Demo {
                component: "use_hotkeys",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { Shortcuts {} },
                wrap: Wrap(code),
            }

            DocSection {
                title: "Chords",
                Text {
                    "A chord is modifiers and a key joined by "
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
            }

            DocSection {
                title: "Text entry and guards",
                Text {
                    "Presses in text entry are skipped unless the binding calls "
                    Code { source: ".include_editable(true)" }
                    ". "
                    Code { source: ".when(guard)" }
                    " lets a press through only while the guard answers "
                    Code { source: "true" }
                    ". "
                    Code { source: ".within(element)" }
                    " lets it through only with focus on or inside an element from "
                    Code { source: "use_element" }
                    ". A libero popup opened inside it, such as a "
                    Code { source: "Menu" }
                    ", a "
                    Code { source: "Select" }
                    " list or a "
                    Code { source: "HoverCard" }
                    ", counts as inside, as does a "
                    Code { source: "use_popover" }
                    " box with its "
                    Code { source: "anchor_events()" }
                    " and "
                    Code { source: "floating_events()" }
                    " spread; call it again to add your own portaled box. "
                    "Spread "
                    Code { source: "..element.attributes()" }
                    " on it, so a WebView finds it. "
                    "A press a binding takes has its default action prevented; one it turns away keeps it. "
                    "Native Blitz hears presses that bubble out of the app, and a server render binds nothing."
                }
            }

            DocSection {
                title: "Lifetime",
                Text {
                    "A hotkey lives as long as the component that calls the hook: it is bound when "
                    "the component mounts and removed when it unmounts, such as when a route change "
                    "drops the page. A hook in a layout or the app root stays bound across navigation. "
                    "Each call listens on its own, so two components with the same chord both run."
                }
            }
        }
    }
}
