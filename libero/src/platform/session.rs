//! Values kept for the session: `sessionStorage` on the web, so a reload keeps
//! them; memory elsewhere, for the app's lifetime.

/// Process-wide: Android runs handlers and render on two threads (2217).
#[cfg(not(target_arch = "wasm32"))]
static VALUES: std::sync::Mutex<std::collections::BTreeMap<String, String>> =
    std::sync::Mutex::new(std::collections::BTreeMap::new());

#[cfg(not(target_arch = "wasm32"))]
fn values() -> std::sync::MutexGuard<'static, std::collections::BTreeMap<String, String>> {
    VALUES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The value stored under `key`, if any.
pub(crate) fn session_get(key: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    return storage()?.get_item(key).ok().flatten();
    #[cfg(not(target_arch = "wasm32"))]
    values().get(key).cloned()
}

/// Stores `value` under `key`. A full or refused storage keeps nothing.
pub(crate) fn session_set(key: &str, value: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(storage) = storage() {
        let _ = storage.set_item(key, value);
    }
    #[cfg(not(target_arch = "wasm32"))]
    values().insert(key.to_string(), value.to_string());
}

#[cfg(target_arch = "wasm32")]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.session_storage().ok().flatten()
}
