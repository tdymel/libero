use dioxus::prelude::*;

use crate::{
    hooks::{Scheduled, use_scheduled},
    platform::{hits_inline_boxes, timer},
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
    hovered: Signal<bool>,
    /// The state the pending timer moves to.
    target: CopyValue<bool>,
    scheduled: Scheduled,
}

impl HoverIntent {
    pub(super) fn get(&self) -> bool {
        (self.hovered)()
    }

    /// Moves towards `target` after `delay` ms, cancelling any move under way.
    pub(super) fn hover(&self, target: bool, delay: u32) {
        self.scheduled.cancel();
        if *self.hovered.peek() == target {
            return;
        }
        // A close waits at least a task: leaving the trigger for the portaled
        // box would otherwise unmount it before its own `mouseenter` runs.
        match (delay > 0 || !target).then(timer).flatten() {
            Some(_) => {
                let mut pending = self.target;
                pending.set(target);
                self.scheduled.after(delay.into());
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
        self.scheduled.cancel();
        let mut hovered = self.hovered;
        hovered.set(value);
    }
}

pub(super) fn use_hover_intent() -> HoverIntent {
    let mut hovered = use_signal(|| false);
    let target = use_hook(|| CopyValue::new(false));
    let scheduled = use_scheduled(move |_| hovered.set(*target.peek()));
    HoverIntent {
        hovered,
        target,
        scheduled,
    }
}
