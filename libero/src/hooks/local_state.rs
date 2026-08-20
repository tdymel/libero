use std::cell::Cell;
use std::rc::Rc;

use dioxus::core::{Runtime, ScopeId, current_scope_id};
use dioxus::prelude::*;

/// Component state nothing outside the component reads.
///
/// Not `use_signal`: a signal's subscription bookkeeping costs ~290 ns on
/// every render, and none of it buys anything where the only reader is the
/// component that owns the state. This re-renders the owning scope by hand
/// instead.
///
/// Use `use_signal` the moment the value has to reach anywhere else.
pub(crate) fn use_local_state<T: Copy + 'static>(initial: impl FnOnce() -> T) -> LocalState<T> {
    use_hook(|| LocalState {
        value: Rc::new(Cell::new(initial())),
        scope: current_scope_id(),
    })
}

/// The handle [`use_local_state`] returns. `Clone`, not `Copy` - it holds an
/// `Rc` - so a component that both reads it and moves it into an event handler
/// reads first.
pub(crate) struct LocalState<T: Copy + 'static> {
    value: Rc<Cell<T>>,
    scope: ScopeId,
}

impl<T: Copy + 'static> LocalState<T> {
    pub fn get(&self) -> T {
        self.value.get()
    }

    /// Writes, then marks the owning scope dirty - unconditionally, like
    /// `Signal::set`, so an equal value still re-renders.
    ///
    /// Through the runtime rather than `needs_update_any`, which reads the
    /// *current* scope and so needs one on the stack.
    pub fn set(&self, value: T) {
        self.value.set(value);
        Runtime::current().needs_update(self.scope);
    }
}

impl<T: Copy + 'static> Clone for LocalState<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            scope: self.scope,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    thread_local! {
        /// The rendered component's handle, so the test can write from outside.
        static HANDLE: RefCell<Option<LocalState<u32>>> = const { RefCell::new(None) };
    }

    #[component]
    fn Counter() -> Element {
        let count = use_local_state(|| 0u32);
        HANDLE.with(|handle| *handle.borrow_mut() = Some(count.clone()));

        rsx! { span { "{count.get()}" } }
    }

    /// The whole point of the hook: a write re-renders the owning scope even
    /// though nothing ever subscribed to the value.
    #[test]
    fn a_write_re_renders_the_owning_scope() {
        let mut dom = VirtualDom::new(Counter);
        dom.rebuild_in_place();
        assert!(dioxus_ssr::render(&dom).contains(">0<"));

        let handle = HANDLE
            .with(|handle| handle.borrow().clone())
            .expect("the component rendered");
        dom.in_runtime(|| handle.set(5));
        dom.render_immediate(&mut dioxus::core::NoOpMutations);

        assert!(dioxus_ssr::render(&dom).contains(">5<"));
    }
}
