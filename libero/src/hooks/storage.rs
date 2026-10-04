//! Values kept under a key past a reload: local storage across runs, session
//! storage for the tab, or the app run off the web.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use dioxus::core::provide_root_context;
use dioxus::prelude::*;
use serde::{Serialize, de::DeserializeOwned};

use crate::platform::{StorageArea, StorageChange, StorageError, StorageSubscription, storage};

/// One key's raw stored text, shared by every handle on it in this document.
#[derive(Clone, Copy)]
struct Entry {
    area: StorageArea,
    key: CopyValue<String>,
    raw: Signal<Option<String>>,
    error: Signal<Option<StorageError>>,
}

type EntryMap = RefCell<HashMap<(StorageArea, String), Entry>>;
type Entries = Rc<EntryMap>;

/// Every key read in this document. Root context, not a `thread_local!`: a
/// liveview server runs many users' documents on one thread.
#[derive(Clone)]
struct StorageHost {
    entries: Entries,
    warned: CopyValue<u8>,
    _changes: Rc<Vec<Box<dyn StorageSubscription>>>,
}

fn storage_host() -> StorageHost {
    try_consume_context::<StorageHost>().unwrap_or_else(|| {
        let entries = Entries::default();
        let changes = [StorageArea::Local, StorageArea::Session]
            .into_iter()
            .filter_map(|area| {
                let entries = Rc::downgrade(&entries);
                storage(area)?.on_change(Box::new(move |change| follow(&entries, area, change)))
            })
            .collect();
        provide_root_context(StorageHost {
            entries,
            warned: CopyValue::new_in_scope(0, ScopeId::ROOT),
            _changes: Rc::new(changes),
        })
    })
}

/// Another tab's write lands in this document's entries; `key: None` was a `clear()`.
fn follow(entries: &Weak<EntryMap>, area: StorageArea, change: StorageChange) {
    let Some(entries) = entries.upgrade() else {
        return;
    };
    let hits: Vec<Entry> = match &change.key {
        Some(key) => (entries.borrow().get(&(area, key.clone())))
            .copied()
            .into_iter()
            .collect(),
        None => (entries.borrow().values())
            .filter(|entry| entry.area == area)
            .copied()
            .collect(),
    };
    for mut entry in hits {
        if *entry.raw.peek() != change.value {
            entry.raw.set(change.value.clone());
        }
    }
}

impl StorageHost {
    /// The key's entry, read from the store on first use.
    fn entry(&self, area: StorageArea, key: &str) -> Entry {
        if let Some(entry) = self.entries.borrow().get(&(area, key.to_string())) {
            return *entry;
        }
        let (raw, error) = match storage(area).map(|store| store.get(key)) {
            Some(Ok(raw)) => (raw, None),
            Some(Err(error)) => (None, Some(error)),
            None => (None, missing(area)),
        };
        if let Some(error) = error {
            warn_once(self.warned, error);
        }
        let entry = Entry {
            area,
            key: CopyValue::new_in_scope(key.to_string(), ScopeId::ROOT),
            raw: Signal::new_in_scope(raw, ScopeId::ROOT),
            error: Signal::new_in_scope(error, ScopeId::ROOT),
        };
        (self.entries.borrow_mut()).insert((area, key.to_string()), entry);
        entry
    }
}

/// Session storage off the web lives in memory by design; local storage there failed to persist.
fn missing(area: StorageArea) -> Option<StorageError> {
    (area == StorageArea::Local).then_some(StorageError::Unavailable)
}

/// Once per kind and document: a failing store fails on every write.
fn warn_once(mut warned: CopyValue<u8>, error: StorageError) {
    let bit = 1 << error as u8;
    if *warned.peek() & bit != 0 {
        return;
    }
    *warned.write() |= bit;
    crate::utils::warn(match error {
        StorageError::Unavailable => "storage: nothing to keep values in, they last the session",
        StorageError::Full => "storage: the store is full, values last the session",
        StorageError::Invalid => "storage: a stored value does not parse, the default shows",
    });
}

/// A value kept under a key, from [`use_local_storage`] or [`use_session_storage`].
/// Every handle on the same key in a document shows the same value.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::use_local_storage;
/// # fn app() -> Element {
/// let mut count = use_local_storage("visits", || 0_u32);
///
/// rsx! {
///     button { onclick: move |_| count.update(|count| *count += 1), "Visited {count.get()} times" }
///     button { onclick: move |_| count.remove(), "Reset" }
///     if let Some(error) = count.error() {
///         p { role: "status", "Not saved: {error:?}" }
///     }
/// }
/// # }
/// ```
pub struct Stored<T: 'static> {
    entry: Entry,
    warned: CopyValue<u8>,
    parsed: Memo<Option<T>>,
    value: Memo<T>,
}

impl<T: 'static> Clone for Stored<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for Stored<T> {}

