use crate::components::common::{ElementApi, PlatformError};

/// Resolves selectors into [`ElementApi`]s - the only way to get one.
pub trait DomApi {
    fn query_selector(&self, selector: &str) -> Result<Box<dyn ElementApi>, PlatformError>;

    /// Whatever currently has focus. Called synchronously from an event
    /// handler this is the element the user acted on, which is how an overlay
    /// learns where to put focus back.
    fn active_element(&self) -> Result<Box<dyn ElementApi>, PlatformError>;
}

#[cfg(target_arch = "wasm32")]
mod web {
    use wasm_bindgen::{JsCast, JsValue};

    use super::DomApi;
    use crate::components::common::{Dimensions, ElementApi, PlatformError, Read};

    pub(super) struct WebDomHandle;

    impl DomApi for WebDomHandle {
        fn query_selector(&self, selector: &str) -> Result<Box<dyn ElementApi>, PlatformError> {
            let element = web_sys::window()
                .and_then(|window| window.document())
                .and_then(|document| document.query_selector(selector).ok().flatten())
                .ok_or(PlatformError::NotFound)?
                .dyn_into::<web_sys::HtmlElement>()
                .map_err(|_| PlatformError::NotFound)?;

            Ok(Box::new(WebElementHandle { element }))
        }

        fn active_element(&self) -> Result<Box<dyn ElementApi>, PlatformError> {
            let element = web_sys::window()
                .and_then(|window| window.document())
                .and_then(|document| document.active_element())
                .ok_or(PlatformError::NotFound)?
                .dyn_into::<web_sys::HtmlElement>()
                .map_err(|_| PlatformError::NotFound)?;

            Ok(Box::new(WebElementHandle { element }))
        }
    }

    fn resolved<T: 'static>(value: T) -> Read<T> {
        Box::pin(std::future::ready(Ok(value)))
    }

    struct WebElementHandle {
        element: web_sys::HtmlElement,
    }

    impl ElementApi for WebElementHandle {
        fn focus(&self) -> Result<(), PlatformError> {
            self.element.focus().map_err(|_| PlatformError::NotFound)
        }

        fn blur(&self) -> Result<(), PlatformError> {
            self.element.blur().map_err(|_| PlatformError::NotFound)
        }

        fn click(&self) -> Result<(), PlatformError> {
            self.element.click();
            Ok(())
        }

        fn is_focused(&self) -> bool {
            web_sys::window()
                .and_then(|window| window.document())
                .and_then(|document| document.active_element())
                .is_some_and(|active| JsValue::from(active) == JsValue::from(self.element.clone()))
        }

        // The DOM answers all four immediately - they are futures only
        // because a webview-backed renderer cannot.
        fn dimensions(&self) -> Read<Dimensions> {
            let rect = self.element.get_bounding_client_rect();
            resolved(Dimensions {
                width: rect.width(),
                height: rect.height(),
            })
        }

        fn client_offset(&self) -> Read<(f64, f64)> {
            let rect = self.element.get_bounding_client_rect();
            resolved((rect.left(), rect.top()))
        }

        fn scroll_size(&self) -> Read<Dimensions> {
            resolved(Dimensions {
                width: self.element.scroll_width() as f64,
                height: self.element.scroll_height() as f64,
            })
        }

        fn scroll_offset(&self) -> Read<(f64, f64)> {
            resolved((
                self.element.scroll_left() as f64,
                self.element.scroll_top() as f64,
            ))
        }

        fn scroll_to(&self, x: f64, y: f64) -> Result<(), PlatformError> {
            self.element.set_scroll_left(x as i32);
            self.element.set_scroll_top(y as i32);
            Ok(())
        }

        fn set_pointer_capture(&self, pointer_id: i32) -> Result<(), PlatformError> {
            self.element
                .set_pointer_capture(pointer_id)
                .map_err(|_| PlatformError::NotFound)
        }

        fn query_selector(&self, selector: &str) -> Result<Box<dyn ElementApi>, PlatformError> {
            let element = self
                .element
                .query_selector(selector)
                .ok()
                .flatten()
                .ok_or(PlatformError::NotFound)?
                .dyn_into::<web_sys::HtmlElement>()
                .map_err(|_| PlatformError::NotFound)?;

            Ok(Box::new(WebElementHandle { element }))
        }

        fn query_selector_all(
            &self,
            selector: &str,
        ) -> Result<Vec<Box<dyn ElementApi>>, PlatformError> {
            let nodes = self
                .element
                .query_selector_all(selector)
                .map_err(|_| PlatformError::NotFound)?;

            let items = (0..nodes.length())
                .filter_map(|index| nodes.get(index))
                .filter_map(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
                .map(|element| Box::new(WebElementHandle { element }) as Box<dyn ElementApi>)
                .collect();

            Ok(items)
        }
    }
}

/// No DOM at all: every lookup fails before an [`ElementApi`] can exist, so
/// no element impl is needed off the web.
#[cfg(not(target_arch = "wasm32"))]
struct UnsupportedDomApi;

#[cfg(not(target_arch = "wasm32"))]
impl DomApi for UnsupportedDomApi {
    fn query_selector(&self, selector: &str) -> Result<Box<dyn ElementApi>, PlatformError> {
        crate::utils::warn(&format!(
            "dom_api().query_selector({selector:?}) on a target without a DOM"
        ));
        Err(PlatformError::Unsupported)
    }

    // Silent, unlike `query_selector`: callers ask where focus was as a
    // courtesy, and off the web the answer is simply "nowhere".
    fn active_element(&self) -> Result<Box<dyn ElementApi>, PlatformError> {
        Err(PlatformError::Unsupported)
    }
}

#[cfg(target_arch = "wasm32")]
static DOM_HANDLE: web::WebDomHandle = web::WebDomHandle;

#[cfg(not(target_arch = "wasm32"))]
static DOM_HANDLE: UnsupportedDomApi = UnsupportedDomApi;

/// Not a hook, so it's callable from anywhere, event handlers included.
pub fn dom_api() -> &'static dyn DomApi {
    &DOM_HANDLE
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn queries_fail_instead_of_panicking_without_a_dom() {
        assert_eq!(
            dom_api().query_selector("body").err(),
            Some(PlatformError::Unsupported)
        );
        assert_eq!(
            dom_api().active_element().err(),
            Some(PlatformError::Unsupported)
        );
    }
}
