//! Values kept in IndexedDB, loaded and saved without blocking the page.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use dioxus::core::{provide_root_context, spawn_forever};
use dioxus::prelude::*;
use serde::{Serialize, de::DeserializeOwned};

use super::storage::{encode, parse, warn_once};
use crate::platform::{Op, StorageChange, StorageError, StorageSubscription, indexed_db};

/// One key's raw stored text, shared by every handle on it in this document.
#[derive(Clone, Copy)]
struct Entry {
    key: CopyValue<String>,
    raw: Signal<Option<String>>,
    error: Signal<Option<StorageError>>,
    loaded: Signal<bool>,
    /// Set by a write here or a change from another tab: a load landing later is stale.
    touched: CopyValue<bool>,
}

type EntryMap = RefCell<HashMap<String, Entry>>;
type Entries = Rc<EntryMap>;

/// Every key read in this document. Root context, not a `thread_local!`: a
/// liveview server runs many users' documents on one thread.
#[derive(Clone)]
struct IndexedDbHost {
    entries: Entries,
    warned: CopyValue<u8>,
    _changes: Rc<Option<Box<dyn StorageSubscription>>>,
}

fn indexed_db_host() -> IndexedDbHost {
    try_consume_context::<IndexedDbHost>().unwrap_or_else(|| {
        let entries = Entries::default();
        let weak = Rc::downgrade(&entries);
        let changes = indexed_db()
            .and_then(|store| store.on_change(Box::new(move |change| follow(&weak, change))));
        provide_root_context(IndexedDbHost {
            entries,
            warned: CopyValue::new_in_scope(0, ScopeId::ROOT),
            _changes: Rc::new(changes),
        })
    })
}

/// Another tab's write or remove lands in this document's entry for the key.
fn follow(entries: &Weak<EntryMap>, change: StorageChange) {
    let (Some(entries), Some(key)) = (entries.upgrade(), change.key) else {
        return;
    };
    let hit = entries.borrow().get(&key).copied();
    if let Some(mut entry) = hit {
        entry.touched.set(true);
        if *entry.raw.peek() != change.value {
            entry.raw.set(change.value);
        }
    }
}

impl IndexedDbHost {
    /// The key's entry, loading it from the store on first use.
    fn entry(&self, key: &str) -> Entry {
        if let Some(entry) = self.entries.borrow().get(key) {
            return *entry;
        }
        let entry = Entry {
            key: CopyValue::new_in_scope(key.to_string(), ScopeId::ROOT),
            raw: Signal::new_in_scope(None, ScopeId::ROOT),
            error: Signal::new_in_scope(None, ScopeId::ROOT),
            loaded: Signal::new_in_scope(false, ScopeId::ROOT),
            touched: CopyValue::new_in_scope(false, ScopeId::ROOT),
        };
        (self.entries.borrow_mut()).insert(key.to_string(), entry);
        // Without a store too, so a server render and the client's first render agree.
        let read: Op<Option<String>> = match indexed_db() {
            Some(store) => store.get(key),
            None => Box::pin(std::future::ready(Err(StorageError::Unavailable))),
        };
        load(read, entry, self.warned);
        entry
    }
}

fn load(read: Op<Option<String>>, mut entry: Entry, warned: CopyValue<u8>) {
    spawn_forever(async move {
        let read = read.await;
        if !*entry.touched.peek() {
            match read {
                Ok(raw) => entry.raw.set(raw),
                Err(error) => settle(entry, warned, Some(error)),
            }
        }
        entry.loaded.set(true);
    });
}

fn settle(mut entry: Entry, warned: CopyValue<u8>, error: Option<StorageError>) {
    if let Some(error) = error {
        warn_once(warned, error);
    }
    if *entry.error.peek() != error {
        entry.error.set(error);
    }
}

/// A value kept under a key in IndexedDB, from [`use_indexed_db`]. Every handle with
/// the same `T` on the same key in a document shows the same value.
///
/// The stored value arrives after the first render: until [`is_loaded`](Self::is_loaded)
/// is true, [`get`](Self::get) shows the default. A key is loaded once per document
/// and kept until the document ends, so avoid keys without bound (one per row).
///
/// Docs: <https://libero-ui.dev/hooks/use-indexed-db>
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::use_indexed_db;
/// # fn app() -> Element {
/// let mut draft = use_indexed_db("draft", String::new);
///
/// rsx! {
///     if draft.is_loaded() {
///         textarea { value: "{draft.get()}", oninput: move |event| draft.set(event.value()) }
///     } else {
///         p { role: "status", "Loading the draft" }
///     }
///     if let Some(error) = draft.error() {
///         p { role: "status", "Not saved: {error:?}" }
///     }
/// }
/// # }
/// ```
pub struct StoredAsync<T: 'static> {
    entry: Entry,
    parsed: Memo<Option<T>>,
    value: Memo<T>,
    fallback: CopyValue<T>,
    warned: CopyValue<u8>,
}

impl<T: 'static> Clone for StoredAsync<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for StoredAsync<T> {}

