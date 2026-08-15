use dioxus::{
    core::{DynamicNode, Properties, VComponent},
    prelude::*,
};

use crate::{
    components::Modal,
    context::{ModalContext, ModalHost},
    hooks::use_portal,
};

pub(crate) fn use_modal_z_index() -> i32 {
    let mut host = use_context::<ModalHost>();
    use_hook(|| host.acquire_z_index())
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
/// [`use_modal_context`] to get a `.close()` handle, and wrap its content in
/// [`crate::components::Dialog`] to get `role="dialog"`/`aria-modal`.
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
