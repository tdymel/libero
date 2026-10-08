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
    /// Whether the store was read; a handle reading after mount creates it unread.
    loaded: CopyValue<bool>,
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
    /// The key's entry; `read` reads the store now unless an earlier handle did.
    fn entry(&self, area: StorageArea, key: &str, read: bool) -> Entry {
        if let Some(entry) = self.entries.borrow().get(&(area, key.to_string())).copied() {
            if read {
                load(entry, self.warned);
            }
            return entry;
        }
        let (raw, error) = match read {
            true => read_store(area, key, self.warned),
            false => (None, None),
        };
        let entry = Entry {
            area,
            key: CopyValue::new_in_scope(key.to_string(), ScopeId::ROOT),
            raw: Signal::new_in_scope(raw, ScopeId::ROOT),
            error: Signal::new_in_scope(error, ScopeId::ROOT),
            loaded: CopyValue::new_in_scope(read, ScopeId::ROOT),
        };
        (self.entries.borrow_mut()).insert((area, key.to_string()), entry);
        entry
    }
}

fn read_store(
    area: StorageArea,
    key: &str,
    warned: CopyValue<u8>,
) -> (Option<String>, Option<StorageError>) {
    let (raw, error) = match storage(area).map(|store| store.get(key)) {
        Some(Ok(raw)) => (raw, None),
        Some(Err(error)) => (None, Some(error)),
        None => (None, missing(area)),
    };
    if let Some(error) = error {
        warn_once(warned, error);
    }
    (raw, error)
}

/// Reads an entry created unread; a no-op once read or written.
fn load(mut entry: Entry, warned: CopyValue<u8>) {
    if *entry.loaded.peek() {
        return;
    }
    entry.loaded.set(true);
    let (raw, error) = read_store(entry.area, &entry.key.peek(), warned);
    if *entry.raw.peek() != raw {
        entry.raw.set(raw);
    }
    if *entry.error.peek() != error {
        entry.error.set(error);
    }
}

/// Session storage off the web lives in memory by design; local storage there failed to persist.
fn missing(area: StorageArea) -> Option<StorageError> {
    (area == StorageArea::Local).then_some(StorageError::Unavailable)
}

/// Once per kind and document: a failing store fails on every write.
pub(super) fn warn_once(mut warned: CopyValue<u8>, error: StorageError) {
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
/// Every handle with the same `T` on the same key in a document shows the same value.
///
/// A key's text is read once per document and kept until the document ends, so
/// avoid keys without bound (one per row). A write to the store from outside the
/// document (another window, page script, a second root) shows in the next
/// document; on the web another tab's write arrives at once.
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
    format: StorageFormat,
    /// False until the read after mount, with [`StorageOptions::read_after_mount`].
    mounted: Signal<bool>,
    parsed: Memo<Option<T>>,
    value: Memo<T>,
    fallback: CopyValue<T>,
}

/// How [`use_local_storage_with`] and [`use_session_storage_with`] read and store.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::{StorageOptions, use_local_storage_with};
/// # fn app() -> Element {
/// let theme = use_local_storage_with("theme", || "light".to_string(), StorageOptions {
///     read_after_mount: true,
///     ..Default::default()
/// });
/// # rsx! {}
/// # }
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StorageOptions {
    /// Show `default` at the first render and read the store after mount, so a
    /// hydrating server render and the client's first render agree.
    pub read_after_mount: bool,
    /// How the value is written as text.
    pub format: StorageFormat,
}

