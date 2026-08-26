use std::rc::Rc;

use dioxus::prelude::MountedData;
use wasm_bindgen::{JsCast, JsValue};

use crate::platform::{Dimensions, DocumentApi, ElementApi, PlatformError, Read};

/// dioxus-web backs a mounted element with the `web_sys::Element` itself, so
/// the whole trait is answerable - no id, no document lookup.
pub(super) fn element(mounted: &Rc<MountedData>) -> Option<Box<dyn ElementApi>> {
    let element = mounted.downcast::<web_sys::Element>()?.clone();
    Some(Box::new(WebElement { element }))
}

pub(super) fn document() -> Option<Box<dyn DocumentApi>> {
    Some(Box::new(WebDocument))
}

struct WebDocument;

impl DocumentApi for WebDocument {
    fn active_element(&self) -> Option<Box<dyn ElementApi>> {
        let element = web_sys::window()?.document()?.active_element()?;
        Some(Box::new(WebElement { element }))
    }
}

pub(super) struct WebElement {
    element: web_sys::Element,
}

impl WebElement {
    /// Focus and clicks are `HtmlElement`'s, not every element's - an `<svg>`
    /// matched by a selector is an `Element` and has neither.
    fn html(&self) -> Result<&web_sys::HtmlElement, PlatformError> {
        self.element
            .dyn_ref::<web_sys::HtmlElement>()
            .ok_or(PlatformError::NotFound)
    }
}

fn resolved<T: 'static>(value: T) -> Read<T> {
    Box::pin(std::future::ready(Ok(value)))
}

impl ElementApi for WebElement {
    fn focus(&self) -> Result<(), PlatformError> {
        self.html()?.focus().map_err(|_| PlatformError::NotFound)
    }

    fn blur(&self) -> Result<(), PlatformError> {
        self.html()?.blur().map_err(|_| PlatformError::NotFound)
    }

    fn click(&self) -> Result<(), PlatformError> {
        self.html()?.click();
        Ok(())
    }

    fn is_focused(&self) -> bool {
        web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.active_element())
            .is_some_and(|active| JsValue::from(active) == JsValue::from(self.element.clone()))
    }

    // The DOM answers all four immediately - they are futures only because a
    // webview-backed renderer cannot.
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
            .ok_or(PlatformError::NotFound)?;

        Ok(Box::new(WebElement { element }))
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
            .filter_map(|node| node.dyn_into::<web_sys::Element>().ok())
            .map(|element| Box::new(WebElement { element }) as Box<dyn ElementApi>)
            .collect();

        Ok(items)
    }
}
