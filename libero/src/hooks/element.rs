use std::rc::Rc;

use dioxus::prelude::*;

use crate::platform::{Dimensions, ElementApi, PlatformError, Read, backend};

/// A handle to one of this component's own elements, and the only way to reach
/// an element at all.
///
/// It implements [`ElementApi`] itself, so `element.dimensions().await` reads
/// the same everywhere; behind it the handle picks the richest backing the
/// renderer offers - the real `web_sys::Element` on the web, a Blitz node
/// natively, and dioxus's portable `MountedData` under a webview, where
/// measuring and focus still work but subtree queries cannot.
///
/// Attach it with `.element(&handle)` on the `Box` builder, or with
/// [`mount`](Self::mount) directly; every call answers
/// [`PlatformError::Unsupported`] until the element is mounted.
#[derive(Clone, Copy, PartialEq)]
pub struct ElementHandle {
    mounted: Signal<Option<Rc<MountedData>>>,
}

impl ElementHandle {
    /// The `onmounted` handler that fills this handle in. Prefer
    /// `.element(&handle)` on the `Box` builder, which wires this up.
    pub fn mount(mut self) -> impl FnMut(Event<MountedData>) + 'static {
        move |event| self.mounted.set(Some(event.data()))
    }

    /// Reactive: an effect that reads this re-runs once the element mounts.
    pub fn is_mounted(&self) -> bool {
        self.mounted.read().is_some()
    }

    /// Identifies *which* mount this is, not merely that one happened.
    ///
    /// An element that unmounts and comes back - a portaled dropdown closing
    /// and reopening - hands over a different `MountedData` at the same handle,
    /// so [`is_mounted`](Self::is_mounted) never changes and an effect watching
    /// it would keep measuring the dead node. Reading this subscribes to the
    /// mount instead.
    pub(crate) fn mount_token(&self) -> Option<usize> {
        self.mounted
            .read()
            .as_ref()
            .map(|mounted| Rc::as_ptr(mounted) as *const () as usize)
    }

    fn get(&self) -> Result<Box<dyn ElementApi>, PlatformError> {
        match self.mounted.read().as_ref() {
            Some(mounted) => Ok(backend::element(mounted)),
            None => Err(PlatformError::Unsupported),
        }
    }

    fn read<T: 'static>(&self, call: impl FnOnce(&dyn ElementApi) -> Read<T>) -> Read<T> {
        match self.get() {
            Ok(element) => call(element.as_ref()),
            Err(error) => Box::pin(std::future::ready(Err(error))),
        }
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

    fn dimensions(&self) -> Read<Dimensions> {
        self.read(|element| element.dimensions())
    }

    fn client_offset(&self) -> Read<(f64, f64)> {
        self.read(|element| element.client_offset())
    }

    fn scroll_size(&self) -> Read<Dimensions> {
        self.read(|element| element.scroll_size())
    }

    fn scroll_offset(&self) -> Read<(f64, f64)> {
        self.read(|element| element.scroll_offset())
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

    fn query_selector_all(
        &self,
        selector: &str,
    ) -> Result<Vec<Box<dyn ElementApi>>, PlatformError> {
        self.get()?.query_selector_all(selector)
    }
}
