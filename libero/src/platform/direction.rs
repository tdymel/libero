//! The app's text direction on the document root, and where a choice is kept.

use super::document;
use crate::tokens::Direction;

/// Where a chosen direction is kept on the web, beside `lsx-color-scheme`.
#[cfg(target_arch = "wasm32")]
const DIRECTION_STORAGE_KEY: &str = "lsx-direction";

/// The stored choice. `None` where nothing was stored, and off the web, where
/// a choice lives for the session.
pub(crate) fn stored_direction() -> Option<Direction> {
    #[cfg(target_arch = "wasm32")]
    return web_sys::window()?
        .local_storage()
        .ok()??
        .get_item(DIRECTION_STORAGE_KEY)
        .ok()?
        .as_deref()
        .and_then(Direction::parse);
    #[cfg(not(target_arch = "wasm32"))]
    None
}

/// Keeps `direction` where the platform can.
pub(crate) fn store_direction(direction: Direction) {
    #[cfg(target_arch = "wasm32")]
    if let Some(storage) = web_sys::window().and_then(|window| window.local_storage().ok()?) {
        let _ = storage.set_item(DIRECTION_STORAGE_KEY, direction.as_str());
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = direction;
}

/// Drops the kept choice.
pub(crate) fn forget_direction() {
    #[cfg(target_arch = "wasm32")]
    if let Some(storage) = web_sys::window().and_then(|window| window.local_storage().ok()?) {
        let _ = storage.remove_item(DIRECTION_STORAGE_KEY);
    }
}

/// Removes the root's `dir`, so the page runs as it would without one.
pub(crate) fn clear_root_direction() {
    if !document().is_some_and(|document| document.set_root_attribute("dir", None)) {
        dioxus::document::eval("document.documentElement.removeAttribute('dir');");
    }
}

/// Sets the root's `dir`, so the portal outlet turns with the app. A WebView
/// holds no handle on its DOM, so there the page's own script does it.
pub(crate) fn apply_direction(direction: Direction) {
    if !set_root_direction(direction) {
        dioxus::document::eval(&format!(
            "document.documentElement.dir = '{}';",
            direction.as_str()
        ));
    }
}

/// The root's `dir` through the document handle alone. `false` where there is
/// none yet: a WebView, or Blitz before the provider mounted.
pub(crate) fn set_root_direction(direction: Direction) -> bool {
    document().is_some_and(|document| document.set_root_attribute("dir", Some(direction.as_str())))
}
