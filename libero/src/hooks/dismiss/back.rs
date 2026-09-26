//! Android's Back as a layer close (1275). wry finishes the activity on a Back
//! the WebView cannot go back from, so a history entry stands in for the open layers.

use std::cell::RefCell;
use std::rc::Rc;

use dioxus::core::provide_root_context;
use dioxus::prelude::*;

use crate::platform;

/// The layers Back closes, newest last, and whether the page holds the entry.
#[derive(Default)]
struct BackStack {
    layers: Vec<(u64, Callback<()>)>,
    next: u64,
    /// `None` until a layer first asks; `Some(false)` without a transport.
    listening: Option<bool>,
    armed: bool,
}

/// Root context, as the Escape stack.
#[derive(Clone, Copy)]
struct Back(CopyValue<BackStack>);

impl Back {
    fn get() -> Self {
        try_consume_context::<Back>().unwrap_or_else(|| {
            provide_root_context(Back(CopyValue::new_in_scope(
                BackStack::default(),
                ScopeId::ROOT,
            )))
        })
    }

    /// One entry while any layer is open: pushed for the first, taken back unheard after the last.
    fn sync(self) {
        let mut stack = self.0;
        let Ok(mut stack) = stack.try_write() else {
            return;
        };
        let want = !stack.layers.is_empty();
        if stack.listening == Some(true) && stack.armed != want {
            stack.armed = want;
            platform::back_entry(want);
        }
    }

    /// Back popped the entry: the top layer closes, and the entry returns while layers are left.
    fn pressed(self) {
        let mut stack = self.0;
        let top = stack.try_write().ok().and_then(|mut stack| {
            stack.armed = false;
            stack.layers.pop().map(|(_, onback)| onback)
        });
        if let Some(onback) = top {
            onback.call(());
        }
        self.sync();
    }

    fn push(self, onback: Callback<()>) -> Option<Guard> {
        let mut stack = self.0;
        let id = {
            let mut stack = stack.try_write().ok()?;
            if stack.listening.is_none() {
                stack.listening = Some(platform::watch_back(move || self.pressed()));
            }
            if stack.listening != Some(true) {
                return None;
            }
            stack.next += 1;
            let id = stack.next;
            stack.layers.push((id, onback));
            id
        };
        self.sync();
        Some(Guard { id, back: self })
    }
}

/// On the Back stack until dropped.
struct Guard {
    id: u64,
    back: Back,
}

impl Drop for Guard {
    fn drop(&mut self) {
        let (id, mut stack) = (self.id, self.back.0);
        if let Ok(mut stack) = stack.try_write() {
            stack.layers.retain(|(layer, _)| *layer != id);
        }
        self.back.sync();
    }
}

pub(super) fn use_back(open: bool, onback: Callback<()>) {
    let back = use_hook(Back::get);
    let guard: Rc<RefCell<Option<Guard>>> = use_hook(|| Rc::new(RefCell::new(None)));
    let slot = guard.clone();
    use_effect(use_reactive!(|(open,)| {
        if !open {
            slot.borrow_mut().take();
        } else if slot.borrow().is_none() {
            let pushed = back.push(onback);
            *slot.borrow_mut() = pushed;
        }
    }));
    use_drop(move || {
        guard.borrow_mut().take();
    });
}
