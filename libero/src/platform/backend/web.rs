use std::rc::Rc;

use dioxus::prelude::MountedData;
use wasm_bindgen::{JsCast, JsValue};

use wasm_bindgen::prelude::Closure;

use crate::platform::{
    Dimensions, DocumentApi, ElementApi, PlatformError, Read, ScrollApi, ScrollSubscription,
};

/// dioxus-web backs a mounted element with the `web_sys::Element` itself, so
/// the whole trait is answerable - no id, no document lookup.
pub(super) fn element(mounted: &Rc<MountedData>) -> Option<Box<dyn ElementApi>> {
    let element = mounted.downcast::<web_sys::Element>()?.clone();
    Some(Box::new(WebElement { element }))
}

/// What a component-driven control held before a form reset.
enum Controlled {
    Input(String, bool),
    Text(String),
    Select(i32),
    Other,
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

    /// `inner_width`/`inner_height`, so the answer includes whatever the
    /// scrollbars leave - which is the box a fixed element is laid out in.
    fn viewport(&self) -> Read<Dimensions> {
        let size = web_sys::window()
            .and_then(|window| {
                let width = window.inner_width().ok()?.as_f64()?;
                let height = window.inner_height().ok()?.as_f64()?;
                Some(Dimensions { width, height })
            })
            .ok_or(PlatformError::Unsupported);
        Box::pin(std::future::ready(size))
    }
}

pub(super) fn scroll() -> Option<Box<dyn ScrollApi>> {
    Some(Box::new(WebScroll))
}

struct WebScroll;

impl ScrollApi for WebScroll {
    /// **In the capture phase, on the window.** A `scroll` event does not
    /// bubble, so a plain listener here would see the page scrolling and miss
    /// every element that scrolls - including a `ScrollArea`, which is what the
    /// docs shell scrolls and where this was first noticed. Capture sees every
    /// scroll in the document on its way down.
    fn on_scroll(&self, callback: Box<dyn Fn()>) -> Box<dyn ScrollSubscription> {
        let closure = Closure::<dyn FnMut()>::new(move || callback());
        let target = web_sys::window().and_then(|window| {
            let target: web_sys::EventTarget = window.into();
            target
                .add_event_listener_with_callback_and_bool(
                    "scroll",
                    closure.as_ref().unchecked_ref(),
                    true,
                )
                .ok()?;
            Some(target)
        });

        Box::new(WebScrollSubscription { target, closure })
    }
}

struct WebScrollSubscription {
    /// `None` when there was no window to listen on, so `Drop` has nothing to
    /// undo - the subscription still exists, it just never fires.
    target: Option<web_sys::EventTarget>,
    /// Kept alive for exactly as long as the listener is registered: dropping a
    /// `Closure` frees the JS function the listener still points at.
    closure: Closure<dyn FnMut()>,
}

impl ScrollSubscription for WebScrollSubscription {}

impl Drop for WebScrollSubscription {
    fn drop(&mut self) {
        if let Some(target) = &self.target {
            // The same three arguments it was added with, capture included, or
            // the browser removes nothing.
            let _ = target.remove_event_listener_with_callback_and_bool(
                "scroll",
                self.closure.as_ref().unchecked_ref(),
                true,
            );
        }
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

    fn form(&self) -> Result<&web_sys::HtmlFormElement, PlatformError> {
        self.element
            .dyn_ref::<web_sys::HtmlFormElement>()
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

    /// The native reset, then every control marked `data-controlled` gets its
    /// value back: its component still holds that value, and dioxus never
    /// sets a value again that did not change, so the control would show its
    /// default while the component's state says otherwise.
    fn reset(&self) -> Result<(), PlatformError> {
        let form = self.form()?;
        let marked = form
            .query_selector_all("[data-controlled]")
            .map_err(|_| PlatformError::NotFound)?;
        let saved: Vec<_> = (0..marked.length())
            .filter_map(|index| marked.item(index))
            .map(|node| {
                let state = if let Some(input) = node.dyn_ref::<web_sys::HtmlInputElement>() {
                    Controlled::Input(input.value(), input.checked())
                } else if let Some(text) = node.dyn_ref::<web_sys::HtmlTextAreaElement>() {
                    Controlled::Text(text.value())
                } else if let Some(select) = node.dyn_ref::<web_sys::HtmlSelectElement>() {
                    Controlled::Select(select.selected_index())
                } else {
                    Controlled::Other
                };
                (node, state)
            })
            .collect();
        form.reset();
        for (node, state) in saved {
            match state {
                Controlled::Input(value, checked) => {
                    if let Some(input) = node.dyn_ref::<web_sys::HtmlInputElement>() {
                        input.set_value(&value);
                        input.set_checked(checked);
                    }
                }
                Controlled::Text(value) => {
                    if let Some(text) = node.dyn_ref::<web_sys::HtmlTextAreaElement>() {
                        text.set_value(&value);
                    }
                }
                Controlled::Select(index) => {
                    if let Some(select) = node.dyn_ref::<web_sys::HtmlSelectElement>() {
                        select.set_selected_index(index);
                    }
                }
                Controlled::Other => {}
            }
        }
        Ok(())
    }

    fn request_submit(&self) -> Result<(), PlatformError> {
        self.form()?
            .request_submit()
            .map_err(|_| PlatformError::NotFound)
    }

    /// The list is rebuilt through a `DataTransfer`, which is the only
    /// constructor a `FileList` has. Every file came from a picker or a drop,
    /// so its `FileData` still carries the `web_sys::File` it was made from.
    fn set_files(&self, files: &[dioxus::html::FileData]) -> Result<(), PlatformError> {
        let input = self
            .element
            .dyn_ref::<web_sys::HtmlInputElement>()
            .ok_or(PlatformError::NotFound)?;
        let transfer = web_sys::DataTransfer::new().map_err(|_| PlatformError::Unsupported)?;
        for file in files {
            let file = file
                .inner()
                .downcast_ref::<web_sys::File>()
                .ok_or(PlatformError::Unsupported)?;
            transfer
                .items()
                .add_with_file(file)
                .map_err(|_| PlatformError::Unsupported)?;
        }
        input.set_files(Some(&transfer.files().ok_or(PlatformError::Unsupported)?));
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
