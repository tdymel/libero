use std::rc::Rc;

use dioxus::prelude::MountedData;

use super::{PlatformError, Read};

/// Whether the page may go fullscreen, and whether the watched element is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct FullscreenState {
    pub available: bool,
    pub active: bool,
}

/// Dropping it stops the callback.
pub(crate) trait FullscreenSubscription {}

/// The Fullscreen API for one element.
pub(crate) trait FullscreenApi {
    /// `Denied` once the request is refused, e.g. by a headless browser.
    fn enter(&self) -> Read<()>;
    /// Leaves fullscreen, whichever element holds it.
    fn exit(&self) -> Result<(), PlatformError>;
    /// Calls `callback` with the state now (async on a WebView) and after each change.
    fn watch(&self, callback: Box<dyn Fn(FullscreenState)>) -> Box<dyn FullscreenSubscription>;
}

/// `None` where no element can go fullscreen (Blitz, a server). `tag` is the
/// element's `ElementHandle` tag, which a WebView finds it by.
pub(crate) fn fullscreen(
    mounted: &Rc<MountedData>,
    tag: Option<u64>,
) -> Option<Box<dyn FullscreenApi>> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = tag;
        web::fullscreen(mounted)
    }
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    {
        let _ = mounted;
        super::backend::webview_fullscreen(tag?)
    }
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    {
        let _ = (mounted, tag);
        None
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use std::rc::Rc;

    use dioxus::prelude::MountedData;
    use js_sys::{Function, Promise, Reflect};
    use wasm_bindgen::{JsCast, prelude::Closure};
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Document, Element};

    use super::{FullscreenApi, FullscreenState, FullscreenSubscription};
    use crate::platform::{PlatformError, Read};

    const CHANGE: &str = "fullscreenchange";

    pub(super) fn fullscreen(mounted: &Rc<MountedData>) -> Option<Box<dyn FullscreenApi>> {
        let element = mounted.downcast::<Element>()?.clone();
        let document = element.owner_document()?;
        Some(Box::new(WebFullscreen { element, document }))
    }

    struct WebFullscreen {
        element: Element,
        document: Document,
    }

    fn state(element: &Element, document: &Document) -> FullscreenState {
        FullscreenState {
            available: document.fullscreen_enabled(),
            active: document.fullscreen_element().as_ref() == Some(element),
        }
    }

    impl FullscreenApi for WebFullscreen {
        // Through `Reflect`: web-sys drops the promise, so a refusal would go unhandled.
        fn enter(&self) -> Read<()> {
            let promise = Reflect::get(&self.element, &"requestFullscreen".into())
                .ok()
                .and_then(|request| request.dyn_into::<Function>().ok())
                .and_then(|request| request.call0(&self.element).ok())
                .and_then(|promise| promise.dyn_into::<Promise>().ok());
            Box::pin(async move {
                let promise = promise.ok_or(PlatformError::Unsupported)?;
                JsFuture::from(promise)
                    .await
                    .map(|_| ())
                    .map_err(|_| PlatformError::Denied)
            })
        }

        fn exit(&self) -> Result<(), PlatformError> {
            if self.document.fullscreen_element().is_some() {
                self.document.exit_fullscreen();
            }
            Ok(())
        }

        fn watch(&self, callback: Box<dyn Fn(FullscreenState)>) -> Box<dyn FullscreenSubscription> {
            callback(state(&self.element, &self.document));
            let (element, document) = (self.element.clone(), self.document.clone());
            let closure = Closure::<dyn FnMut()>::new(move || callback(state(&element, &document)));
            let _ = self
                .document
                .add_event_listener_with_callback(CHANGE, closure.as_ref().unchecked_ref());
            Box::new(WebFullscreenSubscription {
                document: self.document.clone(),
                closure,
            })
        }
    }

    struct WebFullscreenSubscription {
        document: Document,
        closure: Closure<dyn FnMut()>,
    }

    impl FullscreenSubscription for WebFullscreenSubscription {}

    impl Drop for WebFullscreenSubscription {
        fn drop(&mut self) {
            let _ = self
                .document
                .remove_event_listener_with_callback(CHANGE, self.closure.as_ref().unchecked_ref());
        }
    }
}
