//! Text kept under a key without blocking the page: IndexedDB on the web, one file
//! per key off it. Every call answers later, as IndexedDB does. Serialising is the hook's job.

use std::{future::Future, pin::Pin};

use super::{StorageChange, StorageError, StorageSubscription};

/// Resolves once the store has done the work.
pub(crate) type Op<T> = Pin<Box<dyn Future<Output = Result<T, StorageError>>>>;

/// The origin's key-value store of raw text.
pub(crate) trait IndexedDbApi {
    fn get(&self, key: &str) -> Op<Option<String>>;
    fn set(&self, key: &str, value: &str) -> Op<()>;
    fn remove(&self, key: &str) -> Op<()>;

    /// Changes another tab made; `None` where none can arrive.
    fn on_change(
        &self,
        _callback: Box<dyn Fn(StorageChange)>,
    ) -> Option<Box<dyn StorageSubscription>> {
        None
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use std::cell::OnceCell;

    use js_sys::{Object, Promise, Reflect};
    use wasm_bindgen::{JsCast, JsValue, closure::Closure};
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{
        BroadcastChannel, IdbDatabase, IdbObjectStore, IdbRequest, IdbTransactionMode, MessageEvent,
    };

    use super::{IndexedDbApi, Op, StorageChange, StorageError, StorageSubscription};

    const DATABASE: &str = "libero";
    const STORE: &str = "kv";
    const CHANNEL: &str = "libero-indexed-db";

    thread_local! {
        /// Opened on the first call and kept, a refusal too: blocked IndexedDB stays blocked.
        static OPENING: OnceCell<Promise> = const { OnceCell::new() };
        /// One object both sends and listens: a channel never hears its own posts.
        static BROADCAST: OnceCell<Option<BroadcastChannel>> = const { OnceCell::new() };
    }

    pub(super) struct WebIndexedDb;

    pub(super) static INDEXED_DB: WebIndexedDb = WebIndexedDb;

    impl IndexedDbApi for WebIndexedDb {
        fn get(&self, key: &str) -> Op<Option<String>> {
            let key = JsValue::from_str(key);
            Box::pin(async move {
                let found = run(IdbTransactionMode::Readonly, |store| store.get(&key)).await?;
                Ok(found.as_string())
            })
        }

        fn set(&self, key: &str, value: &str) -> Op<()> {
            let (key, value) = (JsValue::from_str(key), JsValue::from_str(value));
            Box::pin(async move {
                run(IdbTransactionMode::Readwrite, |store| {
                    store.put_with_key(&value, &key)
                })
                .await?;
                announce(&key, &value);
                Ok(())
            })
        }

        fn remove(&self, key: &str) -> Op<()> {
            let key = JsValue::from_str(key);
            Box::pin(async move {
                run(IdbTransactionMode::Readwrite, |store| store.delete(&key)).await?;
                announce(&key, &JsValue::NULL);
                Ok(())
            })
        }

        fn on_change(
            &self,
            callback: Box<dyn Fn(StorageChange)>,
        ) -> Option<Box<dyn StorageSubscription>> {
            let channel = channel()?;
            let closure = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
                let field = |name: &str| Reflect::get(&event.data(), &name.into()).ok();
                if let Some(key) = field("key").and_then(|key| key.as_string()) {
                    let value = field("value").and_then(|value| value.as_string());
                    callback(StorageChange {
                        key: Some(key),
                        value,
                    });
                }
            });
            channel.set_onmessage(Some(closure.as_ref().unchecked_ref()));
            Some(Box::new(WebSubscription {
                channel,
                _closure: closure,
            }))
        }
    }

    struct WebSubscription {
        channel: BroadcastChannel,
        _closure: Closure<dyn FnMut(MessageEvent)>,
    }

    impl StorageSubscription for WebSubscription {}

    impl Drop for WebSubscription {
        fn drop(&mut self) {
            self.channel.set_onmessage(None);
        }
    }

    fn channel() -> Option<BroadcastChannel> {
        BROADCAST.with(|channel| {
            channel
                .get_or_init(|| BroadcastChannel::new(CHANNEL).ok())
                .clone()
        })
    }

    /// Tells the origin's other tabs that `key` now holds `value` (`null`: removed).
    fn announce(key: &JsValue, value: &JsValue) {
        let Some(channel) = channel() else {
            return;
        };
        let message = Object::new();
        let _ = Reflect::set(&message, &"key".into(), key);
        let _ = Reflect::set(&message, &"value".into(), value);
        let _ = channel.post_message(&message);
    }

    fn open() -> Promise {
        Promise::new(&mut |resolve, reject| {
            let request = web_sys::window()
                .and_then(|window| window.indexed_db().ok().flatten())
                .and_then(|factory| factory.open_with_u32(DATABASE, 1).ok());
            let Some(request) = request else {
                let _ = reject.call0(&JsValue::NULL);
                return;
            };
            let created = request.clone();
            let upgrade = Closure::once_into_js(move || {
                if let Ok(database) = created.result() {
                    let _ = database
                        .unchecked_into::<IdbDatabase>()
                        .create_object_store(STORE);
                }
            });
            request.set_onupgradeneeded(Some(upgrade.unchecked_ref()));
            let (opened, failed) = (request.clone(), reject.clone());
            let success = Closure::once_into_js(move || {
                let _ = match opened.result() {
                    Ok(database) => resolve.call1(&JsValue::NULL, &database),
                    Err(error) => failed.call1(&JsValue::NULL, &error),
                };
            });
            request.set_onsuccess(Some(success.unchecked_ref()));
            request.set_onerror(Some(&reject));
        })
    }

    async fn database() -> Result<IdbDatabase, StorageError> {
        let opening = OPENING.with(|opening| opening.get_or_init(open).clone());
        let database = (JsFuture::from(opening).await).map_err(|_| StorageError::Unavailable)?;
        Ok(database.unchecked_into())
    }

    /// Runs `request` in one transaction and answers once it is committed.
    async fn run(
        mode: IdbTransactionMode,
        request: impl FnOnce(&IdbObjectStore) -> Result<IdbRequest, JsValue>,
    ) -> Result<JsValue, StorageError> {
        let database = database().await?;
        let transaction = (database.transaction_with_str_and_mode(STORE, mode))
            .map_err(|error| refusal(&error))?;
        let request = (transaction.object_store(STORE))
            .and_then(|store| request(&store))
            .map_err(|error| refusal(&error))?;
        let committed = Promise::new(&mut |resolve, reject| {
            transaction.set_oncomplete(Some(&resolve));
            // A failed request aborts its transaction.
            transaction.set_onabort(Some(&reject));
        });
        if JsFuture::from(committed).await.is_err() {
            return Err(transaction
                .error()
                .map_or(StorageError::Unavailable, |error| refusal(&error)));
        }
        request.result().map_err(|error| refusal(&error))
    }

    fn refusal(error: &JsValue) -> StorageError {
        let name = Reflect::get(error, &"name".into()).ok();
        match name.and_then(|name| name.as_string()).as_deref() {
            Some("QuotaExceededError") => StorageError::Full,
            _ => StorageError::Unavailable,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod files {
    use std::future::ready;
    use std::path::PathBuf;

    use super::{IndexedDbApi, Op, StorageError};
    use crate::platform::storage::files::{delete, indexed_dir, read, write};

    pub(super) struct IndexedFiles;

    pub(super) static FILES: IndexedFiles = IndexedFiles;

    fn dir() -> Result<PathBuf, StorageError> {
        indexed_dir().ok_or(StorageError::Unavailable)
    }

    /// The work is done at once; the future only hands the answer over later.
    impl IndexedDbApi for IndexedFiles {
        fn get(&self, key: &str) -> Op<Option<String>> {
            Box::pin(ready(dir().and_then(|dir| read(&dir, key))))
        }

        fn set(&self, key: &str, value: &str) -> Op<()> {
            Box::pin(ready(dir().and_then(|dir| write(&dir, key, value))))
        }

        fn remove(&self, key: &str) -> Op<()> {
            Box::pin(ready(dir().and_then(|dir| delete(&dir, key))))
        }
    }
}

/// The origin's store; `None` where values live in memory for the document: local
/// files without an app directory, and a server build.
pub(crate) fn indexed_db() -> Option<&'static dyn IndexedDbApi> {
    #[cfg(test)]
    if let Some(fake) = FAKE.with(std::cell::Cell::get) {
        return fake;
    }
    #[cfg(target_arch = "wasm32")]
    return Some(&web::INDEXED_DB);
    #[cfg(not(target_arch = "wasm32"))]
    return crate::platform::storage::files::indexed_dir()
        .map(|_| &files::FILES as &dyn IndexedDbApi);
}

#[cfg(test)]
type FakeStore = Option<&'static dyn IndexedDbApi>;

#[cfg(test)]
thread_local! {
    static FAKE: std::cell::Cell<Option<FakeStore>> = const { std::cell::Cell::new(None) };
}

/// Puts back the platform's store when dropped.
#[cfg(test)]
pub(crate) struct FakeIndexedDbGuard;

#[cfg(test)]
impl Drop for FakeIndexedDbGuard {
    fn drop(&mut self) {
        FAKE.with(|fake| fake.set(None));
    }
}

/// Answers `indexed_db()` with `store` on this thread until the guard drops; `None`
/// stands for no store at all.
#[cfg(test)]
pub(crate) fn fake_indexed_db(store: Option<&'static dyn IndexedDbApi>) -> FakeIndexedDbGuard {
    FAKE.with(|fake| fake.set(Some(store)));
    FakeIndexedDbGuard
}

#[cfg(test)]
pub(crate) use memory::MemoryIndexedDb;

/// A store in memory whose answers can be held back or refused, for tests.
#[cfg(test)]
mod memory {
    use std::cell::{Cell, RefCell};
    use std::collections::HashMap;
    use std::ops::Deref;
    use std::rc::Rc;
    use std::task::{Poll, Waker};

    use super::{IndexedDbApi, Op, StorageChange, StorageError, StorageSubscription};

    type ChangeCallback = Box<dyn Fn(StorageChange)>;

    #[derive(Default)]
    pub(crate) struct State {
        pub(crate) values: RefCell<HashMap<String, String>>,
        pub(crate) refuse: Cell<Option<StorageError>>,
        held: Cell<bool>,
        waiting: RefCell<Vec<Waker>>,
        changes: RefCell<Vec<ChangeCallback>>,
    }

    #[derive(Default)]
    pub(crate) struct MemoryIndexedDb(Rc<State>);

    impl Deref for MemoryIndexedDb {
        type Target = State;

        fn deref(&self) -> &State {
            &self.0
        }
    }

    impl MemoryIndexedDb {
        pub(crate) fn leaked() -> &'static Self {
            Box::leak(Box::default())
        }

        /// Every call waits, work not yet done, until [`release`](Self::release).
        pub(crate) fn hold(&self) {
            self.held.set(true);
        }

        pub(crate) fn release(&self) {
            self.held.set(false);
            for waker in self.waiting.take() {
                waker.wake();
            }
        }

        /// Stands in for another tab's write: stores it and tells every listener.
        pub(crate) fn change_elsewhere(&self, key: &str, value: Option<&str>) {
            match value {
                Some(value) => drop(self.values.borrow_mut().insert(key.into(), value.into())),
                None => drop(self.values.borrow_mut().remove(key)),
            }
            for callback in self.changes.borrow().iter() {
                callback(StorageChange {
                    key: Some(key.into()),
                    value: value.map(Into::into),
                });
            }
        }

        /// Waits out a hold, then does `work` unless the store refuses.
        fn after_hold<T: 'static>(&self, work: impl FnOnce(&State) -> T + 'static) -> Op<T> {
            let state = Rc::clone(&self.0);
            let mut work = Some(work);
            Box::pin(std::future::poll_fn(move |cx| {
                if state.held.get() {
                    state.waiting.borrow_mut().push(cx.waker().clone());
                    return Poll::Pending;
                }
                let work = work.take().expect("polled after it was ready");
                Poll::Ready(match state.refuse.get() {
                    Some(error) => Err(error),
                    None => Ok(work(&state)),
                })
            }))
        }
    }

    struct MemorySubscription;

    impl StorageSubscription for MemorySubscription {}

    impl IndexedDbApi for MemoryIndexedDb {
        fn get(&self, key: &str) -> Op<Option<String>> {
            // As of the call, not the answer: a write made while held is not seen.
            let seen = self.values.borrow().get(key).cloned();
            self.after_hold(move |_| seen)
        }

        fn set(&self, key: &str, value: &str) -> Op<()> {
            let (key, value) = (key.to_string(), value.to_string());
            self.after_hold(move |state| drop(state.values.borrow_mut().insert(key, value)))
        }

        fn remove(&self, key: &str) -> Op<()> {
            let key = key.to_string();
            self.after_hold(move |state| drop(state.values.borrow_mut().remove(&key)))
        }

        fn on_change(
            &self,
            callback: Box<dyn Fn(StorageChange)>,
        ) -> Option<Box<dyn StorageSubscription>> {
            self.changes.borrow_mut().push(callback);
            Some(Box::new(MemorySubscription))
        }
    }
}
