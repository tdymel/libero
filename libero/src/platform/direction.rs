//! The app's text direction and language on the document root, and where a direction is kept.

use super::{
    document,
    storage::{keep, kept},
};
use crate::tokens::Direction;

/// Where a chosen direction is kept, beside `lsx-color-scheme`.
const DIRECTION_STORAGE_KEY: &str = "lsx-direction";

/// The stored choice. `None` where nothing was stored, and where nothing can be,
/// as a server build: there a choice lives for the session.
pub(crate) fn stored_direction() -> Option<Direction> {
    kept(DIRECTION_STORAGE_KEY)
        .as_deref()
        .and_then(Direction::parse)
}

/// Keeps `direction` where the platform can.
pub(crate) fn store_direction(direction: Direction) {
    keep(DIRECTION_STORAGE_KEY, Some(direction.as_str()));
}

/// Drops the kept choice.
pub(crate) fn forget_direction() {
    keep(DIRECTION_STORAGE_KEY, None);
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

/// Sets the root's `lang`, through the page's script where no handle reaches the DOM.
pub(crate) fn apply_lang(lang: &str) {
    if !document().is_some_and(|document| document.set_root_attribute("lang", Some(lang))) {
        let lang = serde_json::to_string(lang).unwrap_or_default();
        dioxus::document::eval(&format!("document.documentElement.lang = {lang};"));
    }
}

/// The root's `dir` through the document handle alone. `false` where there is
/// none yet: a WebView, or Blitz before the provider mounted.
pub(crate) fn set_root_direction(direction: Direction) -> bool {
    document().is_some_and(|document| document.set_root_attribute("dir", Some(direction.as_str())))
}
