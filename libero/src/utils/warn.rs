/// Debug-only warning. Plain string, no `dioxus::warn!` format semantics.
#[cfg(debug_assertions)]
pub(crate) fn warn(message: &str) {
    dioxus::prelude::warn!("{message}");
}

#[cfg(not(debug_assertions))]
pub(crate) fn warn(_message: &str) {}
