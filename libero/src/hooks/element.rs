use std::{cell::RefCell, rc::Rc};

use dioxus::core::{Attribute, AttributeValue};
use dioxus::prelude::*;

use crate::platform::{
    self, ContentSubscription, Dimensions, ElementApi, OBSERVE_ATTR, PlatformError, Read, is_rtl,
    next_observe_tag, observes_by_tag, on_content_change, on_form_reset, on_resize,
};

/// A handle to one of this component's own elements, and the only way to reach
/// an element at all.
///
/// It implements [`ElementApi`] on the richest backing the renderer offers: a
/// webview measures and focuses but has no subtree queries. Every call answers
/// [`PlatformError::Unsupported`] until the element is mounted.
#[derive(Clone, Copy, PartialEq)]
pub struct ElementHandle {
    mounted: Signal<Option<Rc<MountedData>>>,
    /// What a WebView finds the element by, see [`attributes`](Self::attributes).
    tag: Option<u64>,
}

impl ElementHandle {
    /// Spread on the element (`..handle.attributes()`) where a WebView has to find
    /// it, as `use_intersection`'s `root`. Empty on the web and Blitz, and on a
    /// handle made by a component rather than `use_element`.
    pub fn attributes(&self) -> Vec<Attribute> {
        self.tag
            .map(|tag| {
                Attribute::new(
                    OBSERVE_ATTR,
                    AttributeValue::Text(tag.to_string()),
                    None,
                    false,
                )
            })
            .into_iter()
            .collect()
    }

    pub(crate) fn tag(&self) -> Option<u64> {
        self.tag
    }

    /// The `onmounted` handler that fills this handle in. Prefer
    /// `.element(&handle)` on the `Box` builder, which wires this up.
    pub fn mount(mut self) -> impl FnMut(Event<MountedData>) + 'static {
        move |event| self.mounted.set(Some(event.data()))
    }

    /// Reactive: an effect that reads this re-runs once the element mounts.
    pub fn is_mounted(&self) -> bool {
        self.mounted.read().is_some()
    }

    /// Identifies *which* mount this is. A remount (a portaled dropdown
    /// reopening) leaves [`is_mounted`](Self::is_mounted) unchanged; this changes.
    pub(crate) fn mount_token(&self) -> Option<usize> {
        self.mounted
            .read()
            .as_ref()
            .map(|mounted| Rc::as_ptr(mounted) as *const () as usize)
    }

    /// A handle owned by the current scope, outliving the element's component
    /// (a `FormHandle` a parent holds). Not a hook.
    pub(crate) fn new() -> Self {
        Self {
            mounted: Signal::new(None),
            tag: None,
        }
    }

    /// A handle owned by `owner`, for one made in a child's render that an
    /// ancestor of the child has to read - `Menu`'s submenu boxes.
    pub(crate) fn new_in_scope(owner: ScopeId) -> Self {
        Self {
            mounted: Signal::new_in_scope(None, owner),
            tag: None,
        }
    }

    /// The mounted element, for a platform call that needs the node itself.
    pub(crate) fn mounted(&self) -> Option<Rc<MountedData>> {
        self.mounted.peek().clone()
    }

    /// [`mounted`](Self::mounted) that answers `None` once the owner dropped,
    /// for a listener that may outlive it.
    pub(crate) fn try_mounted(&self) -> Option<Rc<MountedData>> {
        self.mounted.try_peek().ok()?.clone()
    }

    /// Whether the element lays out right to left; `false` until it mounts.
    /// Not reactive, so a key handler can ask before it returns.
    pub(crate) fn is_rtl(&self) -> bool {
        self.mounted.peek().as_ref().is_some_and(is_rtl)
    }

    fn get(&self) -> Result<Box<dyn ElementApi>, PlatformError> {
        match self.mounted.read().as_ref() {
            Some(mounted) => Ok(platform::element(mounted)),
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

/// A handle to one of this component's own elements.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::use_element;
/// # use libero::platform::ElementApi;
/// # fn app() -> Element {
/// let input = use_element();
///
/// rsx! {
///     input { onmounted: input.mount() }
///     button { onclick: move |_| { let _ = input.focus(); }, "Focus the input" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/hooks/use-element>
pub fn use_element() -> ElementHandle {
    ElementHandle {
        mounted: use_signal(|| None),
        tag: use_hook(|| observes_by_tag().then(next_observe_tag)),
    }
}

/// Counts the changes to `element`'s subtree while `enabled`, so an effect
/// reading it re-runs. Stays `0` where the renderer cannot watch.
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

/// Runs `onresize` where the renderer fires no `resize` (Blitz), from sizes it
/// measures itself; elsewhere it does nothing. Give the element the same
/// handler with `.event("onresize", ..)`.
pub(crate) fn use_resize_fallback(
    element: ElementHandle,
    onresize: impl FnMut(Event<ResizeData>) + 'static,
) {
    type Handler = Rc<RefCell<Box<dyn FnMut(Event<ResizeData>)>>>;
    // This render's handler: props it captured may have changed.
    let handler: Handler = use_hook(|| {
        let noop: Box<dyn FnMut(Event<ResizeData>)> = Box::new(|_| {});
        Rc::new(RefCell::new(noop))
    });
    *handler.borrow_mut() = Box::new(onresize);
    let slot: Rc<RefCell<Option<Box<dyn ContentSubscription>>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    use_drop({
        let slot = slot.clone();
        move || drop(slot.borrow_mut().take())
    });
    use_effect(move || {
        let _ = element.mount_token();
        slot.borrow_mut().take();
        let handler = handler.clone();
        let watching = element.mounted().and_then(|mounted| {
            on_resize(
                &mounted,
                Box::new(move |event| (handler.borrow_mut())(event)),
            )
        });
        *slot.borrow_mut() = watching;
    });
}

/// The raw DOM `<form>` owning `element`, while `enabled`: `Some` with its
/// `reset` count, `None` without one. Stays `None` off the web.
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

    /// `false` unmounted: no node. The mounted floor's `true` means a node that
    /// exists but cannot be asked.
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
