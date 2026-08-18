use wasm_bindgen::{JsCast, JsValue};

use crate::components::common::{Dimensions, ElementApi};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomApiError {
    NotFound,
}

/// Resolves selectors into [`ElementApi`]s - the only way to get one.
pub trait DomApi {
    fn query_selector(&self, selector: &str) -> Result<Box<dyn ElementApi>, DomApiError>;
}

struct WebDomHandle;

impl DomApi for WebDomHandle {
    fn query_selector(&self, selector: &str) -> Result<Box<dyn ElementApi>, DomApiError> {
        let element = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.query_selector(selector).ok().flatten())
            .ok_or(DomApiError::NotFound)?
            .dyn_into::<web_sys::HtmlElement>()
            .map_err(|_| DomApiError::NotFound)?;

        Ok(Box::new(WebElementHandle { element }))
    }
}

struct WebElementHandle {
    element: web_sys::HtmlElement,
}

impl ElementApi for WebElementHandle {
    fn focus(&self) -> Result<(), DomApiError> {
        self.element.focus().map_err(|_| DomApiError::NotFound)
    }

    fn blur(&self) -> Result<(), DomApiError> {
        self.element.blur().map_err(|_| DomApiError::NotFound)
    }

    fn click(&self) -> Result<(), DomApiError> {
        self.element.click();
        Ok(())
    }

    fn is_focused(&self) -> bool {
        web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.active_element())
            .is_some_and(|active| JsValue::from(active) == JsValue::from(self.element.clone()))
    }

    fn dimensions(&self) -> Result<Dimensions, DomApiError> {
        let rect = self.element.get_bounding_client_rect();
        Ok(Dimensions {
            width: rect.width(),
            height: rect.height(),
        })
    }

    fn query_selector(&self, selector: &str) -> Result<Box<dyn ElementApi>, DomApiError> {
        let element = self
            .element
            .query_selector(selector)
            .ok()
            .flatten()
            .ok_or(DomApiError::NotFound)?
            .dyn_into::<web_sys::HtmlElement>()
            .map_err(|_| DomApiError::NotFound)?;

        Ok(Box::new(WebElementHandle { element }))
    }

    fn query_selector_all(&self, selector: &str) -> Result<Vec<Box<dyn ElementApi>>, DomApiError> {
        let nodes = self
            .element
            .query_selector_all(selector)
            .map_err(|_| DomApiError::NotFound)?;

        let items = (0..nodes.length())
            .filter_map(|index| nodes.get(index))
            .filter_map(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
            .map(|element| Box::new(WebElementHandle { element }) as Box<dyn ElementApi>)
            .collect();

        Ok(items)
    }
}

static WEB_DOM_HANDLE: WebDomHandle = WebDomHandle;

/// The current platform's [`DomApi`] - a plain accessor, not a hook, so it's
/// callable from anywhere including event handlers.
pub fn dom_api() -> &'static dyn DomApi {
    &WEB_DOM_HANDLE
}
