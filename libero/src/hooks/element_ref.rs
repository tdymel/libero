use std::rc::Rc;

use dioxus::prelude::*;

/// A handle to one specific rendered element, obtained by passing it to
/// [`crate::components::Box`]'s `element_ref` prop - the same role React's
/// `ref` or Leptos's `node_ref` play. Wraps the renderer's `MountedData` behind
/// intent-named methods instead of raw `Event<MountedData>`/`Rc` handling at
/// every call site - and, since it's backed by the same
/// `RenderedElementBacking` trait every Dioxus renderer implements, works
/// wherever that renderer supports the operation, not just on web. Starting
/// point for a general element API: today just `focus`/`blur`, but the same
/// handle is where a future `scroll_into_view`/`rect` etc. would live too.
#[derive(Clone, Copy, PartialEq)]
pub struct ElementRef {
    mounted: Signal<Option<Rc<MountedData>>>,
}

impl ElementRef {
    pub(crate) fn set(&mut self, data: Rc<MountedData>) {
        self.mounted.set(Some(data));
    }

    /// Focuses the element - a no-op until it has actually mounted.
    pub fn focus(&self) {
        self.with_element(|el| {
            spawn(async move {
                let _ = el.set_focus(true).await;
            });
        });
    }

    /// Blurs the element - a no-op until it has actually mounted.
    pub fn blur(&self) {
        self.with_element(|el| {
            spawn(async move {
                let _ = el.set_focus(false).await;
            });
        });
    }

    fn with_element(&self, f: impl FnOnce(Rc<MountedData>)) {
        if let Some(el) = self.mounted.read().clone() {
            f(el);
        }
    }
}

/// Creates an [`ElementRef`] that persists across renders - pass it to a
/// `Box`'s `element_ref` prop, then call `.focus()`/`.blur()` on it from anywhere
/// (an event handler, a signal read elsewhere, ...).
pub fn use_element_ref() -> ElementRef {
    use_hook(|| ElementRef {
        mounted: Signal::new(None),
    })
}
