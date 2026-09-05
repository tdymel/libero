use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

use super::window::z_index_at;

static NEXT_MODAL_ID: AtomicU64 = AtomicU64::new(0);

/// Stacks the open modals: the last one opened is on top. Provided by
/// [`crate::LiberoProvider`].
///
/// A stack of ids rather than a counter, like [`super::WindowHost`]: a
/// counter can only give back its top index, so every modal closed below
/// another one leaked a step for the session, and enough of them climbed
/// past `popover`. Derived from the open stack, the indices stay a dense run
/// `modal, modal + step, ..` whatever order modals close in. The top is
/// capped below `popover`, so a dropdown opened inside a modal clears it
/// even with more modals open than the gap holds; the ones past the cap tie.
#[derive(Clone, Copy)]
pub(crate) struct ModalHost {
    stack: Signal<Vec<u64>>,
    base: i32,
    step: i32,
    ceiling: i32,
}

impl ModalHost {
    pub(crate) fn new(stack: Signal<Vec<u64>>, base: i32, step: i32, ceiling: i32) -> Self {
        Self {
            stack,
            base,
            step,
            ceiling,
        }
    }

    /// Puts a new modal on top and returns its id.
    pub(crate) fn open(&self) -> u64 {
        let id = NEXT_MODAL_ID.fetch_add(1, Ordering::Relaxed);
        let mut stack = self.stack;
        stack.write().push(id);
        id
    }

    pub(crate) fn close(&self, id: u64) {
        let mut stack = self.stack;
        stack.write().retain(|other| *other != id);
    }

    /// `id`'s z-index, subscribed: a modal closing below this one moves it
    /// down a step. `base` for an id that is not stacked.
    pub(crate) fn z_index(&self, id: u64) -> i32 {
        let position = self
            .stack
            .read()
            .iter()
            .position(|other| *other == id)
            .unwrap_or(0);
        z_index_at(self.base, self.step, self.ceiling, position)
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

#[cfg(test)]
mod tests {
    use dioxus::dioxus_core::VirtualDom;
    use dioxus::prelude::*;

    use super::ModalHost;

    /// Runs `check` against a fresh host inside a scope, which `Signal` needs.
    fn with_host(check: fn(ModalHost)) {
        let mut dom = VirtualDom::new_with_props(
            |check: fn(ModalHost)| {
                let stack = use_signal(Vec::new);
                use_hook(|| check(ModalHost::new(stack, 1000, 10, 2000)));
                rsx! {}
            },
            check,
        );
        dom.rebuild_in_place();
    }

    #[test]
    fn a_modal_above_another_steps_once() {
        with_host(|host| {
            let a = host.open();
            let b = host.open();
            assert_eq!(host.z_index(a), 1000);
            assert_eq!(host.z_index(b), 1010);
        });
    }

    /// The leak: a counter that only gives back its top index lost a step to
    /// every modal closed below another one.
    #[test]
    fn closing_out_of_order_leaks_nothing() {
        with_host(|host| {
            for _ in 0..500 {
                let a = host.open();
                let b = host.open();
                host.close(a);
                assert_eq!(host.z_index(b), 1000, "b is alone, so it is the base");
                host.close(b);
            }
            let next = host.open();
            assert_eq!(host.z_index(next), 1000);
        });
    }

    /// Opening and closing while another modal stays up never climbs either.
    #[test]
    fn interleaved_modals_stay_a_dense_run() {
        with_host(|host| {
            let mut below = host.open();
            for _ in 0..500 {
                let above = host.open();
                assert_eq!(host.z_index(above), 1010);
                host.close(below);
                below = above;
            }
        });
    }

    #[test]
    fn the_top_is_capped_below_popover() {
        with_host(|host| {
            let ids: Vec<u64> = (0..200).map(|_| host.open()).collect();
            assert_eq!(host.z_index(ids[99]), 1990);
            assert_eq!(host.z_index(ids[199]), 1999);
        });
    }
}
