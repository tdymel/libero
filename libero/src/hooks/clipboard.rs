use dioxus::prelude::*;

use crate::components::PlatformError;

/// The only DOM-adjacent operation here that isn't element-scoped, hence its
/// own trait rather than a place on `DomApi`.
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
        crate::utils::warn("clipboard write on a target without one");
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

/// Writes to the clipboard and tracks a "just copied" flag the caller clears.
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
