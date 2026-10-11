//! Android's Back as a layer close (1275). wry finishes the activity on a Back
//! the WebView cannot go back from, so a history entry stands in for each open layer.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use dioxus::core::{provide_root_context, spawn_forever};
use dioxus::prelude::*;

use crate::platform::{self, TimerSubscription};

/// Entries one layer may bank from taps, for an `onback` that keeps it open (3322).
const MAX_ENTRIES: usize = 16;

/// How long the spare waits for a layer; the docs' route layer came ~180 ms after its search closed.
const SPARE_FOR: Duration = Duration::from_millis(600);

struct Layer {
    id: u64,
    onback: Callback<()>,
    /// History entries it owns: one from its opening tap, one more per tap while on top.
    entries: usize,
}

/// The layers Back closes, newest last, and the entry count the page holds.
#[derive(Default)]
struct BackStack {
    layers: Vec<Layer>,
    next: u64,
    /// `None` until a layer first asks; `Some(false)` without a transport.
    listening: Option<bool>,
    /// Rust's view: Backs already counted, asks maybe still on their way.
    held: usize,
    /// The last tap banked an entry a layer opened by that tap takes over: Chromium
    /// honours one push per activation, a second marks the first skippable (2240).
    fresh: bool,
    /// The last layer closed with a fresh entry: kept a moment for a layer the same
    /// tap opens later (a search result's route), which could not push its own.
    spare: Option<Box<dyn TimerSubscription>>,
}

impl BackStack {
    /// Hands the last tap's banked entry to a layer that tap opens.
    fn take_fresh(&mut self) {
        if std::mem::take(&mut self.fresh)
            && let Some(top) = self.layers.last_mut()
        {
            top.entries = top.entries.saturating_sub(1);
        }
    }

    /// The entries the open layers own: the count to send, if it changed.
    fn sync(&mut self) -> Option<usize> {
        let want = self.layers.iter().map(|layer| layer.entries).sum::<usize>()
            + usize::from(self.spare.is_some());
        (self.listening == Some(true) && self.held != want).then(|| {
            self.held = want;
            want
        })
    }

    /// Drops layer `id`. A tap that banked on it and closes it (a drawer's link) leaves
    /// that entry to the layer below: the route history stepped on (3323).
    /// Without one, whether the entry is kept as the spare.
    fn close(&mut self, id: u64) -> bool {
        self.layers.retain(|layer| layer.id != id);
        let fresh = std::mem::take(&mut self.fresh);
        match self.layers.last_mut() {
            Some(top) if fresh && top.entries < MAX_ENTRIES => {
                top.entries += 1;
                self.fresh = true;
                false
            }
            None => fresh,
            Some(_) => false,
        }
    }

    /// A tap or key, the activation one more entry needs: the top layer banks it.
    fn tapped(&mut self) -> Option<usize> {
        let top = self.layers.last_mut()?;
        self.fresh = top.entries < MAX_ENTRIES;
        if !self.fresh {
            return None;
        }
        top.entries += 1;
        self.sync()
    }

