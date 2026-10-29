use crate::components::{Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, Kbd, Shortcut, ShortcutHelp, Text};

// snippet: in ShortcutHelp { .. }
const SHORTCUTS: &str = r#"shortcuts: vec![
    Shortcut::new("mod+b", "Bold"), // Ctrl+B, or Cmd+B on a Mac
    Shortcut::new("mod+i", "Italic"),
    Shortcut::new("shift+mod+k", "Insert a link"),
    Shortcut::new("alt+f10", "Go to the toolbar"),
    Shortcut::new("escape", "Close"),
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
                    prop("close_label", "String").default("\"Close\"").doc("The close button's accessible name, such as \"Close shortcuts\". Unset, the localization's `common.close`."),
                    prop("full_screen", "DialogFullScreen")
                        .default("never")
                        .doc("Inside a modal, fills the screen edge to edge instead of centring: `\"phone\"` below the `sm` breakpoint, `\"always\"` at every width. The list then grows past its 24rem cap and the modal scrolls. Outside a modal it does nothing."),
                ]),
            ],
            accessibility: a11y()
                .handles([
                    "A `Dialog` named by its title, the shortcuts a description list: each chord a `dt`, what it does the `dd`.",
                    "Each key is a `kbd`. `mod` shows as Cmd on Apple platforms and Ctrl elsewhere. The modifiers, Space and named keys such as Escape take their names from the localization's `ShortcutHelpLabels`; a key missing from its `key_names` shows its `Key` name, `F10`.",
                    "A list taller than 24rem or 60% of the window scrolls inside the dialog, and only then becomes a tab stop, a `region` named \"Shortcut list\" (the localization's `shortcut_help.list`), so the keyboard can scroll it.",
                ])
                .must([
                    "Open it with `use_modal`, which traps focus, closes on Escape and hands focus back.",
                    "List only shortcuts that work where the reader is.",
                ])
                .example("A `ShortcutHelp` opened with `use_modal` on \"?\": focus moves into the dialog, a screen reader reads each chord and what it does as a pair, and Escape returns focus."),
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
                    " hotkey; here it is shown inline. It belongs to the app, one dialog for the "
                    "screen the reader is on: "
                    Code { source: "RichTextEditor" }
                    ", "
                    Code { source: "Video" }
                    " and "
                    Code { source: "Audio" }
                    " open their own."
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
                            Shortcut::new("escape", "Close"),
                        ],
                    }
                },
            }
        }
    }
}
