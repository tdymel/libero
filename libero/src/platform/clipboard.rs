use std::{future::Future, pin::Pin};

use super::PlatformError;

/// The write only resolves once the platform accepted it, so this is a
/// future - reporting success synchronously would lie about a rejection.
pub type Write = Pin<Box<dyn Future<Output = Result<(), PlatformError>>>>;

/// Not element-scoped at all, hence its own capability rather than a place on
/// [`ElementApi`](super::ElementApi).
pub trait ClipboardApi {
    fn write_text(&self, text: String) -> Write;
}

#[cfg(target_arch = "wasm32")]
mod web {
    use wasm_bindgen_futures::JsFuture;

    use super::{ClipboardApi, Write};
    use crate::platform::PlatformError;

    pub(super) struct WebClipboard;

    impl ClipboardApi for WebClipboard {
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

    pub(super) static CLIPBOARD: WebClipboard = WebClipboard;
}

/// `None` where the platform has no clipboard at all - an absent capability is
/// a missing accessor, never a stub that answers `Unsupported`.
pub(crate) fn clipboard() -> Option<&'static dyn ClipboardApi> {
    #[cfg(target_arch = "wasm32")]
    return Some(&web::CLIPBOARD);
    #[cfg(not(target_arch = "wasm32"))]
    return None;
}
