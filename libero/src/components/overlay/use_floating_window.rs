use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::overlay::{FloatingWindow, FloatingWindowOptions},
    hooks::{FocusReturn, use_focus_return, use_portal},
    platform::{ElementApi, document},
};

/// Opens and closes one window. `Copy`, so a trigger anywhere can hold it.
#[derive(Clone, Copy)]
pub struct FloatingWindowHandle {
    open: Signal<bool>,
    focus_return: FocusReturn,
    /// The open window's "has focus left me?", asked in its own scope.
    focus_left: Signal<Option<Callback<(), bool>>>,
    /// Where the page had focus, for F6 to go back to.
    page: CopyValue<Option<Rc<dyn ElementApi>>>,
}

impl FloatingWindowHandle {
    /// Shows the window, remembering where focus was for the close.
    pub fn open(&self) {
        if *self.open.peek() {
            return;
        }
        self.focus_return.remember_active();
        let mut page = self.page;
        page.set(
            document()
                .and_then(|document| document.active_element())
                .map(Rc::from),
        );
        let mut focus_left = self.focus_left;
        focus_left.set(None);
        let mut open = self.open;
        open.set(true);
    }

    /// Hides the window and returns focus to the opener, unless focus already
    /// moved to the page or another window.
    pub fn close(&self) {
        if !*self.open.peek() {
            return;
        }
        let elsewhere = (*self.focus_left.peek()).is_some_and(|left| left.call(()));
        let mut open = self.open;
        open.set(false);
        if !elsewhere {
            self.focus_return.restore();
        }
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

/// A non-modal window over the page, draggable and optionally resizable.
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
/// Call it in a component that outlives every trigger: the window portals from there.
///
/// Docs: <https://libero-ui.dev/overlay/floating-window>
pub fn use_floating_window(
    options: FloatingWindowOptions,
    render: impl FnMut(FloatingWindowHandle) -> Element + 'static,
) -> FloatingWindowHandle {
    let open = use_signal(|| false);
    let focus_return = use_focus_return();
    let mut focus_left = use_signal(|| None);
    let page = use_hook(|| CopyValue::new(None));
    let handle = FloatingWindowHandle {
        open,
        focus_return,
        focus_left,
        page,
    };
    // `use_callback`, so a closure rebuilt each render keeps its captures live.
    let render = use_callback(render);
    let onclose = use_callback(move |()| handle.close());
    let onmount = use_callback(move |left| focus_left.set(Some(left)));
    let opener = use_callback(move |()| page.peek().clone());

    let content = open().then(|| {
        rsx! {
            FloatingWindow { options, onclose, onmount, opener, {render.call(handle)} }
        }
    });
    use_portal(content);

    handle
}
