use dioxus::prelude::*;

/// Hands out increasing z-indices so later-opened modals stack above
/// earlier ones. Provided by [`crate::LiberoProvider`].
#[derive(Clone, Copy)]
pub struct ModalHost {
    next_z_index: Signal<i32>,
    step: i32,
}

impl ModalHost {
    pub(crate) fn new(next_z_index: Signal<i32>, step: i32) -> Self {
        Self { next_z_index, step }
    }

    pub(crate) fn acquire_z_index(&mut self) -> i32 {
        let z_index = (self.next_z_index)();
        self.next_z_index.set(z_index + self.step);
        z_index
    }

    /// Only the top of the stack can be given back - anything below it is
    /// still spanned by a modal above, so its index stays spent.
    pub(crate) fn release_z_index(&mut self, z_index: i32) {
        if (self.next_z_index)() == z_index + self.step {
            self.next_z_index.set(z_index);
        }
    }
}

/// Lets a dialog rendered by [`crate::hooks::use_modal`] close itself.
#[derive(Clone, Copy)]
pub struct ModalContext {
    pub(crate) onclose: EventHandler<()>,
}

impl ModalContext {
    /// Deferred a microtask: closing synchronously from a click still
    /// bubbling through the torn-down modal re-enters the same
    /// `EventHandler` and panics with `AlreadyBorrowedMut`.
    pub fn close(&self) {
        let onclose = self.onclose;
        spawn(async move {
            onclose.call(());
        });
    }
}
