use dioxus::prelude::*;

use crate::{
    components::{FloatingWindow, FloatingWindowOptions},
    hooks::{FocusReturn, use_focus_return, use_portal},
};

/// Opens and closes one window. `Copy`, so a trigger anywhere can hold it.
#[derive(Clone, Copy)]
pub struct FloatingWindowHandle {
    open: Signal<bool>,
    focus_return: FocusReturn,
}

impl FloatingWindowHandle {
    /// Shows the window, remembering where focus was so closing can put it
    /// back. Opening an open window does nothing.
    pub fn open(&self) {
        if *self.open.peek() {
            return;
        }
        self.focus_return.remember_active();
        let mut open = self.open;
        open.set(true);
    }

    /// Hides the window and returns focus to whatever opened it.
    pub fn close(&self) {
        if !*self.open.peek() {
            return;
        }
        let mut open = self.open;
        open.set(false);
        self.focus_return.restore();
    }

    pub fn toggle(&self) {
        if *self.open.peek() {
            self.close();
        } else {
            self.open();
        }
    }

    pub fn is_open(&self) -> bool {
        (self.open)()
    }
}

/// A non-modal window over the page: a title bar that drags and moves by
/// keyboard, an optional corner resize handle, Escape and a close button.
/// The hook owns whether it exists, so no caller holds an open flag; the
/// window owns where it is and how big.
///
/// It is not a modal: no focus trap, no overlay, and the page stays usable.
/// Focus moves into the window on open and back to the trigger on close.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{Button, FloatingWindowOptions};
/// # use libero::hooks::use_floating_window;
/// # fn app() -> Element {
/// let inspector = use_floating_window(
///     FloatingWindowOptions { title: Some("Inspector".into()), resizable: true, ..Default::default() },
///     |window| rsx! { InspectorBody {} },
/// );
/// rsx! { Button { onclick: move |_| inspector.toggle(), "Inspector" } }
/// # }
/// # #[component] fn InspectorBody() -> Element { rsx! {} }
/// ```
///
/// Call it under `LiberoProvider`, in a component that outlives every
/// trigger: the window is portaled from there.
pub fn use_floating_window(
    options: FloatingWindowOptions,
    render: impl FnMut(FloatingWindowHandle) -> Element + 'static,
) -> FloatingWindowHandle {
    let open = use_signal(|| false);
    let focus_return = use_focus_return();
    let handle = FloatingWindowHandle { open, focus_return };
    // `use_callback`, so a closure rebuilt each render keeps its captures live.
    let render = use_callback(render);
    let onclose = use_callback(move |()| handle.close());

    let content = open().then(|| {
        rsx! {
            FloatingWindow { options, onclose, {render.call(handle)} }
        }
    });
    use_portal(content);

    handle
}
