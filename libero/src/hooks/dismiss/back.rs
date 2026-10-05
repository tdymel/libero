//! Android's Back as a layer close (1275). wry finishes the activity on a Back
//! the WebView cannot go back from, so a history entry stands in for each open layer.

use std::cell::RefCell;
use std::rc::Rc;

use dioxus::core::provide_root_context;
use dioxus::prelude::*;

use crate::platform;

/// The layers Back closes, newest last, and the entry count the page was asked for.
#[derive(Default)]
struct BackStack {
    layers: Vec<(u64, Callback<()>)>,
    next: u64,
    /// `None` until a layer first asks; `Some(false)` without a transport.
    listening: Option<bool>,
    sent: usize,
    /// Backs since, whose layers may not have closed yet.
    popped: usize,
}

impl BackStack {
    /// One entry per open layer: the count to send, if it changed. A closed
    /// layer settles a Back; a new one starts afresh (its tap paid what was owed).
    fn sync(&mut self) -> Option<usize> {
        let want = self.layers.len();
        (self.listening == Some(true) && self.sent != want).then(|| {
            self.popped = match want < self.sent {
                true => self.popped.saturating_sub(self.sent - want),
                false => 0,
            };
            self.sent = want;
            want
        })
    }

    /// Back popped the newest entry left: its layer, or the top one when none is
    /// left. Rust's view, so a layer whose entry is still on its way counts.
    fn pressed(&mut self) -> Option<Callback<()>> {
        let top = self.layers.len().checked_sub(1)?;
        let left = self.sent.saturating_sub(self.popped);
        self.popped += 1;
        Some(self.layers[left.saturating_sub(1).min(top)].1)
    }
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

    /// Pushed while a layer opens within its tap, taken back unheard as layers close.
    fn sync(self) {
        let mut stack = self.0;
        let change = stack.try_write().ok().and_then(|mut stack| stack.sync());
        if let Some(count) = change {
            platform::back_entries(count);
        }
    }

    /// No sync here: the layer is still open, and a push now has no activation.
    /// One kept open gets its entry back at the next tap.
    fn pressed(self) {
        let mut stack = self.0;
        let layer = stack.try_write().ok().and_then(|mut stack| stack.pressed());
        if let Some(onback) = layer {
            onback.call(());
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(layers: Vec<(u64, Callback<()>)>) -> BackStack {
        BackStack {
            next: layers.len() as u64,
            layers,
            listening: Some(true),
            ..BackStack::default()
        }
    }

    #[test]
    fn each_layer_holds_an_entry_and_back_closes_the_newest() {
        let mut dom = VirtualDom::new(|| rsx! {});
        dom.rebuild_in_place();
        dom.in_scope(ScopeId::APP, || {
            let (lower, upper) = (Callback::new(|()| {}), Callback::new(|()| {}));
            let mut stack = stack(vec![(1, lower)]);
            assert_eq!(stack.sync(), Some(1));
            stack.layers.push((2, upper));
            assert_eq!(stack.sync(), Some(2));
            assert_eq!(stack.pressed(), Some(upper));
            assert_eq!(stack.sync(), None, "Back asks for no entry back");
            assert_eq!(
                stack.pressed(),
                Some(lower),
                "a second Back before upper closed"
            );
            stack.layers.retain(|(id, _)| *id != 2);
            assert_eq!(stack.sync(), Some(1));
            stack.layers.clear();
            assert_eq!(stack.sync(), Some(0));
            assert_eq!(stack.pressed(), None);
        });
    }

    #[test]
    fn a_layer_kept_open_takes_the_next_back_too() {
        let mut dom = VirtualDom::new(|| rsx! {});
        dom.rebuild_in_place();
        dom.in_scope(ScopeId::APP, || {
            let (wizard, menu) = (Callback::new(|()| {}), Callback::new(|()| {}));
            let mut stack = stack(vec![(1, wizard)]);
            stack.sync();
            assert_eq!(stack.pressed(), Some(wizard));
            assert_eq!(stack.pressed(), Some(wizard), "still open, still on top");
            stack.layers.push((2, menu));
            stack.sync();
            assert_eq!(stack.pressed(), Some(menu), "a new layer starts afresh");
        });
    }
}
