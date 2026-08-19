use std::{future::Future, pin::Pin};

use dioxus::prelude::*;

use crate::components::PlatformError;

/// The write only resolves once the platform accepted it, so this is a
/// future - reporting success synchronously would lie about a rejection.
type Write = Pin<Box<dyn Future<Output = Result<(), PlatformError>>>>;

/// The only DOM-adjacent operation here that isn't element-scoped, hence its
/// own trait rather than a place on `DomApi`.
trait ClipboardApi {
    fn write_text(&self, text: String) -> Write;
}

#[cfg(target_arch = "wasm32")]
mod web {
    use wasm_bindgen_futures::JsFuture;

    use super::{ClipboardApi, Write};
    use crate::components::PlatformError;

    pub(super) struct WebClipboardHandle;

    impl ClipboardApi for WebClipboardHandle {
        fn write_text(&self, text: String) -> Write {
            Box::pin(async move {
                let window = web_sys::window().ok_or(PlatformError::Unsupported)?;
                let promise = window.navigator().clipboard().write_text(&text);
                JsFuture::from(promise)
                    .await
                    .map(|_| ())
                    .map_err(|_| PlatformError::Denied)
            })
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct UnsupportedClipboardApi;

#[cfg(not(target_arch = "wasm32"))]
impl ClipboardApi for UnsupportedClipboardApi {
    fn write_text(&self, _text: String) -> Write {
        crate::utils::warn("clipboard write on a target without one");
        Box::pin(std::future::ready(Err(PlatformError::Unsupported)))
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
    /// `copied()` only flips once the platform confirms the write.
    pub fn copy(&mut self, text: impl Into<String>) {
        let write = clipboard_api().write_text(text.into());
        let mut copied = self.copied;
        spawn(async move {
            if write.await.is_ok() {
                copied.set(true);
            }
        });
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