    /// Back popped the newest entry: its owner, entries counted from the bottom,
    /// or the top layer past them. A layer that has not closed yet keeps the rest.
    fn pressed(&mut self) -> Option<Callback<()>> {
        self.fresh = false;
        let entry = self.held;
        if self.layers.is_empty() {
            // Only the spare to pop: the page stays.
            if self.spare.take().is_some() {
                self.held = self.held.saturating_sub(1);
            }
            return None;
        }
        let top = self.layers.len() - 1;
        self.held = self.held.saturating_sub(1);
        let mut below = 0;
        let owner = match entry {
            0 => top,
            _ => self
                .layers
                .iter()
                .position(|layer| {
                    below += layer.entries;
                    below >= entry
                })
                .unwrap_or(top),
        };
        let layer = &mut self.layers[owner];
        layer.entries = layer.entries.saturating_sub(1);
        Some(layer.onback)
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
    /// One kept open lives on the entries its taps banked.
    fn pressed(self) {
        let mut stack = self.0;
        let layer = stack.try_write().ok().and_then(|mut stack| stack.pressed());
        if let Some(onback) = layer {
            onback.call(());
        }
    }

    /// Pushed at once, inside the tap's activation the page reported.
    fn tapped(self) {
        let mut stack = self.0;
        let change = stack.try_write().ok().and_then(|mut stack| stack.tapped());
        if let Some(count) = change {
            platform::back_entries(count);
        }
    }

    /// Starts the page's listener on the first ask; whether Back reaches the page.
    fn listen(self, stack: &mut BackStack) -> bool {
        if stack.listening.is_none() {
            stack.listening = Some(platform::watch_back(move |back| match back {
                true => self.pressed(),
                false => self.tapped(),
            }));
        }
        stack.listening == Some(true)
    }

    /// One entry more than the open layers hold, pushed now: the layer's own `push` finds it sent.
    fn claim(self) {
        let mut stack = self.0;
        let want = {
            let Ok(mut stack) = stack.try_write() else {
                return;
            };
            if !self.listen(&mut stack) {
                return;
            }
            stack.take_fresh();
            let want = stack
                .layers
                .iter()
                .map(|layer| layer.entries)
                .sum::<usize>()
                + 1;
            if stack.held >= want {
                return;
            }
            stack.held = want;
            want
        };
        platform::back_entries(want);
    }

    fn push(self, onback: Callback<()>) -> Option<Guard> {
        let mut stack = self.0;
        let id = {
            let mut stack = stack.try_write().ok()?;
            if !self.listen(&mut stack) {
                return None;
            }
            stack.take_fresh();
            // Takes the spare's place: no push.
            stack.spare = None;
            stack.next += 1;
            let id = stack.next;
            stack.layers.push(Layer {
                id,
                onback,
                entries: 1,
            });
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
        let (id, back, mut stack) = (self.id, self.back, self.back.0);
        if let Ok(mut stack) = stack.try_write()
            && stack.close(id)
        {
            let mut slot = back.0;
            stack.spare = platform::timer().map(|timer| {
                timer.after(
                    SPARE_FOR,
                    Box::new(move || {
                        if let Ok(mut stack) = slot.try_write() {
                            stack.spare = None;
                        }
                        back.sync();
                    }),
                )
            });
        }
        // A tick later: a layer the same tap opens takes the entry instead of a second
        // push in one activation, which would wait for a tap.
        spawn_forever(async move { back.sync() });
    }
}

pub(super) fn claim_entry() {
    Back::get().claim();
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

    fn layer(id: u64, onback: Callback<()>) -> Layer {
        Layer {
            id,
            onback,
            entries: 1,
        }
    }

    fn stack(layers: Vec<Layer>) -> BackStack {
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
            let mut stack = stack(vec![layer(1, lower)]);
            assert_eq!(stack.sync(), Some(1));
            stack.layers.push(layer(2, upper));
            assert_eq!(stack.sync(), Some(2));
            assert_eq!(stack.pressed(), Some(upper));
            assert_eq!(stack.sync(), None, "Back asks for no entry back");
            assert_eq!(
                stack.pressed(),
                Some(lower),
                "a second Back before upper closed"
            );
            stack.layers.retain(|layer| layer.id != 2);
            assert_eq!(stack.sync(), None, "both entries went with the Backs");
            stack.layers.clear();
            assert_eq!(stack.sync(), None);
            assert_eq!(stack.pressed(), None);
        });
    }

    #[test]
    fn a_layer_kept_open_steps_back_once_per_banked_tap() {
        let mut dom = VirtualDom::new(|| rsx! {});
        dom.rebuild_in_place();
        dom.in_scope(ScopeId::APP, || {
            let (steps, menu) = (Callback::new(|()| {}), Callback::new(|()| {}));
            let mut stack = stack(vec![layer(1, steps)]);
            stack.sync();
            assert_eq!(stack.tapped(), Some(2), "Next banks an entry");
            assert_eq!(stack.pressed(), Some(steps));
            assert_eq!(stack.pressed(), Some(steps), "still open, its banked entry");
            stack.layers.push(layer(2, menu));
            assert_eq!(stack.sync(), Some(1));
            assert_eq!(stack.pressed(), Some(menu), "a new layer takes its own");
        });
    }

