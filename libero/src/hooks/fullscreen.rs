use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::hooks::ElementHandle;
use crate::platform::{self, FullscreenApi, FullscreenSubscription};

/// Puts one element in fullscreen: the Fullscreen API where the page has it and
/// grants it, else a pseudo-fullscreen the caller draws (a fixed box over the page).
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct FullscreenHandle {
    element: ElementHandle,
    available: Signal<bool>,
    active: Signal<bool>,
    pseudo: Signal<bool>,
}

impl FullscreenHandle {
    fn api(&self) -> Option<Box<dyn FullscreenApi>> {
        platform::fullscreen(&self.element.mounted()?, self.element.tag())
    }

    /// Native or pseudo; reactive.
    pub(crate) fn is_fullscreen(&self) -> bool {
        (self.active)() || (self.pseudo)()
    }

    /// Only the pseudo-fullscreen, which the caller draws; reactive.
    pub(crate) fn is_pseudo(&self) -> bool {
        (self.pseudo)()
    }

    pub(crate) fn toggle(&self) {
        let mut pseudo = self.pseudo;
        if *self.active.peek() {
            if let Some(api) = self.api() {
                let _ = api.exit();
            }
        } else if *pseudo.peek() {
            pseudo.set(false);
        } else if let Some(api) = self.api().filter(|_| *self.available.peek()) {
            // Refused (a headless browser, a WebView without fullscreen): draw it ourselves.
            spawn(async move {
                if api.enter().await.is_err() {
                    pseudo.set(true);
                }
            });
        } else {
            pseudo.set(true);
        }
    }

    /// Leaves the pseudo-fullscreen; native fullscreen leaves on Escape by itself.
    pub(crate) fn exit_pseudo(&self) {
        let mut pseudo = self.pseudo;
        if *pseudo.peek() {
            pseudo.set(false);
        }
    }
}

/// `element` is the box that goes fullscreen; spread its attributes and mount it.
pub(crate) fn use_fullscreen(element: ElementHandle) -> FullscreenHandle {
    let handle = FullscreenHandle {
        element,
        available: use_signal(|| false),
        active: use_signal(|| false),
        pseudo: use_signal(|| false),
    };
    let slot: Rc<RefCell<Option<Box<dyn FullscreenSubscription>>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    use_drop({
        let slot = slot.clone();
        move || drop(slot.borrow_mut().take())
    });
    use_effect(move || {
        let Some(_) = element.mount_token() else {
            return;
        };
        slot.borrow_mut().take();
        let (available, active) = (handle.available, handle.active);
        *slot.borrow_mut() = handle.api().map(|api| {
            api.watch(Box::new(move |state| {
                let (mut available, mut active) = (available, active);
                if *available.peek() != state.available {
                    available.set(state.available);
                }
                if *active.peek() != state.active {
                    active.set(state.active);
                }
            }))
        });
    });
    handle
}
