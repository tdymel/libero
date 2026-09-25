use std::time::Duration;

use dioxus::prelude::*;

use super::keyboard::Level;
use crate::hooks::{Scheduled, use_scheduled};

/// What the pointer resting on an item does once the delay runs out.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct HoverAction {
    focus: usize,
    /// The submenu to have open afterwards; `None` closes the open one.
    open: Option<usize>,
}

/// The pointer resting on an item, acting after the delay.
#[derive(Clone, Copy)]
pub(super) struct HoverDelay {
    action: CopyValue<Option<HoverAction>>,
    scheduled: Scheduled,
    delay: Duration,
}

impl HoverDelay {
    fn schedule(&self, action: HoverAction) {
        let mut pending = self.action;
        pending.set(Some(action));
        self.scheduled.after(self.delay.as_millis() as u64);
    }

    pub(super) fn cancel(&self) {
        self.scheduled.cancel();
    }

    /// The pointer entered item `index`, which opens submenu `opens`.
    pub(super) fn enter(&self, level: Level, index: usize, opens: Option<usize>) {
        self.cancel();
        let expanded = *level.open_child.peek();
        match expanded {
            // Another item's submenu is open: wait, as the pointer may be crossing into it.
            Some(open) if open != index => self.schedule(HoverAction {
                focus: index,
                open: opens,
            }),
            open => {
                level.focus(index);
                if opens.is_some() && open != opens {
                    self.schedule(HoverAction {
                        focus: index,
                        open: opens,
                    });
                }
            }
        }
    }
}

pub(super) fn use_hover_delay(level: Level, delay: Duration) -> HoverDelay {
    let action = use_hook(|| CopyValue::new(None::<HoverAction>));
    let scheduled = use_scheduled(move |_| {
        let mut slot = action;
        let Some(action) = slot.write().take() else {
            return;
        };
        let mut open_child = level.open_child;
        level.focus(action.focus);
        open_child.set(action.open);
    });
    HoverDelay {
        action,
        scheduled,
        delay,
    }
}
