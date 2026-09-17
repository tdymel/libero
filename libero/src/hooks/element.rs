use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::platform::{
    ContentSubscription, Dimensions, ElementApi, PlatformError, Read, backend, is_rtl,
    on_content_change, on_form_reset,
};

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

    /// A handle owned by the current scope, for one that outlives the element's
    /// own component - a `FormHandle` a parent holds. Not a hook: call it where
    /// a signal may be created.
    pub(crate) fn new() -> Self {
        Self {
            mounted: Signal::new(None),
        }
    }

    /// A handle owned by `owner`, for one made in a child's render that an
    /// ancestor of the child has to read - `Menu`'s submenu boxes.
    pub(crate) fn new_in_scope(owner: ScopeId) -> Self {
        Self {
            mounted: Signal::new_in_scope(None, owner),
        }
    }

    /// The mounted element, for a platform call that needs the node itself.
    pub(crate) fn mounted(&self) -> Option<Rc<MountedData>> {
        self.mounted.peek().clone()
    }

    /// Whether the element lays out right to left; `false` until it mounts.
    /// Not reactive, so a key handler can ask before it returns.
    pub(crate) fn is_rtl(&self) -> bool {
        self.mounted.peek().as_ref().is_some_and(is_rtl)
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

/// Counts the changes to `element`'s subtree while `enabled`, from each mount
/// until this component unmounts: an effect that reads it re-runs after one.
/// One observer per call, and it stays `0` where the renderer cannot watch.
pub(crate) fn use_content_changes(element: ElementHandle, enabled: bool) -> ReadSignal<u64> {
    // Bumped from the observer, which runs outside every scope.
    let changes = use_signal(|| 0u64);
    let slot: Rc<RefCell<Option<Box<dyn ContentSubscription>>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    use_drop({
        let slot = slot.clone();
        move || drop(slot.borrow_mut().take())
    });
    use_effect(use_reactive!(|enabled| {
        let _ = element.mount_token();
        // Dropped first, so a remount never runs two observers.
        slot.borrow_mut().take();
        if !enabled {
            return;
        }
        let watching = element.mounted().and_then(|mounted| {
            on_content_change(
                &mounted,
                Box::new(move || {
                    let mut changes = changes;
                    let next = changes.peek().wrapping_add(1);
                    changes.set(next);
                }),
            )
        });
        *slot.borrow_mut() = watching;
    }));
    changes.into()
}

/// The DOM `<form>` that owns `element`, while `enabled`: `Some` with the
/// `reset`s it has fired since, `None` without one. For a control inside a raw
/// `<form>`, which no libero `Form` context reaches. Stays `None` off the web.
pub(crate) fn use_form_owner(element: ElementHandle, enabled: bool) -> ReadSignal<Option<u32>> {
    // Bumped from the listener, which runs outside every scope.
    let owner = use_signal(|| None::<u32>);
    let slot: Rc<RefCell<Option<Box<dyn ContentSubscription>>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    use_drop({
        let slot = slot.clone();
        move || drop(slot.borrow_mut().take())
    });
    use_effect(use_reactive!(|enabled| {
        let _ = element.mount_token();
        slot.borrow_mut().take();
        let watching = enabled
            .then(|| element.mounted())
            .flatten()
            .and_then(|mounted| {
                on_form_reset(
                    &mounted,
                    Box::new(move || {
                        let mut owner = owner;
                        let next = owner.peek().unwrap_or(0).wrapping_add(1);
                        owner.set(Some(next));
                    }),
                )
            });
        // Kept across a remount, so a count read before it never repeats.
        let found = watching.is_some().then(|| owner.peek().unwrap_or(0));
        if *owner.peek() != found {
            let mut owner = owner;
            owner.set(found);
        }
        *slot.borrow_mut() = watching;
    }));
    owner.into()
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

    fn reset(&self) -> Result<(), PlatformError> {
        self.get()?.reset()
    }

    fn request_submit(&self) -> Result<(), PlatformError> {
        self.get()?.request_submit()
    }

    fn set_files(&self, files: &[dioxus::html::FileData]) -> Result<(), PlatformError> {
        self.get()?.set_files(files)
    }

    fn set_indeterminate(&self, indeterminate: bool) -> Result<(), PlatformError> {
        self.get()?.set_indeterminate(indeterminate)
    }

    fn set_value(&self, value: &str) -> Result<(), PlatformError> {
        self.get()?.set_value(value)
    }

    fn attribute(&self, name: &str) -> Result<Option<String>, PlatformError> {
        self.get()?.attribute(name)
    }

    fn selection_start(&self) -> Option<u32> {
        self.get().ok()?.selection_start()
    }

    fn previous_focusable(
        &self,
        selector: &str,
    ) -> Result<Option<Box<dyn ElementApi>>, PlatformError> {
        self.get()?.previous_focusable(selector)
    }

    fn is_focused(&self) -> bool {
        self.get().is_ok_and(|element| element.is_focused())
    }

    /// An unmounted handle answers `false`: there is no node, so there is
    /// nothing in the document. That differs from the mounted floor's `true`,
    /// which is about a node that exists and cannot be asked about.
    fn is_connected(&self) -> bool {
        self.get().is_ok_and(|element| element.is_connected())
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

    fn natural_size(&self) -> Read<Dimensions> {
        self.read(|element| element.natural_size())
    }

    fn computed_px(&self, property: &str) -> Read<Option<f64>> {
        self.read(|element| element.computed_px(property))
    }

    fn scroll_to(&self, x: f64, y: f64) -> Result<(), PlatformError> {
        self.get()?.scroll_to(x, y)
    }

    fn scroll_into_view(&self, smooth: bool) -> Result<(), PlatformError> {
        self.get()?.scroll_into_view(smooth)
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