    #[test]
    fn a_layer_opened_by_a_banking_tap_takes_that_entry() {
        let mut dom = VirtualDom::new(|| rsx! {});
        dom.rebuild_in_place();
        dom.in_scope(ScopeId::APP, || {
            let (modal, menu) = (Callback::new(|()| {}), Callback::new(|()| {}));
            let mut stack = stack(vec![layer(1, modal)]);
            stack.sync();
            assert_eq!(stack.tapped(), Some(2));
            stack.take_fresh();
            stack.layers.push(layer(2, menu));
            assert_eq!(stack.sync(), None, "no second push in one tap");
            assert_eq!(stack.pressed(), Some(menu));
            assert_eq!(stack.pressed(), Some(modal));
        });
    }

    #[test]
    fn a_closed_layer_takes_its_banked_entries_along() {
        let mut dom = VirtualDom::new(|| rsx! {});
        dom.rebuild_in_place();
        dom.in_scope(ScopeId::APP, || {
            let (steps, modal) = (Callback::new(|()| {}), Callback::new(|()| {}));
            let mut stack = stack(vec![layer(1, steps), layer(2, modal)]);
            stack.sync();
            for _ in 0..MAX_ENTRIES + 3 {
                stack.tapped();
            }
            assert_eq!(stack.held, 1 + MAX_ENTRIES, "taps bank on the top, capped");
            stack.close(2);
            assert_eq!(stack.sync(), Some(1), "the lower layer keeps its own");
            assert_eq!(stack.pressed(), Some(steps));
            stack.layers.clear();
            assert_eq!(stack.tapped(), None, "no layer banks nothing");
        });
    }

    #[test]
    fn a_tap_that_closes_its_layer_banks_on_the_one_below() {
        let mut dom = VirtualDom::new(|| rsx! {});
        dom.rebuild_in_place();
        dom.in_scope(ScopeId::APP, || {
            let (route, drawer) = (Callback::new(|()| {}), Callback::new(|()| {}));
            let mut stack = stack(vec![layer(1, route), layer(2, drawer)]);
            stack.sync();
            assert_eq!(stack.tapped(), Some(3), "the link tap banks on the drawer");
            stack.close(2);
            assert_eq!(stack.sync(), Some(2), "the route history keeps it");
            assert_eq!(stack.pressed(), Some(route));
            assert_eq!(stack.pressed(), Some(route));
        });
    }

    struct Pending;
    impl TimerSubscription for Pending {}

    #[test]
    fn the_last_layer_closed_by_its_tap_leaves_a_spare_for_a_later_one() {
        let mut dom = VirtualDom::new(|| rsx! {});
        dom.rebuild_in_place();
        dom.in_scope(ScopeId::APP, || {
            let (search, route) = (Callback::new(|()| {}), Callback::new(|()| {}));
            let mut stack = stack(vec![layer(1, search)]);
            stack.sync();
            assert_eq!(stack.tapped(), Some(2), "Enter on a result banks");
            assert!(stack.close(1), "nothing below takes it");
            stack.spare = Some(Box::new(Pending));
            assert_eq!(stack.sync(), Some(1), "the spare stays");
            stack.spare = None;
            stack.layers.push(layer(2, route));
            assert_eq!(stack.sync(), None, "the route takes it: no push");
            assert_eq!(stack.pressed(), Some(route));

            stack.layers.clear();
            stack.spare = Some(Box::new(Pending));
            assert_eq!(stack.pressed(), None, "Back pops the spare alone");
            assert!(stack.spare.is_none());
            assert!(!stack.close(3), "no fresh tap, no spare");
        });
    }
}
