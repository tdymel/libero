use dioxus::prelude::*;

use crate::components::PlatformError;

/// Writes text to the system clipboard - the only DOM-adjacent operation in
/// this crate that isn't element-scoped (no selector/`ElementApi` applies),
/// so it gets its own tiny trait here instead of living on `DomApi`. Only
/// this hook uses it, so it stays private to this module.
trait ClipboardApi {
    fn write_text(&self, text: &str) -> Result<(), PlatformError>;
}

#[cfg(target_arch = "wasm32")]
mod web {
    use wasm_bindgen_futures::JsFuture;

    use super::ClipboardApi;
    use crate::components::PlatformError;

    pub(super) struct WebClipboardHandle;

    impl ClipboardApi for WebClipboardHandle {
        fn write_text(&self, text: &str) -> Result<(), PlatformError> {
            let window = web_sys::window().ok_or(PlatformError::Unsupported)?;
            let promise = window.navigator().clipboard().write_text(text);
            wasm_bindgen_futures::spawn_local(async move {
                let _ = JsFuture::from(promise).await;
            });
            Ok(())
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct UnsupportedClipboardApi;

#[cfg(not(target_arch = "wasm32"))]
impl ClipboardApi for UnsupportedClipboardApi {
    fn write_text(&self, _text: &str) -> Result<(), PlatformError> {
        crate::components::warn("clipboard write on a target without one");
        Err(PlatformError::Unsupported)
    }
}

#[cfg(target_arch = "wasm32")]
static CLIPBOARD_HANDLE: web::WebClipboardHandle = web::WebClipboardHandle;

#[cfg(not(target_arch = "wasm32"))]
static CLIPBOARD_HANDLE: UnsupportedClipboardApi = UnsupportedClipboardApi;

fn clipboard_api() -> &'static dyn ClipboardApi {
    &CLIPBOARD_HANDLE
}

/// Writes to the system clipboard and tracks a transient "just copied" flag
/// - callers decide when to clear it (e.g. `onmouseleave`).
#[derive(Clone, Copy)]
pub struct Clipboard {
    copied: Signal<bool>,
}

impl Clipboard {
    pub fn copy(&mut self, text: impl Into<String>) {
        if clipboard_api().write_text(&text.into()).is_ok() {
            self.copied.set(true);
        }
    }

    pub fn reset(&mut self) {
        self.copied.set(false);
    }

    pub fn copied(&self) -> bool {
        (self.copied)()
    }
}

pub fn use_clipboard() -> Clipboard {
    Clipboard {
        copied: use_signal(|| false),
    }
}
