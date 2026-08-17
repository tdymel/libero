use std::rc::Rc;

use wasm_bindgen::JsCast;

use crate::components::common::ElementApi;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomApiError {
    NotFound,
}

/// Resolves selectors into [`ElementApi`]s - the only way to get one.
pub trait DomApi {
    fn query_selector(&self, selector: Rc<str>) -> Result<Box<dyn ElementApi>, DomApiError>;
}

struct WebDomHandle;

impl DomApi for WebDomHandle {
    fn query_selector(&self, selector: Rc<str>) -> Result<Box<dyn ElementApi>, DomApiError> {
        let element = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.query_selector(&selector).ok().flatten())
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
}

static WEB_DOM_HANDLE: WebDomHandle = WebDomHandle;

/// The current platform's [`DomApi`] - a plain accessor, not a hook, so it's
/// callable from anywhere including event handlers.
pub fn dom_api() -> &'static dyn DomApi {
    &WEB_DOM_HANDLE
}
