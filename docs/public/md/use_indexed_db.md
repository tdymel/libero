# IndexedDB

Crate: `libero`
Import: `use libero::hooks::{StorageError, StoredAsync, use_indexed_db};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/indexed_db.rs>
Index: [index.md](index.md) lists every other page
Description: A larger value kept under a key in IndexedDB, loaded and saved without blocking the page; a file per key off the web.

`use_indexed_db(key, default) -> StoredAsync<T>` keeps a value across reloads
and app runs like `use_local_storage`, but loads and saves without blocking the
page. Pick it over `use_local_storage` for values of more than a few kilobytes,
such as a document, a draft or a list of rows: `localStorage` reads and writes
on the main thread and holds about 5 MB, IndexedDB holds far more. For a small
setting read at the first render, `use_local_storage` shows the stored value at
once.

The value arrives after the first render. Until `is_loaded()` is true, `get()`
shows the default. A `set`, `update` or `remove` before the load lands wins and
the stored value is dropped. Writes show at once and are saved afterwards;
`error()` reports a save that failed. `get()`, `is_stored()`, `is_loaded()` and
`error()` are reactive, and every handle of the same type on a key shows the
same value.

Web: one IndexedDB database, `libero`, with one store. A write or remove in one
tab shows in the origin's other tabs at once. Blitz (the `native` feature), a
desktop app with the `desktop` feature and Android: a file per key in the app's
data directory, next to `use_local_storage`'s. Any other build keeps values in
memory until you call `libero::platform::set_storage_dir(path)`. Blocked
IndexedDB, as in some private modes, reports `Unavailable` and the value lasts
the session.

Values are stored as JSON, so a `String` is kept with its quotes; binary data is
not supported. Keys are used verbatim. Each write hits the store, so pass a fast
source such as a slider through `use_debounced_value` first.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, TextField},
    hooks::use_indexed_db,
};

#[component]
fn KeptNotes() -> Element {
    let mut notes = use_indexed_db("notes", String::new);
    // Rendered from the start, so a failed save is announced once.
    let status = match notes.error() {
        _ if !notes.is_loaded() => "Loading your notes".to_string(),
        Some(error) => format!("Not saved ({error:?}): kept until you leave"),
        None if notes.is_stored() => "Saved on this device".to_string(),
        None => String::new(),
    };

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            TextField {
                label: "Notes, kept after a reload",
                value: notes.get(),
                disabled: !notes.is_loaded(),
                oninput: move |next| notes.set(next),
            }
            div { role: "status", "{status}" }
            Button { variant: "outlined", onclick: move |_| notes.remove(), "Forget notes" }
        }
    }
}
```

## API

```rust,ignore
pub fn use_indexed_db<T>(key: &str, default: impl FnOnce() -> T) -> StoredAsync<T>
// T: Serialize + DeserializeOwned + Clone + PartialEq + 'static

pub enum StorageError { Unavailable, Full, Invalid }
```

| Method of `StoredAsync<T>` | Description |
|---|---|
| `get()` | The stored value, or the default while loading and where none or no valid one is stored. |
| `value()` | The same as a `ReadSignal<T>`, to hand on, as to `use_debounced_value`. |
| `set(value)` | Stores `value`: `get()` shows it at once and the store follows later. A `set` before the load lands wins. Where the store refuses, it still holds for the session and `error()` says why. A value that does not read back (a NaN float) is not kept and reports `Invalid`. |
| `update(change)` | Changes the current value in place, then stores it. Before the load lands it starts from the default; so it does while the stored text is `Invalid`, and overwrites that text. |
| `remove()` | Drops the stored value: `get()` falls back to the default. The default itself is never written. |
| `is_stored()` | Whether anything is stored under the key, valid or not; false until loaded. |
| `is_loaded()` | Whether the first load has finished, with a value or without one. |
| `error()` | Why the last load or save failed, or `Invalid` while the stored text does not parse; the next good save clears it. |

| `StorageError` | When |
|---|---|
| `Unavailable` | No store: IndexedDB blocked, no data directory, a build with only memory. |
| `Full` | The browser's quota or the disk is full. |
| `Invalid` | The stored text does not parse as `T`; the text stays until the next `set`. Or `set` got a value that does not read back. |

`StoredAsync<T>` is `Copy`. The key is read at mount: give the component a
`key` to switch it. A failure logs one warning per kind and document. A key is
loaded once per document and kept until it ends, so avoid keys without bound;
off the web a write from outside the document (another window, a second
process) shows in the next one. Values are plain text: not for secrets.

| Platform | Store |
|---|---|
| Web | Database `libero`, object store `kv`; other tabs follow a write or remove through a `BroadcastChannel` named `libero-indexed-db` |
| Blitz (`native`), desktop (`desktop`) | A file per key under the data directory's `<app>/storage/indexed` |
| Android | A file per key in the app's files directory, under `storage/indexed` |
| Server, liveview, other builds | Memory per document until `set_storage_dir` |

## Accessibility

### Libero handles

- It announces nothing and moves no focus: the value shows where you render it.
- A failed save never loses the value for the session; `error()` says why.

### You must

- Show a loading state until `is_loaded()` is true, or disable the control: the
  default shows first, and typing over it before the load lands replaces the
  stored value.
- Say what is kept and for how long next to the control, as the demo's label
  does.
- Offer a way to forget a kept value.
- Announce a failed save once in a status region where the user relies on it,
  as the demo does.
- Keep personal data out of IndexedDB unless the user agreed: it stays on the
  device, readable by every script of the site.

### Example

A notes box keeps its text: while it loads the field is disabled and a status
line says "Loading your notes", after a reload the text is back, and a "Forget
notes" button clears it.

### Limits

- A kept value lives in one browser or app install: another device starts from
  the default.
