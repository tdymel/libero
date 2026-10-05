//! Text kept under a key: `localStorage` and `sessionStorage` on the web, one file
//! per key for local storage off it. Serialising is the hook's job.

use std::path::PathBuf;

/// Which store a key lives in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum StorageArea {
    Local,
    Session,
}

/// Why a value could not be read or kept. It still holds in memory for the session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageError {
    /// Nothing to keep it in: blocked site data, no app directory, a server build.
    Unavailable,
    /// The store is full: the browser's quota, or a full disk.
    Full,
    /// The stored text does not parse as the value's type, or the value does not serialise.
    Invalid,
}

/// A key changed outside this document, as by another tab. `key: None` cleared them all.
pub(crate) struct StorageChange {
    pub(crate) key: Option<String>,
    pub(crate) value: Option<String>,
}

/// Dropping it stops the [`StorageApi::on_change`] callback.
pub(crate) trait StorageSubscription {}

/// One area's store of raw text.
pub(crate) trait StorageApi {
    fn get(&self, key: &str) -> Result<Option<String>, StorageError>;
    fn set(&self, key: &str, value: &str) -> Result<(), StorageError>;
    fn remove(&self, key: &str) -> Result<(), StorageError>;

    /// Changes made outside this document; `None` where none can arrive.
    fn on_change(
        &self,
        _callback: Box<dyn Fn(StorageChange)>,
    ) -> Option<Box<dyn StorageSubscription>> {
        None
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use wasm_bindgen::{JsCast, JsValue, closure::Closure};
    use web_sys::{Storage, StorageEvent, Window};

    use super::{StorageApi, StorageArea, StorageChange, StorageError, StorageSubscription};

    pub(super) struct WebStorage(StorageArea);

    pub(super) static LOCAL: WebStorage = WebStorage(StorageArea::Local);
    pub(super) static SESSION: WebStorage = WebStorage(StorageArea::Session);

    impl WebStorage {
        fn storage(&self) -> Result<Storage, StorageError> {
            let window = web_sys::window().ok_or(StorageError::Unavailable)?;
            let storage = match self.0 {
                StorageArea::Local => window.local_storage(),
                StorageArea::Session => window.session_storage(),
            };
            // Blocked site data throws on access.
            storage.ok().flatten().ok_or(StorageError::Unavailable)
        }
    }

    impl StorageApi for WebStorage {
        fn get(&self, key: &str) -> Result<Option<String>, StorageError> {
            (self.storage()?.get_item(key)).map_err(|_| StorageError::Unavailable)
        }

        fn set(&self, key: &str, value: &str) -> Result<(), StorageError> {
            (self.storage()?.set_item(key, value)).map_err(|error| refusal(&error))
        }

        fn remove(&self, key: &str) -> Result<(), StorageError> {
            (self.storage()?.remove_item(key)).map_err(|_| StorageError::Unavailable)
        }

        // The `storage` event fires only in the origin's other documents.
        fn on_change(
            &self,
            callback: Box<dyn Fn(StorageChange)>,
        ) -> Option<Box<dyn StorageSubscription>> {
            let window = web_sys::window()?;
            let own = self.storage().ok()?;
            let closure = Closure::<dyn FnMut(StorageEvent)>::new(move |event: StorageEvent| {
                if event.storage_area().as_ref() == Some(&own) {
                    callback(StorageChange {
                        key: event.key(),
                        value: event.new_value(),
                    });
                }
            });
            (window.add_event_listener_with_callback("storage", closure.as_ref().unchecked_ref()))
                .ok()?;
            Some(Box::new(WebStorageSubscription { window, closure }))
        }
    }

    fn refusal(error: &JsValue) -> StorageError {
        let name = js_sys::Reflect::get(error, &"name".into()).ok();
        match name.and_then(|name| name.as_string()).as_deref() {
            Some("QuotaExceededError") => StorageError::Full,
            _ => StorageError::Unavailable,
        }
    }

    struct WebStorageSubscription {
        window: Window,
        closure: Closure<dyn FnMut(StorageEvent)>,
    }

    impl StorageSubscription for WebStorageSubscription {}

    impl Drop for WebStorageSubscription {
        fn drop(&mut self) {
            let _ = self.window.remove_event_listener_with_callback(
                "storage",
                self.closure.as_ref().unchecked_ref(),
            );
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod files {
    use std::fmt::Write as _;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Mutex, OnceLock, PoisonError};

    use super::{StorageApi, StorageError};

    /// Set by [`set_storage_dir`](super::set_storage_dir); wins over the app's own directory.
    pub(super) static DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

    pub(super) struct FileStorage;

    pub(super) static FILES: FileStorage = FileStorage;

    pub(super) fn local_dir() -> Option<PathBuf> {
        let set = DIR.lock().unwrap_or_else(PoisonError::into_inner).clone();
        set.or_else(|| {
            static OWN: OnceLock<Option<PathBuf>> = OnceLock::new();
            OWN.get_or_init(app_storage_dir).clone()
        })
    }

    #[cfg(target_os = "android")]
    fn app_storage_dir() -> Option<PathBuf> {
        Some(super::app_dir()?.join("storage").join("local"))
    }

    #[cfg(all(
        not(target_os = "android"),
        any(feature = "native", feature = "desktop")
    ))]
    fn app_storage_dir() -> Option<PathBuf> {
        let exe = std::env::current_exe().ok()?;
        let app = exe.file_stem()?;
        Some(
            dirs::data_local_dir()?
                .join(app)
                .join("storage")
                .join("local"),
        )
    }

