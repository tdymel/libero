# Hotkeys

Crate: `libero`
Import: `use libero::hooks::{Hotkey, use_hotkeys};`
Feature: `use-hotkeys` (on by default, turns on `use-popover`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/hotkeys.rs>
Index: [index.md](index.md) lists every other page
Description: Runs a handler on a keyboard shortcut from anywhere in the page, skipping text entry unless asked, and stops when the component unmounts.

`use_hotkeys(bindings)` runs a handler when its chord is pressed anywhere in the
document, even with focus on the page body or in a portal. Each binding is
`Hotkey::new(chord, handler)`. A chord joins modifiers (`ctrl`, `alt`, `shift`,
`meta`, `mod`) and a key with `+`, such as `mod+shift+k`; `mod` is Cmd on Apple
platforms and Ctrl elsewhere. Presses in text entry are skipped unless the
binding calls `.include_editable(true)`; `.when(guard)` and `.within(element)`
narrow it further. A hotkey lives as long as the component that calls the hook.

## Usage

```rust
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
fn Shortcuts() -> Element {
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
```

## API

```rust,ignore
pub fn use_hotkeys(bindings: impl IntoIterator<Item = Hotkey>)
impl Hotkey {
    pub fn new(chord: impl Into<String>, handler: impl FnMut() + 'static) -> Self
    pub fn include_editable(self, include: bool) -> Self
    pub fn when(self, guard: impl Fn() -> bool + 'static) -> Self
    pub fn within(self, element: ElementHandle) -> Self
}
```

| Method | Description |
|---|---|
| `Hotkey::new(chord, handler)` | A shortcut. The handler runs in your component's scope, once per press. |
| `include_editable(bool)` | Also fire while typing in a text field, `textarea`, `select` or editable content. Off by default. |
| `when(guard)` | Lets a press through only while `guard` answers `true`, checked at each press. A turned-away press keeps its default action. |
| `within(element)` | Lets a press through only with focus on `element` or inside it, a libero popup opened from it included. Repeat to add elements, such as your own portaled box. A press elsewhere keeps its default action. Spread `..element.attributes()` on it for a WebView. |

Bindings are read again every render, so handlers see current state. A chord
that parses to no key is ignored.

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `F2` | Runs the first demo shortcut. Not Ctrl+K: the docs search has it. |
| `F8` | Runs the second one, also while typing in the field. |
| `Cmd+B` or `Ctrl+B` | Runs the third one, only with focus in the second field. |

### Libero handles

- A shortcut is not taken from a text field, a `textarea`, a `select` or
  editable content unless you ask with `include_editable`.
- A press never fires mid-composition, so an IME is not cut off, and a held key
  runs the handler once.
- A debug build warns once about a chord that shadows Tab, Escape, the arrows or
  a browser shortcut such as Ctrl+L.
- Listening stops when the component unmounts.

### You must

- Offer the same action another way: a visible control or menu item. A shortcut
  alone excludes anyone who cannot press the chord (WCAG 2.1.4, 2.5.1).
- Tell the reader the chord exists, on the control it triggers or in a help
  dialog. The demo's buttons show it and name it in `aria-keyshortcuts`.

### Example

A bold shortcut, `Hotkey::new("mod+b", ..)`, next to a Bold button whose
tooltip says "Ctrl+B": the button does the same for anyone who cannot press
the chord, and the tooltip tells everyone else it exists.

### Limits

- A WebView prevents a chord's default action only from the second press of it.
- Native Blitz hears presses that bubble out of the app, and a server render
  binds nothing.
