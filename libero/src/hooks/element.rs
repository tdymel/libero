use dioxus::prelude::*;

use crate::components::{Dimensions, ElementApi, MountedElement, PlatformError};


/// A handle to one of this component's own elements, implementing
/// [`ElementApi`] - so `element.dimensions()` reads the same here as it does
/// through [`dom_api`](crate::components::dom_api), and works on every
/// renderer rather than only where a `document` exists.
///
/// Attach it with [`mount`](Self::mount); every call answers
/// [`PlatformError::Unsupported`] until the element is mounted.
#[derive(Clone, Copy, PartialEq)]
pub struct ElementHandle {
    mounted: Signal<Option<std::rc::Rc<MountedData>>>,
}

impl ElementHandle {
    /// The `onmounted` handler that fills this handle in:
    /// `onmounted: track.mount()`, or `.event("onmounted", track.mount())`.
    pub fn mount(mut self) -> impl FnMut(dioxus::prelude::Event<MountedData>) + 'static {
        move |event| self.mounted.set(Some(event.data()))
    }

    fn get(&self) -> Result<MountedElement, PlatformError> {
        (self.mounted)()
            .map(MountedElement)
            .ok_or(PlatformError::Unsupported)
    }
}

/// A handle to one of this component's own elements. Positional, like every
/// hook.
pub fn use_element() -> ElementHandle {
    ElementHandle {
        mounted: use_signal(|| None),
    }
}

impl ElementApi for ElementHandle {
    fn focus(&self) -> Result<(), PlatformError> {
        self.get()?.focus()
    }

    fn blur(&self) -> Result<(), PlatformError> {
        self.get()?.blur()
    }

    fn click(&self) -> Result<(), PlatformError> {
        self.get()?.click()
    }

    fn is_focused(&self) -> bool {
        self.get().is_ok_and(|element| element.is_focused())
    }

    fn dimensions(&self) -> Result<Dimensions, PlatformError> {
        self.get()?.dimensions()
    }

    fn client_offset(&self) -> Result<(f64, f64), PlatformError> {
        self.get()?.client_offset()
    }

    fn scroll_size(&self) -> Result<Dimensions, PlatformError> {
        self.get()?.scroll_size()
    }

    fn scroll_offset(&self) -> Result<(f64, f64), PlatformError> {
        self.get()?.scroll_offset()
    }

    fn scroll_to(&self, x: f64, y: f64) -> Result<(), PlatformError> {
        self.get()?.scroll_to(x, y)
    }

    fn set_pointer_capture(&self, pointer_id: i32) -> Result<(), PlatformError> {
        self.get()?.set_pointer_capture(pointer_id)
    }

    fn query_selector(&self, selector: &str) -> Result<Box<dyn ElementApi>, PlatformError> {
        self.get()?.query_selector(selector)
    }

    fn query_selector_all(&self, selector: &str) -> Result<Vec<Box<dyn ElementApi>>, PlatformError> {
        self.get()?.query_selector_all(selector)
    }
}
