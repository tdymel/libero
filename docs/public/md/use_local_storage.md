# Local storage

Crate: `libero`
Import: `use libero::hooks::{StorageError, Stored, use_local_storage, use_session_storage};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/storage.rs>
Index: [index.md](index.md) lists every other page
Description: A value kept under a key across reloads and app runs, or for the tab's session; a file per key off the web.

`use_local_storage(key, default) -> Stored<T>` keeps a value across reloads
and app runs, `use_session_storage(key, default)` for the tab or the window's
run. Read `get()`, `is_stored()` and `error()`; all are reactive. `set(value)`,
`update(change)` and `remove()` write through; every handle on a key shows the
same value.

Web: `localStorage` and `sessionStorage`; another tab's write arrives live.
Blitz (the `native` feature), a desktop app with the `desktop` feature and
Android: a file per key in the app's data directory, session values in memory
per window. Any other build, such as a server, liveview or a desktop app
without the feature, keeps values in memory until you call
`libero::platform::set_storage_dir(path)`. Off the web, other windows and
processes read a value at their mount.

Values are stored as JSON, so a `String` is kept with its quotes. Keys are used
verbatim; `lsx-` keys are libero's own. The value is read at the first render:
a hydrating server render shows the default and mismatches a stored value. Each
write hits the store (a file off the web), so pass a fast source such as a
slider through `use_debounced_value` first.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Text, TextField},
    hooks::{use_local_storage, use_session_storage},
};

#[component]
fn KeptDraft() -> Element {
    let mut draft = use_local_storage("draft", String::new);
    let mut presses = use_session_storage("presses", || 0_u32);
    // Rendered from the start, so a failed save is announced once.
    let saved = match draft.error() {
        Some(error) => format!("Not saved ({error:?}): kept until you leave"),
        None if draft.is_stored() => "Saved on this device".to_string(),
        None => String::new(),
    };

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            TextField {
                label: "Draft, kept after a reload",
                value: draft.get(),
                oninput: move |next| draft.set(next),
            }
            div { role: "status", "{saved}" }
            Flex { direction: "row", gap: "sm",
                Button { variant: "outlined", onclick: move |_| draft.remove(), "Forget draft" }
                Button {
                    variant: "outlined",
                    onclick: move |_| presses.update(|count| *count += 1),
                    "Count in this tab"
                }
            }
            Text { size: "sm", "Pressed {presses.get()} times in this tab" }
        }
    }
}
```

A desktop app built without libero's `desktop` feature names its directory
once, before the first storage hook runs:

```rust
fn main() {
    if let Some(data) = std::env::var_os("XDG_DATA_HOME") {
        libero::platform::set_storage_dir(std::path::Path::new(&data).join("my-app"));
    }
    // dioxus::launch(app);
}
```

## API

```rust,ignore
pub fn use_local_storage<T>(key: &str, default: impl FnOnce() -> T) -> Stored<T>
pub fn use_session_storage<T>(key: &str, default: impl FnOnce() -> T) -> Stored<T>
// T: Serialize + DeserializeOwned + Clone + PartialEq + 'static

pub enum StorageError { Unavailable, Full, Invalid }

pub fn libero::platform::set_storage_dir(dir: impl Into<PathBuf>)
```

| Method of `Stored<T>` | Description |
|---|---|
| `get()` | The stored value, or the default where none or no valid one is stored. |
| `value()` | The same as a `ReadSignal<T>`, to hand on, as to `use_debounced_value`. |
| `set(value)` | Stores `value`. Where the store refuses, it still holds for the session and `error()` says why. |
| `update(change)` | Changes the current value in place, then stores it. |
| `remove()` | Drops the stored value: `get()` falls back to the default. The default itself is never written. |
| `is_stored()` | Whether anything is stored under the key, valid or not. |
| `error()` | Why the last read or write failed, or `Invalid` while the stored text does not parse; the next good write clears it. |

| `StorageError` | When |
|---|---|
| `Unavailable` | No local store: site data blocked, no data directory, a build with only memory. Session storage off the web is memory by design and reports nothing. |
| `Full` | The browser's quota or the disk is full. |
| `Invalid` | The stored text does not parse as `T`; the text stays until the next `set`. |

`Stored<T>` is `Copy`. The key is read at mount: give the component a `key` to
switch it. A failure logs one warning per kind and document.

| Platform | Local | Session |
|---|---|---|
| Web | `localStorage`, other tabs follow live | `sessionStorage`, survives a reload in the tab |
| Blitz (`native`), desktop (`desktop`) | A file per key under the data directory's `<app>/storage/local` | Memory per window |
| Android | A file per key in the app's files directory | Memory per window |
| Server, liveview, other builds | Memory per document until `set_storage_dir` | Memory per document |

## Accessibility

### Libero handles

- It announces nothing and moves no focus: the value shows where you render it.
- A failed save never loses the value for the session; `error()` says why.

### You must

- Say what is kept and for how long next to the control, as the demo's label
  does.
- Offer a way to forget a kept value.
- Announce a failed save once in a status region where the user relies on it,
  as the demo does.
- Keep personal data out of local storage unless the user agreed: it stays on
  the device, readable by every script of the site.

### Example

A comment box keeps its draft: after a reload the text is back, a "Discard
draft" button forgets it, and a status line says "Draft not saved" once when
the store is full.

### Limits

- A kept value lives in one browser or app install: another device starts from
  the default.