    /// A server build shares this cfg: one file would be shared by every user's document.
    #[cfg(all(
        not(target_os = "android"),
        not(feature = "native"),
        not(feature = "desktop")
    ))]
    fn app_storage_dir() -> Option<PathBuf> {
        None
    }

    impl StorageApi for FileStorage {
        fn get(&self, key: &str) -> Result<Option<String>, StorageError> {
            read(&local_dir().ok_or(StorageError::Unavailable)?, key)
        }

        fn set(&self, key: &str, value: &str) -> Result<(), StorageError> {
            write(&local_dir().ok_or(StorageError::Unavailable)?, key, value)
        }

        fn remove(&self, key: &str) -> Result<(), StorageError> {
            delete(&local_dir().ok_or(StorageError::Unavailable)?, key)
        }
    }

    pub(super) fn read(dir: &Path, key: &str) -> Result<Option<String>, StorageError> {
        match std::fs::read(dir.join(file_name(key))) {
            Ok(bytes) => String::from_utf8(bytes)
                .map(Some)
                .map_err(|_| StorageError::Invalid),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(refusal(&error)),
        }
    }

    /// Through a temporary file and a rename, so a crash never leaves half a value.
    pub(super) fn write(dir: &Path, key: &str, value: &str) -> Result<(), StorageError> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let name = file_name(key);
        let temp = dir.join(format!(
            "{name}.{}-{}.tmp",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let written = std::fs::create_dir_all(dir)
            .and_then(|()| std::fs::write(&temp, value))
            .and_then(|()| std::fs::rename(&temp, dir.join(name)));
        written.map_err(|error| {
            let _ = std::fs::remove_file(&temp);
            refusal(&error)
        })
    }

    pub(super) fn delete(dir: &Path, key: &str) -> Result<(), StorageError> {
        match std::fs::remove_file(dir.join(file_name(key))) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => Err(refusal(&error)),
            _ => Ok(()),
        }
    }

    fn refusal(error: &io::Error) -> StorageError {
        match error.kind() {
            io::ErrorKind::StorageFull | io::ErrorKind::QuotaExceeded => StorageError::Full,
            _ => StorageError::Unavailable,
        }
    }

    /// Names Windows reserves whatever the extension.
    const RESERVED: [&str; 22] = [
        "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
        "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
    ];

    /// `key` as a file name on any file system: `[a-z0-9_-]` stay, every other byte
    /// becomes `%XX`, so `/`, `..` and case-folding cannot make two keys meet.
    pub(super) fn file_name(key: &str) -> String {
        let mut name = String::with_capacity(key.len() + 5);
        for (index, byte) in key.bytes().enumerate() {
            let plain = matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-');
            if plain && !(index == 0 && RESERVED.contains(&key)) {
                name.push(byte as char);
            } else {
                let _ = write!(name, "%{byte:02X}");
            }
        }
        name + ".json"
    }
}

#[cfg(test)]
type FakeStores = [Option<&'static dyn StorageApi>; 2];

#[cfg(test)]
thread_local! {
    static FAKE: std::cell::Cell<Option<FakeStores>> = const { std::cell::Cell::new(None) };
}

/// Puts back the platform's stores when dropped.
#[cfg(test)]
pub(crate) struct FakeStorageGuard;

#[cfg(test)]
impl Drop for FakeStorageGuard {
    fn drop(&mut self) {
        FAKE.with(|fake| fake.set(None));
    }
}

/// Answers `storage()` with `local` and `session` on this thread until the guard
/// drops; `None` stands for no store at all.
#[cfg(test)]
pub(crate) fn fake_storage(
    local: Option<&'static dyn StorageApi>,
    session: Option<&'static dyn StorageApi>,
) -> FakeStorageGuard {
    FAKE.with(|fake| fake.set(Some([local, session])));
    FakeStorageGuard
}

#[cfg(test)]
type ChangeCallback = Box<dyn Fn(StorageChange)>;

/// A store in memory that can be told to refuse, for tests.
#[cfg(test)]
#[derive(Default)]
pub(crate) struct MemoryStorage {
    pub(crate) values: std::cell::RefCell<std::collections::HashMap<String, String>>,
    pub(crate) refuse: std::cell::Cell<Option<StorageError>>,
    changes: std::cell::RefCell<Vec<ChangeCallback>>,
}

#[cfg(test)]
impl MemoryStorage {
    pub(crate) fn leaked() -> &'static Self {
        Box::leak(Box::default())
    }

    /// Stands in for another tab's write: stores it and tells every listener.
    pub(crate) fn change_elsewhere(&self, key: Option<&str>, value: Option<&str>) {
        match (key, value) {
            (Some(key), Some(value)) => {
                drop(self.values.borrow_mut().insert(key.into(), value.into()))
            }
            (Some(key), None) => drop(self.values.borrow_mut().remove(key)),
            (None, _) => self.values.borrow_mut().clear(),
        }
        for callback in self.changes.borrow().iter() {
            callback(StorageChange {
                key: key.map(Into::into),
                value: value.map(Into::into),
            });
        }
    }

    fn check(&self) -> Result<(), StorageError> {
        self.refuse.get().map_or(Ok(()), Err)
    }
}

