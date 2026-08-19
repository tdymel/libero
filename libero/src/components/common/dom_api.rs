use crate::components::common::{ElementApi, PlatformError};

/// Resolves selectors into [`ElementApi`]s - the only way to get one.
pub trait DomApi {
    fn query_selector(&self, selector: &str) -> Result<Box<dyn ElementApi>, PlatformError>;
}

#[cfg(target_arch = "wasm32")]
mod web {
    use wasm_bindgen::{JsCast, JsValue};

    use super::DomApi;
    use crate::components::common::{Dimensions, ElementApi, PlatformError};

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

        fn dimensions(&self) -> Result<Dimensions, PlatformError> {
            let rect = self.element.get_bounding_client_rect();
            Ok(Dimensions {
                width: rect.width(),
                height: rect.height(),
            })
        }

        fn scroll_size(&self) -> Result<Dimensions, PlatformError> {
            Ok(Dimensions {
                width: self.element.scroll_width() as f64,
                height: self.element.scroll_height() as f64,
            })
        }

        fn scroll_offset(&self) -> Result<(f64, f64), PlatformError> {
            Ok((
                self.element.scroll_left() as f64,
                self.element.scroll_top() as f64,
            ))
        }

        fn scroll_to(&self, x: f64, y: f64) -> Result<(), PlatformError> {
            self.element.set_scroll_left(x as i32);
            self.element.set_scroll_top(y as i32);
            Ok(())
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

/// The platform that has no DOM at all - every lookup fails before an
/// [`ElementApi`] can exist, so no element impl is needed off the web.
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
}

#[cfg(target_arch = "wasm32")]
static DOM_HANDLE: web::WebDomHandle = web::WebDomHandle;

#[cfg(not(target_arch = "wasm32"))]
static DOM_HANDLE: UnsupportedDomApi = UnsupportedDomApi;

/// The current platform's [`DomApi`] - a plain accessor, not a hook, so it's
/// callable from anywhere including event handlers.
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
    }
}
