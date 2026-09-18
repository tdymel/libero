//! Values kept for the session: `sessionStorage` on the web, so a reload keeps
//! them; memory elsewhere, for the app's lifetime.

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static VALUES: std::cell::RefCell<std::collections::HashMap<String, String>> =
        Default::default();
}

/// The value stored under `key`, if any.
pub(crate) fn session_get(key: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    return storage()?.get_item(key).ok().flatten();
    #[cfg(not(target_arch = "wasm32"))]
    VALUES.with(|values| values.borrow().get(key).cloned())
}

/// Stores `value` under `key`. A full or refused storage keeps nothing.
pub(crate) fn session_set(key: &str, value: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(storage) = storage() {
        let _ = storage.set_item(key, value);
    }
    #[cfg(not(target_arch = "wasm32"))]
    VALUES.with(|values| {
        values
            .borrow_mut()
            .insert(key.to_string(), value.to_string())
    });
}

#[cfg(target_arch = "wasm32")]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.session_storage().ok().flatten()
}
