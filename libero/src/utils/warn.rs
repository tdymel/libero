/// Debug-only warning, a no-op in release builds - unlike `dioxus::warn!`,
/// just a plain string, no format-string semantics.
#[cfg(debug_assertions)]
pub(crate) fn warn(message: &str) {
    dioxus::prelude::warn!("{message}");
}

#[cfg(not(debug_assertions))]
pub(crate) fn warn(_message: &str) {}
