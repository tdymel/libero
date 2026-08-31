use std::rc::Rc;

use dioxus::prelude::*;

use crate::platform::{ElementApi, document};

/// Two ways of naming the same element. An overlay that renders its own
/// trigger has that trigger's `onmounted`; one opened from the caller's markup
/// has only whatever held focus at the moment it opened.
enum Trigger {
    Mounted(Rc<MountedData>),
    Active(Rc<dyn ElementApi>),
}

/// Restores focus to whatever triggered an overlay, once it closes.
#[derive(Clone, Copy)]
pub struct FocusReturn {
    trigger: Signal<Option<Trigger>>,
}

impl FocusReturn {
    /// Attach to the trigger's `onmounted`.
    pub fn remember(&self, event: Event<MountedData>) {
        let mut trigger = self.trigger;
        trigger.set(Some(Trigger::Mounted(event.data.clone())));
    }

    /// Remembers whatever holds focus right now, for a trigger this overlay
    /// never sees - the caller's own button, say.
    ///
    /// Call it **synchronously inside the handler that opens the overlay**:
    /// there the active element still is the element the user acted on. A
    /// frame later it is not.
    pub fn remember_active(&self) {
        let active = document()
            .and_then(|document| document.active_element())
            .map(Rc::from);
        let mut trigger = self.trigger;
        trigger.set(active.map(Trigger::Active));
    }

    /// Hands focus back, and forgets the trigger - a second close cannot then
    /// take focus off whatever holds it by then.
    ///
    /// Focus lands from a task rather than inline. Called from the handler of
    /// the event that closed the overlay, a synchronous focus would be taken
    /// straight back by the focus trap the overlay has not finished leaving.
    pub fn restore(&self) {
        let mut signal = self.trigger;
        let Some(trigger) = signal.write().take() else {
            return;
        };

        spawn(async move {
            match trigger {
                Trigger::Mounted(data) => {
                    let _ = data.set_focus(true).await;
                }
                Trigger::Active(element) => {
                    let _ = element.focus();
                }
            }
        });
    }
}

pub fn use_focus_return() -> FocusReturn {
    FocusReturn {
        trigger: use_signal(|| None),
    }
}
