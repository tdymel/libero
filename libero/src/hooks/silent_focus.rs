use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use super::ElementHandle;
use crate::platform::{FocusMove, SilentFocusSubscription, silent_focus};

type OnMove = Rc<RefCell<Box<dyn Fn(&dyn FocusMove)>>>;

/// Calls `onmove` after each focus move that fired no `focusin`/`focusout`,
/// for as long as the component lives. Never where every move fires its
/// events (the web). The latest render's `onmove` is the one called.
fn use_silent_focus(onmove: impl Fn(&dyn FocusMove) + 'static) {
    // Fixed per build, so the hook order holds; the web pays no hook slot.
    if silent_focus().is_none() {
        return;
    }
    let slot = use_hook(|| {
        let current: OnMove = Rc::new(RefCell::new(Box::new(|_| {})));
        let subscription = silent_focus().map(|api| {
            let current = current.clone();
            api.on_move(Box::new(move |moved| (current.borrow())(moved)))
        });
        Rc::new((current, subscription))
    });
    let (current, subscription): &(OnMove, Option<Box<dyn SilentFocusSubscription>>) = &slot;
    if subscription.is_some() {
        *current.borrow_mut() = Box::new(onmove);
    }
}

/// An element whose silent `focusout` calls `onout`, for the caller to mount.
/// `None` where every move fires its events, so the web mounts nothing.
pub(crate) fn use_silent_focus_out(onout: impl Fn() + 'static) -> Option<ElementHandle> {
    let element = use_hook(|| silent_focus().is_some().then(ElementHandle::new));
    use_silent_focus(move |moved| {
        if element.is_some_and(|element| moved_out(moved, &element)) {
            onout();
        }
    });
    element
}

/// Whether `element` held focus before `moved` and does not after: its `focusout`.
fn moved_out(moved: &dyn FocusMove, element: &ElementHandle) -> bool {
    element
        .mounted()
        .is_some_and(|mounted| moved.was_in(&mounted) && !moved.is_in(&mounted))
}
