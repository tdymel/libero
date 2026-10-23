use crate::components::{Demo, DemoFile, DemoValues, DocPage, a11y};
use dioxus::prelude::*;
use libero::components::{Code, Kbd, Text};

mod demo;
use demo::Shortcuts;

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
                    "Native Blitz hears presses that bubble out of the app, and a server render binds nothing.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_hotkeys(bindings)" }
                    " runs a handler when its chord is pressed anywhere in the document, "
                    "even with focus on the page body or in a portal. Each binding is "
                    Code { source: "Hotkey::new(chord, handler)" }
                    ". A chord joins modifiers ("
                    Code { source: "ctrl" }
                    ", "
                    Code { source: "alt" }
                    ", "
                    Code { source: "shift" }
                    ", "
                    Code { source: "meta" }
                    ", "
                    Code { source: "mod" }
                    ") and a key with "
                    Code { source: "+" }
                    ", such as "
                    Kbd { "mod+shift+k" }
                    "; "
                    Code { source: "mod" }
                    " is Cmd on Apple platforms and Ctrl elsewhere. Presses in text entry are skipped unless the binding calls "
                    Code { source: ".include_editable(true)" }
                    "; "
                    Code { source: ".when(guard)" }
                    " and "
                    Code { source: ".within(element)" }
                    " narrow it further. A hotkey lives as long as the component that calls the hook."
                }
            },

            Demo {
                component: "Shortcuts",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { Shortcuts {} },
                file: DemoFile(include_str!("use_hotkeys/demo.rs")),
            }
        }
    }
}
