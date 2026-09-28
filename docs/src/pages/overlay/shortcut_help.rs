use crate::components::{Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, Kbd, Shortcut, ShortcutHelp, Text};

// snippet: in ShortcutHelp { .. }
const SHORTCUTS: &str = r#"shortcuts: vec![
    Shortcut::new("mod+b", "Bold"), // Ctrl+B, or Cmd+B on a Mac
    Shortcut::new("mod+i", "Italic"),
    Shortcut::new("shift+mod+k", "Insert a link"),
    Shortcut::new("alt+f10", "Go to the toolbar"),
]"#;

#[component]
pub fn ShortcutHelpPage() -> Element {
    rsx! {
        DocPage {
            title: "ShortcutHelp",
            source: "libero/src/components/overlay/shortcut_help.rs",
            markdown: "/md/shortcut_help.md",
            properties: vec![
                props("ShortcutHelp", vec![
                    prop("shortcuts", "Vec<Shortcut>")
                        .default("required")
                        .doc("The rows, each `Shortcut::new(chord, what it does)`. A chord `use_hotkeys` would not bind is left out, with a warning in debug builds."),
                    prop("title", "String")
                        .doc("The heading. Unset, the localization's `shortcut_help.title`, \"Keyboard shortcuts\"."),
                ]),
            ],
            accessibility: a11y()
                .handles([
                    "A `Dialog` named by its title, the shortcuts a description list: each chord a `dt`, what it does the `dd`.",
                    "Each key is a `kbd`. `mod` shows as Cmd on Apple platforms and Ctrl elsewhere; the key names come from the localization.",
                ])
                .must([
                    "Open it with `use_modal`, which traps focus, closes on Escape and hands focus back.",
                    "List only shortcuts that work where the reader is.",
                ]),
            lead: rsx! {
                Text {
                    "A dialog listing keyboard shortcuts. Chords are written as "
                    Code { source: "use_hotkeys" }
                    " takes them and shown in the platform's key names, so "
                    Code { source: "\"mod+b\"" }
                    " reads "
                    Kbd { "Ctrl" }
                    " + "
                    Kbd { "B" }
                    " on Windows and Linux and "
                    Kbd { "Cmd" }
                    " + "
                    Kbd { "B" }
                    " on a Mac. Open it with "
                    Code { source: "use_modal" }
                    ", often from a "
                    Code { source: "shift+?" }
                    " hotkey; here it is shown inline."
                }
                Text {
                    "It belongs to the app: one dialog for the shortcuts of the screen the reader is on. "
                    Code { source: "RichTextEditor" }
                    " opens its own from "
                    Kbd { "Ctrl" }
                    " + "
                    Kbd { "/" }
                    ". "
                    Code { source: "Video" }
                    " and "
                    Code { source: "Audio" }
                    " open their own from "
                    Kbd { "Shift" }
                    " + "
                    Kbd { "?" }
                    " with focus in the player. "
                    Code { source: "Kanban" }
                    ", "
                    Code { source: "Sortable" }
                    " and "
                    Code { source: "Table" }
                    " have no shortcuts: their drag handles describe their keys to screen readers."
                }
            },
            Demo {
                component: "ShortcutHelp",
                children_text: "",
                children_code: "",
                fixed: vec![SHORTCUTS.to_string()],
                controls: vec![],
                render: move |_: DemoValues| rsx! {
                    ShortcutHelp {
                        shortcuts: vec![
                            Shortcut::new("mod+b", "Bold"),
                            Shortcut::new("mod+i", "Italic"),
                            Shortcut::new("shift+mod+k", "Insert a link"),
                            Shortcut::new("alt+f10", "Go to the toolbar"),
                        ],
                    }
                },
            }
        }
    }
}
