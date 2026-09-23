# Hotkeys

Crate: `libero`
Import: `use libero::hooks::{Hotkey, use_hotkeys};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/hotkeys.rs>
Index: [index.md](index.md) lists every other page
Description: Runs a handler on a keyboard shortcut from anywhere in the page, skipping text entry unless asked, and stops when the component unmounts.

`use_hotkeys(bindings)` runs a handler when its chord is pressed anywhere in the
document, even with focus on the page body or in a portal. Each binding is
`Hotkey::new(chord, handler)`. A chord is modifiers and a key joined by `+`:
`ctrl`, `alt`, `shift`, `meta` (or `cmd`) and `mod`, then a character or a key
name such as `f8`, `escape` or `space`. `mod` is Cmd on Apple platforms and Ctrl
elsewhere. Modifiers match exactly, so `mod+k` ignores `mod+shift+k`.

Presses in text entry are skipped unless the binding calls
`.include_editable(true)`. `.when(guard)` lets a press through only while the
guard answers `true`. A press a binding takes has its default action prevented.
Native Blitz hears presses that bubble out of the app, and a server render binds
nothing.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Flex, Text, TextField},
    hooks::{Hotkey, use_hotkeys},
};

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
            TextField { label: "Press Alt+H while typing" }
        }
    }
}
```

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Cmd+K` or `Ctrl+K` | Runs the first demo shortcut: Cmd on macOS, Ctrl elsewhere. |
| `Alt+H` | Runs the second one, also while typing in the field. |

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
  dialog.

### Limits

- A WebView prevents a chord's default action only from the second press of it.

## API

```rust,ignore
pub fn use_hotkeys(bindings: impl IntoIterator<Item = Hotkey>)
impl Hotkey {
    pub fn new(chord: impl Into<String>, handler: impl FnMut() + 'static) -> Self
    pub fn include_editable(self, include: bool) -> Self
    pub fn when(self, guard: impl Fn() -> bool + 'static) -> Self
}
```

| Method | Description |
|---|---|
| `Hotkey::new(chord, handler)` | A shortcut. The handler runs in your component's scope, once per press. |
| `include_editable(bool)` | Also fire while typing in a text field, `textarea`, `select` or editable content. Off by default. |
| `when(guard)` | Lets a press through only while `guard` answers `true`, checked at each press. A turned-away press keeps its default action. |

Bindings are read again every render, so handlers see current state. A chord
that parses to no key is ignored.