#[cfg(test)]
struct MemorySubscription;

#[cfg(test)]
impl StorageSubscription for MemorySubscription {}

#[cfg(test)]
impl StorageApi for MemoryStorage {
    fn get(&self, key: &str) -> Result<Option<String>, StorageError> {
        self.check()?;
        Ok(self.values.borrow().get(key).cloned())
    }

    fn set(&self, key: &str, value: &str) -> Result<(), StorageError> {
        self.check()?;
        self.values.borrow_mut().insert(key.into(), value.into());
        Ok(())
    }

    fn remove(&self, key: &str) -> Result<(), StorageError> {
        self.check()?;
        self.values.borrow_mut().remove(key);
        Ok(())
    }

    fn on_change(
        &self,
        callback: Box<dyn Fn(StorageChange)>,
    ) -> Option<Box<dyn StorageSubscription>> {
        self.changes.borrow_mut().push(callback);
        Some(Box::new(MemorySubscription))
    }
}

/// `area`'s store; `None` where it lives in memory for the document: session
/// storage off the web, and local storage without an app directory.
pub(crate) fn storage(area: StorageArea) -> Option<&'static dyn StorageApi> {
    #[cfg(test)]
    if let Some(fakes) = FAKE.with(std::cell::Cell::get) {
        return fakes[area as usize];
    }
    #[cfg(target_arch = "wasm32")]
    return Some(match area {
        StorageArea::Local => &web::LOCAL,
        StorageArea::Session => &web::SESSION,
    });
    #[cfg(not(target_arch = "wasm32"))]
    return match area {
        StorageArea::Local => files::local_dir().map(|_| &files::FILES as &dyn StorageApi),
        StorageArea::Session => None,
    };
}

/// Keeps [`use_local_storage`](crate::hooks::use_local_storage) off the web as one
/// file per key in `dir`. Ignored on the web.
///
/// Builds with libero's `native` or `desktop` feature, and Android, keep a
/// directory of their own; any other build keeps local storage in memory unless
/// this names one, because a server build shares that cfg. Call it at start,
/// before the first value is read.
///
/// ```rust
/// // In `main`, before launching the app.
/// libero::platform::set_storage_dir(std::env::temp_dir().join("my-app"));
/// ```
pub fn set_storage_dir(dir: impl Into<PathBuf>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        *files::DIR
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(dir.into());
    }
    #[cfg(target_arch = "wasm32")]
    let _ = dir;
}

/// Libero's own setting under `key` in local storage, as raw text. Android also
/// reads the file an earlier libero kept straight in the app's files directory.
pub(crate) fn kept(key: &str) -> Option<String> {
    let stored = storage(StorageArea::Local)?.get(key).ok().flatten();
    #[cfg(target_os = "android")]
    let stored = stored.or_else(|| std::fs::read_to_string(app_dir()?.join(key)).ok());
    stored
}

/// Keeps libero's own setting as raw text, `None` drops it. Where the store refuses,
/// the caller's state lasts the session.
pub(crate) fn keep(key: &str, value: Option<&str>) {
    let Some(store) = storage(StorageArea::Local) else {
        return;
    };
    let _ = match value {
        Some(value) => store.set(key, value),
        None => store.remove(key),
    };
    // Else the earlier file would answer again once this one is dropped.
    #[cfg(target_os = "android")]
    if let Some(dir) = app_dir() {
        let _ = std::fs::remove_file(dir.join(key));
    }
}

/// The app's files directory. User 0's path: under a secondary Android user
/// nothing is kept past the session.
#[cfg(target_os = "android")]
fn app_dir() -> Option<PathBuf> {
    static PACKAGE: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    let package = PACKAGE.get_or_init(|| {
        let cmdline = std::fs::read("/proc/self/cmdline").ok()?;
        // The package name, minus a `:process` suffix and the NUL padding.
        let process = cmdline.split(|byte| *byte == 0).next()?;
        let package = std::str::from_utf8(process).ok()?.split(':').next()?;
        (!package.is_empty() && !package.contains('/')).then(|| package.to_string())
    });
    Some(PathBuf::from(format!(
        "/data/data/{}/files",
        package.as_ref()?
    )))
}

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "storage_tests.rs"]
mod tests;
