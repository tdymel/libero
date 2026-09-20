use std::time::Duration;

use dioxus::prelude::*;

use crate::{
    platform::{TimerSubscription, hits_inline_boxes, timer},
    sx::{StaticSx, sx},
};

/// The trigger's wrapper and the popover's anchor. `max-content`, so a
/// stretching flex or grid parent cannot widen it past the trigger.
pub(super) static TRIGGER_WRAPPER_SX: StaticSx = StaticSx::new(|| {
    let base = sx()
        .display("inline-block")
        .width("max-content")
        .max_width("100%");
    // Blitz reaches an inline box in a padded block only as a z-indexed one.
    match hits_inline_boxes() {
        true => base,
        false => base.position("relative").z_index("1"),
    }
});

/// Whether the pointer rests on a trigger, applied after the open and close
/// delays. `Tooltip` and `HoverCard` share it.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct HoverIntent {
    /// Root-owned: the timer's callback runs outside every scope, and writing
    /// a signal is all it may do ([[codebase/platform/platform-timer]]).
    hovered: Signal<bool>,
    /// Replacing or dropping the subscription cancels it.
    pending: CopyValue<Option<Box<dyn TimerSubscription>>>,
}

impl HoverIntent {
    pub(super) fn get(&self) -> bool {
        (self.hovered)()
    }

    /// Moves towards `target` after `delay` ms, cancelling any move under way.
    pub(super) fn hover(&self, target: bool, delay: u32) {
        let mut pending = self.pending;
        pending.set(None);
        if *self.hovered.peek() == target {
            return;
        }
        // A close waits at least a task: leaving the trigger for the portaled
        // box would otherwise unmount it before its own `mouseenter` runs.
        match (delay > 0 || !target).then(timer).flatten() {
            Some(timer) => {
                let hovered = self.hovered;
                pending.set(Some(timer.after(
                    Duration::from_millis(delay.into()),
                    Box::new(move || {
                        let mut hovered = hovered;
                        hovered.set(target);
                    }),
                )));
            }
            None => {
                let mut hovered = self.hovered;
                hovered.set(target);
            }
        }
    }

    /// Sets it now, cancelling any move under way: a dismissal must not be
    /// undone by an open delay that was already counting.
    pub(super) fn set(&self, value: bool) {
        let mut pending = self.pending;
        pending.set(None);
        let mut hovered = self.hovered;
        hovered.set(value);
    }
}

pub(super) fn use_hover_intent() -> HoverIntent {
    let hovered = use_hook(|| Signal::new_in_scope(false, ScopeId::ROOT));
    let pending = use_hook(|| CopyValue::new(None::<Box<dyn TimerSubscription>>));
    use_drop(move || {
        let mut pending = pending;
        pending.set(None);
        hovered.manually_drop();
    });
    HoverIntent { hovered, pending }
}
