use std::{future::Future, pin::Pin};

use super::PlatformError;

/// Resolves once the platform accepted the write, so a rejection is reported.
pub(crate) type Write = Pin<Box<dyn Future<Output = Result<(), PlatformError>>>>;

/// Writing to the system clipboard.
pub(crate) trait ClipboardApi {
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

/// A desktop clipboard, kept for the process: `blitz-shell` drops its arboard
/// handle per write, and on X11 that drop empties the selection.
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
mod native {
    use std::cell::RefCell;

    use super::{ClipboardApi, Write};
    use crate::platform::PlatformError;

    thread_local! {
        /// Built on first use, never dropped: dropping loses the copied text.
        static HANDLE: RefCell<Option<arboard::Clipboard>> = const { RefCell::new(None) };
    }

    pub(super) struct NativeClipboard;

    impl ClipboardApi for NativeClipboard {
        fn write_text(&self, text: String) -> Write {
            let written = HANDLE.with(|handle| {
                let mut handle = handle.borrow_mut();
                if handle.is_none() {
                    *handle = arboard::Clipboard::new().ok();
                }
                let Some(clipboard) = handle.as_mut() else {
                    return Err(PlatformError::Unsupported);
                };
                clipboard.set_text(text).map_err(|_| PlatformError::Denied)
            });
            Box::pin(std::future::ready(written))
        }
    }

    pub(super) static CLIPBOARD: NativeClipboard = NativeClipboard;
}

/// `None` where the platform has no clipboard: never a stub answering `Unsupported`.
pub(crate) fn clipboard() -> Option<&'static dyn ClipboardApi> {
    #[cfg(target_arch = "wasm32")]
    return Some(&web::CLIPBOARD);
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return Some(&native::CLIPBOARD);
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return super::backend::webview_clipboard();
}