/// The text a value is stored as.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StorageFormat {
    /// JSON: a `String` keeps its quotes.
    #[default]
    Json,
    /// The bare text of a value that serialises to a string (a `String`, or an
    /// enum of unit variants), as libero's own `lsx-` keys and page scripts keep it.
    /// Any other value is not kept and reports [`Invalid`](StorageError::Invalid).
    Text,
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
    /// A value that does not read back (a NaN float) is not kept and reports `Invalid`.
    pub fn set(&mut self, value: T) {
        self.mount();
        let Some(text) = encode_as(self.format, &value) else {
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
    /// While the stored text is [`Invalid`](StorageError::Invalid) it starts from the
    /// default and overwrites that text.
    pub fn update(&mut self, change: impl FnOnce(&mut T)) {
        self.mount();
        // From the raw text: a memo stays stale until read, so a second update would lose the first.
        let parsed = parse_as(self.format, self.entry.raw.peek().as_deref());
        let mut value = parsed.unwrap_or_else(|| self.fallback.cloned());
        change(&mut value);
        self.set(value);
    }

    /// Drops the stored value: [`get`](Self::get) falls back to the default.
    pub fn remove(&mut self) {
        self.mount();
        let removed = storage(self.entry.area).map(|store| store.remove(&self.entry.key.peek()));
        self.settle(removed);
        if self.entry.raw.peek().is_some() {
            self.entry.raw.set(None);
        }
    }

    /// Whether anything is stored under the key, valid or not. Reactive.
    pub fn is_stored(&self) -> bool {
        (self.mounted)() && self.entry.raw.read().is_some()
    }

    /// Whether the store was read: from the first render, or after mount with
    /// [`StorageOptions::read_after_mount`]. Reactive.
    pub fn is_loaded(&self) -> bool {
        (self.mounted)()
    }

    /// Why the last read or write failed, or [`Invalid`](StorageError::Invalid)
    /// while the stored text does not parse; the next good write clears it.
    /// `None` until loaded. Reactive.
    pub fn error(&self) -> Option<StorageError> {
        if !(self.mounted)() {
            return None;
        }
        if let Some(error) = (self.entry.error)() {
            return Some(error);
        }
        (self.is_stored() && self.parsed.read().is_none()).then_some(StorageError::Invalid)
    }

    /// Reads the store if this handle has not yet: a write starts from what is stored.
    fn mount(&mut self) {
        if !*self.mounted.peek() {
            load(self.entry, self.warned);
            self.mounted.set(true);
        }
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
/// per key where libero keeps a directory (its `native` or `desktop` feature,
/// Android, or [`set_storage_dir`](crate::platform::set_storage_dir)), memory otherwise.
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
/// Stored as JSON. Read at the first render: a hydrating fullstack app takes
/// [`use_local_storage_with`] and [`StorageOptions::read_after_mount`]. `key` is
/// read at mount: give the component a `key` to switch it. Another tab's write
/// arrives on the web; other windows and processes off the web read it at their
/// mount. Kept as plain text that anyone with the browser profile or the file can
/// read: not for secrets.
pub fn use_local_storage<T: Serialize + DeserializeOwned + Clone + PartialEq + 'static>(
    key: &str,
    default: impl FnOnce() -> T,
) -> Stored<T> {
    use_storage(StorageArea::Local, key, default, StorageOptions::default())
}

/// [`use_local_storage`] with [`StorageOptions`]: read after mount for a hydrating
/// server render, or keep bare text as libero's own `lsx-` keys do.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::{StorageFormat, StorageOptions, use_local_storage_with};
/// # fn app() -> Element {
/// // The colour scheme libero keeps: "light" or "dark", read at its next start.
/// let scheme = use_local_storage_with("lsx-color-scheme", String::new, StorageOptions {
///     read_after_mount: true,
///     format: StorageFormat::Text,
/// });
///
/// rsx! {
///     if scheme.is_loaded() {
///         p { "Kept scheme: {scheme.get()}" }
///     }
/// }
/// # }
/// ```
pub fn use_local_storage_with<T: Serialize + DeserializeOwned + Clone + PartialEq + 'static>(
    key: &str,
    default: impl FnOnce() -> T,
    options: StorageOptions,
) -> Stored<T> {
    use_storage(StorageArea::Local, key, default, options)
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
    use_storage(
        StorageArea::Session,
        key,
        default,
        StorageOptions::default(),
    )
}

/// [`use_session_storage`] with [`StorageOptions`], as [`use_local_storage_with`].
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::{StorageOptions, use_session_storage_with};
/// # fn app() -> Element {
/// let tab = use_session_storage_with("open-tab", || 0_usize, StorageOptions {
///     read_after_mount: true,
///     ..Default::default()
/// });
/// # rsx! {}
/// # }
/// ```
pub fn use_session_storage_with<T: Serialize + DeserializeOwned + Clone + PartialEq + 'static>(
    key: &str,
    default: impl FnOnce() -> T,
    options: StorageOptions,
) -> Stored<T> {
    use_storage(StorageArea::Session, key, default, options)
}

