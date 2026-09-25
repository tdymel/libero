use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    context::WindowHost,
    hooks::{ElementHandle, use_dismiss_layer},
    platform::{ElementApi, KeyChord, KeySubscription, keyboard},
};

/// Whether focus is known to be on `root` or inside it.
fn focus_within(root: &ElementHandle) -> bool {
    root.is_focused() || root.query_selector(":focus").is_ok()
}

fn active_element() -> Option<Rc<dyn ElementApi>> {
    crate::platform::document()
        .and_then(|document| document.active_element())
        .map(Rc::from)
}

/// F6 moves focus between the page and the topmost window, which Tab reaches
/// last (todo 572). Not while a modal or popover is open.
pub(super) fn use_page_switch(
    root: ElementHandle,
    host: WindowHost,
    id: u64,
    opener: Callback<(), Option<Rc<dyn ElementApi>>>,
) {
    let layer = use_dismiss_layer();
    // Where the page had focus: the opener at first, then wherever F6 left.
    let mut page = use_hook(|| CopyValue::new(opener.call(())));
    // The key callback runs outside every scope on the web; an effect moves focus.
    let tick = use_signal(|| 0u64);
    let slot: Rc<RefCell<Option<Box<dyn KeySubscription>>>> = use_hook(|| {
        let callback = Box::new(move |chord: KeyChord| {
            let modifiers = chord.modifiers;
            // Shift+F6 too: with two stops, backward is forward.
            if chord.key != Key::F6
                || modifiers.ctrl()
                || modifiers.alt()
                || modifiers.meta()
                || !host.is_top(id)
                || layer.any_open()
            {
                return false;
            }
            if !chord.repeat {
                let mut tick = tick;
                let next = tick.peek().wrapping_add(1);
                tick.set(next);
            }
            true
        });
        Rc::new(RefCell::new(
            keyboard().map(|api| api.on_key_unfiltered(callback)),
        ))
    });
    use_drop(move || {
        slot.borrow_mut().take();
    });

    let mut seen = use_signal(|| 0u64);
    use_effect(move || {
        let pressed = tick();
        if pressed == *seen.peek() {
            return;
        }
        seen.set(pressed);
        if focus_within(&root) {
            let target = page.peek().clone().filter(|target| target.is_connected());
            if let Some(target) = target {
                let _ = target.focus();
            }
        } else {
            page.set(active_element());
            let _ = root.focus();
        }
    });
}
