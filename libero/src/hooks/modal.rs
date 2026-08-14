use dioxus::{
    core::{DynamicNode, Properties, VComponent},
    prelude::*,
};

use crate::{components::Modal, hooks::use_portal};

pub(crate) const MODAL_BASE_Z_INDEX: i32 = 1000;
const MODAL_Z_INDEX_STEP: i32 = 10;

/// Hands out increasing z-indices so later-opened modals stack above
/// earlier ones. Provided by [`crate::LiberoProvider`].
#[derive(Clone, Copy)]
pub struct ModalHost {
    next_z_index: Signal<i32>,
}

impl ModalHost {
    pub(crate) fn new(next_z_index: Signal<i32>) -> Self {
        Self { next_z_index }
    }

    fn acquire_z_index(&mut self) -> i32 {
        let z_index = (self.next_z_index)();
        self.next_z_index.set(z_index + MODAL_Z_INDEX_STEP);
        z_index
    }
}

pub(crate) fn use_modal_z_index() -> i32 {
    let mut host = use_context::<ModalHost>();
    use_hook(|| host.acquire_z_index())
}

/// Lets a dialog rendered by [`use_modal`] close itself.
#[derive(Clone, Copy)]
pub struct ModalContext {
    pub(crate) onclose: EventHandler<()>,
}

impl ModalContext {
    /// Deferred to the next microtask: closing synchronously (from a click
    /// still bubbling through the modal being torn down) can re-enter the
    /// same `EventHandler` and panic with `AlreadyBorrowedMut`.
    pub fn close(&self) {
        let onclose = self.onclose;
        spawn(async move {
            onclose.call(());
        });
    }
}

pub fn use_modal_context() -> ModalContext {
    use_context::<ModalContext>()
}

/// Handle returned by [`use_modal`] - opens/closes a specific modal
/// component from anywhere, without the caller managing its own signal.
pub struct ModalHandle<P> {
    state: Signal<Option<P>>,
}

impl<P> Clone for ModalHandle<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P> Copy for ModalHandle<P> {}

impl<P: Clone + PartialEq + 'static> ModalHandle<P> {
    pub fn open(&self, props: P) {
        let mut state = self.state;
        state.set(Some(props));
    }

    pub fn close(&self) {
        let mut state = self.state;
        state.set(None);
    }

    pub fn is_open(&self) -> bool {
        self.state.read().is_some()
    }
}

impl<P: Default + Clone + PartialEq + 'static> ModalHandle<P> {
    /// Opens with default props - the ergonomic zero-argument `.open()` for
    /// dialogs with no meaningful props (`P = ()` included).
    pub fn open_default(&self) {
        self.open(P::default());
    }
}

/// Registers `component` as a modal that can be opened from anywhere via the
/// returned handle: `handle.open(props)`. Inside `component`, call
/// [`use_modal_context`] to get a `.close()` handle.
pub fn use_modal<P>(component: fn(P) -> Element) -> ModalHandle<P>
where
    P: Properties + Clone + PartialEq + 'static,
{
    let state = use_signal(|| None::<P>);
    let handle = ModalHandle { state };

    use_portal(move || {
        let props = state.read().clone()?;

        Some(rsx! {
            Modal {
                onclose: move |_| handle.close(),
                {DynamicNode::Component(VComponent::new(component, props.clone(), "ModalContent"))}
            }
        })
    });

    handle
}
