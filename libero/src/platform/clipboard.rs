use std::{future::Future, pin::Pin};

use super::PlatformError;

/// The write only resolves once the platform accepted it, so this is a
/// future - reporting success synchronously would lie about a rejection.
pub(crate) type Write = Pin<Box<dyn Future<Output = Result<(), PlatformError>>>>;

/// Not element-scoped at all, hence its own capability rather than a place on
/// [`ElementApi`](super::ElementApi).
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

/// A desktop clipboard, under Blitz or anything else native. Not routed
/// through the shell: `blitz-shell` drops its `arboard::Clipboard` after every
/// write, and arboard's X11 backend destroys the window owning the selection
/// on the last drop - the write reports success and the paste comes up empty.
/// So the handle is kept for the process instead.
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
mod native {
    use std::cell::RefCell;

    use super::{ClipboardApi, Write};
    use crate::platform::PlatformError;

    thread_local! {
        /// Built on first use, then never dropped - dropping it is what loses
        /// the copied text.
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

/// `None` where the platform has no clipboard at all - an absent capability is
/// a missing accessor, never a stub that answers `Unsupported`.
pub(crate) fn clipboard() -> Option<&'static dyn ClipboardApi> {
    #[cfg(target_arch = "wasm32")]
    return Some(&web::CLIPBOARD);
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return Some(&native::CLIPBOARD);
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return super::backend::webview_clipboard();
}
