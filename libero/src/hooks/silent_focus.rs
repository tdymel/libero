use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use super::ElementHandle;
use crate::platform::{FocusMove, SilentFocusSubscription, silent_focus};

type OnMove = Rc<RefCell<Box<dyn Fn(&dyn FocusMove)>>>;

/// Calls `onmove` after each focus move that fired no `focusin`/`focusout`,
/// for as long as the component lives. Never where every move fires its
/// events (the web). The latest render's `onmove` is the one called.
pub(crate) fn use_silent_focus(onmove: impl Fn(&dyn FocusMove) + 'static) {
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

/// [`use_silent_focus`] as `element`'s own `focusin` (`true`) and `focusout`
/// (`false`): `onchange` hears only a move into or out of it.
pub(crate) fn use_silent_focus_within(element: ElementHandle, onchange: impl Fn(bool) + 'static) {
    use_silent_focus(move |moved| {
        let Some(mounted) = element.mounted() else {
            return;
        };
        let (was, is) = (moved.was_in(&mounted), moved.is_in(&mounted));
        if was != is {
            onchange(is);
        }
    });
}

/// Whether `element` held focus before `moved` and does not after: its `focusout`.
pub(crate) fn moved_out(moved: &dyn FocusMove, element: &ElementHandle) -> bool {
    moved_within(moved, element) == (true, false)
}

/// Whether `element` held focus before `moved`, and after it.
pub(crate) fn moved_within(moved: &dyn FocusMove, element: &ElementHandle) -> (bool, bool) {
    element.mounted().map_or((false, false), |mounted| {
        (moved.was_in(&mounted), moved.is_in(&mounted))
    })
}