impl<T: Serialize + DeserializeOwned + Clone + PartialEq + 'static> Stored<T> {
    /// The stored value, or the default where none (or no valid one) is stored. Reactive.
    pub fn get(&self) -> T {
        (self.value)()
    }

    /// The value as a signal, to hand on, as to [`use_debounced_value`](super::use_debounced_value).
    pub fn value(&self) -> ReadSignal<T> {
        self.value.into()
    }

    /// Stores `value`. Where the store refuses, it still holds for the session
    /// and [`error`](Self::error) says why. Each call writes: debounce a fast source.
    pub fn set(&mut self, value: T) {
        let Ok(text) = serde_json::to_string(&value) else {
            self.settle(Some(Err(StorageError::Invalid)));
            return;
        };
        let written =
            storage(self.entry.area).map(|store| store.set(&self.entry.key.peek(), &text));
        self.settle(written);
        if self.entry.raw.peek().as_deref() != Some(text.as_str()) {
            self.entry.raw.set(Some(text));
        }
    }

    /// Changes the current value in place, then stores it as [`set`](Self::set) does.
    pub fn update(&mut self, change: impl FnOnce(&mut T)) {
        let mut value = self.value.peek().clone();
        change(&mut value);
        self.set(value);
    }

    /// Drops the stored value: [`get`](Self::get) falls back to the default.
    pub fn remove(&mut self) {
        let removed = storage(self.entry.area).map(|store| store.remove(&self.entry.key.peek()));
        self.settle(removed);
        if self.entry.raw.peek().is_some() {
            self.entry.raw.set(None);
        }
    }

    /// Whether anything is stored under the key, valid or not. Reactive.
    pub fn is_stored(&self) -> bool {
        self.entry.raw.read().is_some()
    }

    /// Why the last read or write failed, or [`Invalid`](StorageError::Invalid)
    /// while the stored text does not parse; the next good write clears it. Reactive.
    pub fn error(&self) -> Option<StorageError> {
        if let Some(error) = (self.entry.error)() {
            return Some(error);
        }
        (self.is_stored() && self.parsed.read().is_none()).then_some(StorageError::Invalid)
    }

    fn settle(&mut self, done: Option<Result<(), StorageError>>) {
        let error = match done {
            Some(done) => done.err(),
            None => missing(self.entry.area),
        };
        if let Some(error) = error {
            warn_once(self.warned, error);
        }
        if *self.entry.error.peek() != error {
            self.entry.error.set(error);
        }
    }
}

/// A value kept across reloads and app runs: `localStorage` on the web, a file
/// per key elsewhere (see [`set_storage_dir`](crate::platform::set_storage_dir)).
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::use_local_storage;
/// # fn app() -> Element {
/// let mut name = use_local_storage("user-name", String::new);
///
/// rsx! {
///     input { value: "{name.get()}", oninput: move |event| name.set(event.value()) }
/// }
/// # }
/// ```
///
/// Stored as JSON. Read at the first render, so a hydrating server render shows
/// `default` where the client then shows the stored value. `key` is read at
/// mount: give the component a `key` to switch it. Another tab's write arrives
/// on the web; other windows and processes off the web read it at their mount.
pub fn use_local_storage<T: Serialize + DeserializeOwned + Clone + PartialEq + 'static>(
    key: &str,
    default: impl FnOnce() -> T,
) -> Stored<T> {
    use_storage(StorageArea::Local, key, default)
}

/// A value kept for the session: `sessionStorage` on the web, so a reload keeps
/// it in the tab; memory for the window's run elsewhere.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::use_session_storage;
/// # fn app() -> Element {
/// let mut step = use_session_storage("checkout-step", || 1_u8);
///
/// rsx! {
///     button { onclick: move |_| step.update(|step| *step += 1), "Step {step.get()}: next" }
/// }
/// # }
/// ```
///
/// Behaves as [`use_local_storage`] otherwise.
pub fn use_session_storage<T: Serialize + DeserializeOwned + Clone + PartialEq + 'static>(
    key: &str,
    default: impl FnOnce() -> T,
) -> Stored<T> {
    use_storage(StorageArea::Session, key, default)
}

fn use_storage<T: Serialize + DeserializeOwned + Clone + PartialEq + 'static>(
    area: StorageArea,
    key: &str,
    default: impl FnOnce() -> T,
) -> Stored<T> {
    let (entry, warned, fallback) = use_hook(|| {
        let host = storage_host();
        (
            host.entry(area, key),
            host.warned,
            CopyValue::new(default()),
        )
    });
    let raw = entry.raw;
    let parsed = use_memo(move || {
        let parsed = (raw.read().as_deref()).map(serde_json::from_str::<T>);
        if let Some(Err(_)) = parsed {
            warn_once(warned, StorageError::Invalid);
        }
        parsed.and_then(Result::ok)
    });
    let value = use_memo(move || parsed().unwrap_or_else(|| fallback.cloned()));
    Stored {
        entry,
        warned,
        parsed,
        value,
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "storage_tests.rs"]
mod tests;
