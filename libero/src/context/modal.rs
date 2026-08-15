use dioxus::prelude::*;

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

    pub(crate) fn acquire_z_index(&mut self) -> i32 {
        let z_index = (self.next_z_index)();
        self.next_z_index.set(z_index + MODAL_Z_INDEX_STEP);
        z_index
    }
}

/// Lets a dialog rendered by [`crate::hooks::use_modal`] close itself.
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
