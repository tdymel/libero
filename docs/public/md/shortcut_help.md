# ShortcutHelp

Crate: `libero`
Import: `use libero::components::{Shortcut, ShortcutHelp};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/shortcut_help.rs>
Index: [index.md](index.md) lists every other page
Description: A dialog listing keyboard shortcuts, each chord in the platform's key names.

A dialog listing keyboard shortcuts. Chords are written as
[`use_hotkeys`](use_hotkeys.md) takes them and shown in the platform's key
names, so `"mod+b"` reads Ctrl + B on Windows and Linux and Cmd + B on a Mac.
Open it with [`use_modal`](modal.md), often from a `shift+?` hotkey.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Shortcut, ShortcutHelp};
use libero::hooks::{Hotkey, ModalScope, use_hotkeys, use_modal};

#[component]
fn Demo() -> Element {
    let help = use_modal(|_: ModalScope<()>| {
        rsx! {
            ShortcutHelp {
                shortcuts: vec![
                    Shortcut::new("mod+b", "Bold"),
                    Shortcut::new("mod+i", "Italic"),
                    Shortcut::new("shift+mod+k", "Insert a link"),
                    Shortcut::new("alt+f10", "Go to the toolbar"),
                ],
            }
        }
    });
    use_hotkeys([Hotkey::new("shift+?", move || {
        help.open();
    })]);

    rsx! {}
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `shortcuts` | `Vec<Shortcut>` | required | The rows, each `Shortcut::new(chord, what it does)`. A chord `use_hotkeys` would not bind is left out, with a warning in debug builds. |
| `title` | `String` | - | The heading. Unset, the localization's `shortcut_help.title`, "Keyboard shortcuts". |

Like every component, `ShortcutHelp` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes. They go to its `Dialog`.

## Accessibility

### Libero handles

- A `Dialog` named by its title, the shortcuts a description list: each chord
  a `dt`, what it does the `dd`.
- Each key is a `kbd`. `mod` shows as Cmd on Apple platforms and Ctrl
  elsewhere; the key names come from the localization.

### You must

- Open it with `use_modal`, which traps focus, closes on Escape and hands focus
  back.
- List only shortcuts that work where the reader is.