impl<T: Serialize + DeserializeOwned + Clone + PartialEq + 'static> StoredAsync<T> {
    /// The stored value, or the default while loading and where none (or no valid one)
    /// is stored. Reactive.
    pub fn get(&self) -> T {
        (self.value)()
    }

    /// The value as a signal, to hand on, as to [`use_debounced_value`](super::use_debounced_value).
    pub fn value(&self) -> ReadSignal<T> {
        self.value.into()
    }

    /// Stores `value`: [`get`](Self::get) shows it at once and the store follows later.
    /// A `set` before the load lands wins; the load is dropped. Where the store
    /// refuses, the value still holds for the session and [`error`](Self::error) says
    /// why. Each call writes: debounce a fast source. A value that does not read back
    /// (a NaN float) is not kept and reports `Invalid`.
    pub fn set(&mut self, value: T) {
        let Some(text) = encode(&value) else {
            settle(self.entry, self.warned, Some(StorageError::Invalid));
            return;
        };
        self.entry.touched.set(true);
        if self.entry.raw.peek().as_deref() != Some(text.as_str()) {
            self.entry.raw.set(Some(text.clone()));
        }
        self.save(Some(text));
    }

    /// Changes the current value in place, then stores it as [`set`](Self::set) does.
    /// Before the load lands it starts from the default; so it does while the stored
    /// text is [`Invalid`](StorageError::Invalid), and overwrites that text.
    pub fn update(&mut self, change: impl FnOnce(&mut T)) {
        // From the raw text: a memo stays stale until read, so a second update would lose the first.
        let parsed = parse(self.entry.raw.peek().as_deref());
        let mut value = parsed.unwrap_or_else(|| self.fallback.cloned());
        change(&mut value);
        self.set(value);
    }

    /// Drops the stored value: [`get`](Self::get) falls back to the default.
    pub fn remove(&mut self) {
        self.entry.touched.set(true);
        if self.entry.raw.peek().is_some() {
            self.entry.raw.set(None);
        }
        self.save(None);
    }

    /// Whether anything is stored under the key, valid or not. Reactive; false until loaded.
    pub fn is_stored(&self) -> bool {
        self.entry.raw.read().is_some()
    }

    /// Whether the first load has finished, with a value or without one. Reactive.
    pub fn is_loaded(&self) -> bool {
        (self.entry.loaded)()
    }

    /// Why the last load or save failed, or [`Invalid`](StorageError::Invalid)
    /// while the stored text does not parse; the next good save clears it. Reactive.
    pub fn error(&self) -> Option<StorageError> {
        if let Some(error) = (self.entry.error)() {
            return Some(error);
        }
        (self.is_stored() && self.parsed.read().is_none()).then_some(StorageError::Invalid)
    }

    /// Hands the text, or its removal, to the store; the answer lands in `error()`.
    fn save(&self, text: Option<String>) {
        let (entry, warned) = (self.entry, self.warned);
        let Some(store) = indexed_db() else {
            settle(entry, warned, Some(StorageError::Unavailable));
            return;
        };
        let key = entry.key.peek();
        let done = match &text {
            Some(text) => store.set(&key, text),
            None => store.remove(&key),
        };
        spawn_forever(async move { settle(entry, warned, done.await.err()) });
    }
}

/// A value kept across reloads and app runs in IndexedDB, which loads and saves
/// without blocking the page and holds far more than `localStorage`. Off the web a
/// file per key where libero keeps a directory (its `native` or `desktop` feature,
/// Android, or [`set_storage_dir`](crate::platform::set_storage_dir)), memory otherwise.
///
/// Docs: <https://libero-ui.dev/hooks/use-indexed-db>
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::use_indexed_db;
/// # fn app() -> Element {
/// let mut notes = use_indexed_db("notes", Vec::<String>::new);
///
/// rsx! {
///     button { onclick: move |_| notes.update(|notes| notes.push("A note".into())), "Add" }
///     p { "{notes.get().len()} notes" }
/// }
/// # }
/// ```
///
/// Stored as JSON. `default` shows until the load lands: see
/// [`is_loaded`](StoredAsync::is_loaded). `key` is read at mount: give the component
/// a `key` to switch it. On the web another tab's save arrives at once; off it,
/// other windows and processes read the file at their mount. Kept as plain text that
/// anyone with the browser profile or the file can read: not for secrets.
pub fn use_indexed_db<T: Serialize + DeserializeOwned + Clone + PartialEq + 'static>(
    key: &str,
    default: impl FnOnce() -> T,
) -> StoredAsync<T> {
    let (entry, warned, fallback) = use_hook(|| {
        let host = indexed_db_host();
        (host.entry(key), host.warned, CopyValue::new(default()))
    });
    let raw = entry.raw;
    let parsed = use_memo(move || {
        let parsed = (raw.read().as_deref()).map(serde_json::from_str::<T>);
        if let Some(Err(_)) = parsed {
            warn_once(warned, StorageError::Invalid);
        }
        parsed.and_then(Result::ok)
    });
    // Straight from `raw`, not `parsed`: a read right after a write then recomputes it.
    let value = use_memo(move || parse(raw.read().as_deref()).unwrap_or_else(|| fallback.cloned()));
    StoredAsync {
        entry,
        parsed,
        value,
        fallback,
        warned,
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "indexed_db_tests.rs"]
mod tests;