/// Raw text kept for the session under a key, for libero's own caches. Reactive;
/// off the web it lives in the document's memory.
#[derive(Clone, Copy)]
pub(crate) struct SessionText(Entry);

/// The document's session text under `key`, read from the store on first use.
pub(crate) fn session_text(key: &str) -> SessionText {
    SessionText(storage_host().entry(StorageArea::Session, key, true))
}

impl SessionText {
    pub(crate) fn get(&self) -> Option<String> {
        self.0.raw.cloned()
    }

    /// Stores `text`; a full or refused store keeps it in memory only.
    pub(crate) fn set(mut self, text: String) {
        if let Some(store) = storage(StorageArea::Session) {
            let _ = store.set(&self.0.key.peek(), &text);
        }
        self.0.raw.set(Some(text));
    }
}

fn use_storage<T: Serialize + DeserializeOwned + Clone + PartialEq + 'static>(
    area: StorageArea,
    key: &str,
    default: impl FnOnce() -> T,
    options: StorageOptions,
) -> Stored<T> {
    let StorageOptions {
        read_after_mount,
        format,
    } = use_hook(|| options);
    let (entry, warned, fallback) = use_hook(|| {
        let host = storage_host();
        (
            host.entry(area, key, !read_after_mount),
            host.warned,
            CopyValue::new(default()),
        )
    });
    let mut mounted = use_signal(|| !read_after_mount);
    // Effects never run in a server render, so it and the hydrating client show the default.
    use_effect(move || {
        if !*mounted.peek() {
            load(entry, warned);
            mounted.set(true);
        }
    });
    let raw = entry.raw;
    let parsed = use_memo(move || {
        let raw = raw.read();
        let parsed = raw.as_deref().map(|raw| parse_as::<T>(format, Some(raw)));
        if let Some(None) = parsed {
            warn_once(warned, StorageError::Invalid);
        }
        parsed.flatten()
    });
    // Straight from `raw`, not `parsed`: a read right after a write then recomputes it.
    let value = use_memo(move || {
        let shown = mounted().then(|| parse_as(format, raw.read().as_deref()));
        shown.flatten().unwrap_or_else(|| fallback.cloned())
    });
    Stored {
        entry,
        warned,
        format,
        mounted,
        parsed,
        value,
        fallback,
    }
}

pub(super) fn parse<T: DeserializeOwned>(raw: Option<&str>) -> Option<T> {
    serde_json::from_str(raw?).ok()
}

fn parse_as<T: DeserializeOwned>(format: StorageFormat, raw: Option<&str>) -> Option<T> {
    match format {
        StorageFormat::Json => parse(raw),
        StorageFormat::Text => serde_json::from_value(serde_json::Value::String(raw?.into())).ok(),
    }
}

fn encode_as<T: Serialize + DeserializeOwned>(format: StorageFormat, value: &T) -> Option<String> {
    match format {
        StorageFormat::Json => encode(value),
        StorageFormat::Text => match serde_json::to_value(value).ok()? {
            serde_json::Value::String(text) => parse_as::<T>(format, Some(&text)).map(|_| text),
            _ => None,
        },
    }
}

/// `value` as JSON text that reads back as a `T`. serde_json writes a NaN float as `null`.
pub(super) fn encode<T: Serialize + DeserializeOwned>(value: &T) -> Option<String> {
    serde_json::to_string(value)
        .ok()
        .filter(|text| !text.contains("null") || serde_json::from_str::<T>(text).is_ok())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "storage_tests.rs"]
mod tests;
