use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    hooks::ElementHandle,
    platform::{self, ElementApi, document},
};

/// Two ways of naming the same element. An overlay that renders its own
/// trigger has that trigger's `onmounted`; one opened from the caller's markup
/// has only whatever held focus at the moment it opened.
#[derive(Clone)]
enum Trigger {
    Mounted(Rc<MountedData>),
    Active(Rc<dyn ElementApi>),
}

impl Trigger {
    /// Whether this element is still in the document.
    ///
    /// A renderer that cannot tell answers `true`, so this reads as "not known
    /// to be gone" - which is what keeps the fallback chain from firing on
    /// every restore under a WebView.
    fn is_connected(&self) -> bool {
        match self {
            Trigger::Mounted(data) => platform::element(data).is_connected(),
            Trigger::Active(element) => element.is_connected(),
        }
    }
}

/// Restores focus to whatever triggered an overlay, once it closes.
#[derive(Clone, Copy)]
pub struct FocusReturn {
    trigger: Signal<Option<Trigger>>,
    /// Where focus goes when the trigger is gone, nearest first.
    fallbacks: Signal<Vec<ElementHandle>>,
}

impl FocusReturn {
    /// Attach to the trigger's `onmounted`. Kept for the overlay whose trigger
    /// belongs to the caller and is never handed to the component -
    /// an overlay rendered conditionally by its caller has no opening handler
    /// of its own to call `remember_active()` from, so the caller names the
    /// trigger instead.
    ///
    /// **Arm it once.** The element named here is an element, not a snapshot:
    /// it outlives any number of open/close cycles, and
    /// [`restore`](Self::restore) keeps it rather than clearing it.
    ///
    /// No component here calls it; the e2e unit `focus_return` covers it in a
    /// browser.
    pub fn remember(&self, event: Event<MountedData>) {
        let mut trigger = self.trigger;
        trigger.set(Some(Trigger::Mounted(event.data.clone())));
    }

    /// Remembers whatever holds focus right now, for a trigger this overlay
    /// never sees - the caller's own button, say.
    ///
    /// Call it **synchronously inside the handler that opens the overlay**:
    /// there the active element still is the element the user acted on. A
    /// frame later it is not. And call it **on every open**: a snapshot is a
    /// fact about one moment, so [`restore`](Self::restore) consumes it.
    pub fn remember_active(&self) {
        let active = document()
            .and_then(|document| document.active_element())
            .map(Rc::from);
        let mut trigger = self.trigger;
        trigger.set(active.map(Trigger::Active));
    }

    /// Remembers `element` as a snapshot, consumed by the next
    /// [`restore`](Self::restore) like [`remember_active`](Self::remember_active).
    pub(crate) fn remember_element(&self, element: ElementHandle) {
        let mut trigger = self.trigger;
        trigger.set(Some(Trigger::Active(Rc::new(element))));
    }

    /// Names where focus should land if the trigger is gone by the time the
    /// overlay closes - for a confirm-delete dialog, the list or table the
    /// deleted row lived in.
    ///
    /// **Appends**, so call it once per tier, nearest first: the container the
    /// trigger lived in, then a landing place the consumer makes focusable
    /// with `tabindex="-1"` so a screen reader announces something rather than
    /// resuming at the top of the document. Calling it again with an element
    /// already named does nothing, so it is safe to call from a render.
    pub fn fallback(&self, element: ElementHandle) {
        let mut fallbacks = self.fallbacks;
        if fallbacks.peek().contains(&element) {
            return;
        }
        fallbacks.write().push(element);
    }

    /// Hands focus back.
    ///
    /// **A snapshot is consumed, an element is kept.** A
    /// [`remember_active`](Self::remember_active) trigger is a fact about the
    /// moment the overlay opened, so it is forgotten here and a second close
    /// cannot take focus off whatever holds it by then - which is safe only
    /// because every such consumer arms again on each open. A
    /// [`remember`](Self::remember) trigger is the element itself and stays:
    /// clearing it would make a caller who armed once from `onmounted` drop
    /// focus to `<body>` on the second close ([[todos]] item 44).
    ///
    /// Focus goes to the remembered trigger if it is still in the document,
    /// otherwise to the first [`fallback`](Self::fallback) that is. **The
    /// trigger being gone is the ordinary case, not an exotic one**: a
    /// confirm-delete dialog is opened by the row's own Delete button and the
    /// application deletes that row. Without the chain, `focus()` on the
    /// detached node returns `Ok(())`, does nothing, and focus silently falls
    /// to `<body>` ([[todos]] item 37).
    ///
    /// Connectedness is read here rather than inside the `spawn`: under Blitz
    /// the document is locked while dioxus drains tasks, so a read is better
    /// informed where it is called.
    ///
    /// The `spawn` is load-bearing: this runs inside the dispatch of the event
    /// that closed the overlay, where focusing re-enters a dioxus
    /// `EventHandler` whose click is still bubbling, and that panics. It also
    /// lands the focus after the overlay is gone rather than beside it.
    pub fn restore(&self) {
        let mut signal = self.trigger;
        let Some(trigger) = (*signal.peek()).clone() else {
            return;
        };
        if matches!(trigger, Trigger::Active(_)) {
            signal.write().take();
        }

        let target = match trigger.is_connected() {
            true => Some(trigger),
            false => self
                .fallbacks
                .peek()
                .iter()
                .copied()
                .find(|element| element.is_connected())
                .map(|element| Trigger::Active(Rc::new(element))),
        };
        // Nothing left to focus. Deliberately not a blur: whatever the browser
        // moved focus to when the trigger went away is a better answer than
        // `<body>`.
        let Some(target) = target else {
            return;
        };

        // Through the backend, not `MountedData::set_focus`: under Blitz that
        // takes the document mutably, which a task never finds free, and only
        // the backend knows to defer the command (todo 189).
        spawn(async move {
            let _ = match target {
                Trigger::Mounted(data) => platform::element(&data).focus(),
                Trigger::Active(element) => element.focus(),
            };
        });
    }
}

pub fn use_focus_return() -> FocusReturn {
    FocusReturn {
        trigger: use_signal(|| None),
        fallbacks: use_signal(Vec::new),
    }
}
