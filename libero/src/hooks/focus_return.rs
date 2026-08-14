use std::rc::Rc;

use dioxus::prelude::*;

/// Restores focus to whatever triggered an overlay, once it closes.
#[derive(Clone, Copy)]
pub struct FocusReturn {
    trigger: Signal<Option<Rc<MountedData>>>,
}

impl FocusReturn {
    /// Attach to the trigger's `onmounted`.
    pub fn remember(&mut self, event: Event<MountedData>) {
        self.trigger.set(Some(event.data.clone()));
    }

    pub fn restore(&self) {
        if let Some(trigger) = (self.trigger)() {
            spawn(async move {
                let _ = trigger.set_focus(true).await;
            });
        }
    }
}

pub fn use_focus_return() -> FocusReturn {
    FocusReturn {
        trigger: use_signal(|| None),
    }
}
