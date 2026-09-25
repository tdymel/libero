//! `Toolbar`, for the `RovingTabindex` archetype: every kind of item, a vertical bar, RTL.

use dioxus::prelude::*;
use libero::components::{
    ActionIcon, Button, ButtonGroup, Checkbox, Flex, NumberField, Options, SegmentedControl,
    Select, Shortcut, ShortcutHelp, Switch, TextField, Toolbar, ToolbarGroup, ToolbarSeparator,
};
use libero::hooks::{ModalScope, use_element, use_modal};
use libero::platform::ElementApi;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/toolbar", || rsx! { ToolbarPage {} }),
    ("/toolbar-vertical", || rsx! { ToolbarVerticalPage {} }),
    ("/toolbar-rtl", || rsx! { ToolbarRtlPage {} }),
    (
        "/toolbar-script-focus",
        || rsx! { ToolbarScriptFocusPage {} },
    ),
    ("/toolbar-fields", || rsx! { ToolbarFieldsPage {} }),
    ("/toolbar-editor", || rsx! { ToolbarEditorPage {} }),
];

#[derive(Clone, PartialEq, Options)]
enum Font {
    Serif,
    Sans,
    Mono,
}

/// Icons in a group, a separator, a `ButtonGroup`, a disabled button and a `Select`.
#[component]
fn ToolbarPage() -> Element {
    let mut font = use_signal(|| Some(Font::Serif));
    let mut undo = use_signal(|| 0);
    let mut right = use_signal(|| 0);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "640px",
            Toolbar { "aria-label": "Formatting",
                ToolbarGroup { "aria-label": "Style",
                    ActionIcon { id: "bold", aria_label: "Bold", "B" }
                    ActionIcon { id: "italic", aria_label: "Italic", "I" }
                }
                ToolbarSeparator {}
                ButtonGroup { "aria-label": "Align", variant: "outlined",
                    Button { id: "left", "Left" }
                    Button { id: "right", onclick: move |_| right += 1, "Right" }
                }
                Button { id: "undo", disabled: true, onclick: move |_| undo += 1, "Undo" }
                Select { value: font(), onchange: move |next| font.set(next), "aria-label": "Font" }
            }
            button { id: "after", "after" }
            div { id: "presses", "data-undo": "{undo}", "data-right": "{right}" }
        }
    }
}

#[component]
fn ToolbarVerticalPage() -> Element {
    rsx! {
        Toolbar { "aria-label": "Tools", orientation: "vertical", loop_focus: false,
            ActionIcon { id: "pen", aria_label: "Pen", "P" }
            ToolbarSeparator {}
            ActionIcon { id: "eraser", aria_label: "Eraser", "E" }
            ActionIcon { id: "fill", aria_label: "Fill", "F" }
        }
    }
}

/// Fields as items (1190): their own arrows until an end, then the bar's.
#[component]
fn ToolbarFieldsPage() -> Element {
    let mut bold = use_signal(|| false);
    let mut wrap = use_signal(|| false);
    let mut font = use_signal(|| Font::Serif);
    let mut find = use_signal(|| String::from("ab"));
    let mut size = use_signal(|| Some(12_i32));
    rsx! {
        Toolbar { "aria-label": "Fields",
            Button { id: "first", "First" }
            Checkbox { id: "bold", aria_label: "Bold", checked: bold(), onchange: move |next| bold.set(next) }
            Switch { id: "wrap", aria_label: "Wrap", checked: wrap(), onchange: move |next| wrap.set(next) }
            SegmentedControl {
                "aria-label": "Font",
                value: font(),
                onchange: move |next| font.set(next),
            }
            TextField { id: "find", aria_label: "Find", value: find(), oninput: move |next| find.set(next) }
            NumberField::<i32> { id: "size", aria_label: "Size", value: size(), onchange: move |next| size.set(next) }
        }
    }
}

/// An editor's bar (1193): Alt+F10 in the text reaches it, Escape hands focus back.
#[component]
fn ToolbarEditorPage() -> Element {
    let editor = use_element();
    let help = use_modal(|_: ModalScope<()>| {
        rsx! {
            ShortcutHelp {
                shortcuts: vec![
                    Shortcut::new("alt+f10", "Go to the toolbar"),
                    Shortcut::new("shift+mod+k", "Insert a link"),
                ],
            }
        }
    });
    rsx! {
        Toolbar { "aria-label": "Formatting", focus_from: editor,
            Button { id: "bold", "Bold" }
            Button { id: "italic", "Italic" }
            Button { id: "help", onclick: move |_| { help.open(); }, "Shortcuts" }
        }
        div { onmounted: editor.mount(), ..editor.attributes(),
            textarea { id: "text", "aria-label": "Text" }
        }
    }
}

/// A button outside moves focus into the bar by script, which Blitz does with no `focusin` (1192).
/// `#show` mounts `#zero` last but first in the bar: the arrows follow the DOM.
#[component]
fn ToolbarScriptFocusPage() -> Element {
    let page = use_element();
    let mut zero = use_signal(|| false);
    rsx! {
        div { onmounted: page.mount(),
            Toolbar { "aria-label": "Formatting",
                if zero() {
                    Button { id: "zero", "Zero" }
                }
                Button { id: "one", "One" }
                Button { id: "two", "Two" }
                Button { id: "three", "Three" }
            }
            button {
                id: "jump",
                onclick: move |_| {
                    if page.query_selector("#three").and_then(|three| three.focus()).is_err() {
                        // A WebView queries nothing: the page's own script does it.
                        document::eval("document.getElementById('three').focus();");
                    }
                },
                "Jump"
            }
            button { id: "show", onclick: move |_| zero.set(true), "Show zero" }
        }
    }
}

#[component]
fn ToolbarRtlPage() -> Element {
    rsx! {
        div { dir: "rtl",
            Toolbar { "aria-label": "Formatting",
                Button { id: "one", "One" }
                Button { id: "two", "Two" }
                Button { id: "three", "Three" }
            }
        }
    }
}
