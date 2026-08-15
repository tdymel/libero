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

/// Opens/closes a modal registered via [`use_modal`] from anywhere.
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
    /// Opens with default props - for dialogs with no meaningful props.
    pub fn open_default(&self) {
        self.open(P::default());
    }
}

/// Registers `component` as a modal, opened from anywhere via
/// `handle.open(props)`. Inside `component`, use [`use_modal_context`] to
/// close it and [`crate::components::Dialog`] for its a11y roles.
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
