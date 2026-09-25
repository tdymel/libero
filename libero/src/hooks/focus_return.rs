use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    hooks::ElementHandle,
    platform::{self, ElementApi, document},
};

/// The trigger's `onmounted` data, or whatever held focus when the overlay
/// opened (a trigger in the caller's markup).
#[derive(Clone)]
enum Trigger {
    Mounted(Rc<MountedData>),
    Active(Rc<dyn ElementApi>),
    /// A WebView's focused element, kept page-side under this token.
    Kept(u64),
}

impl Trigger {
    /// Whether this element is not known to be gone. A renderer that cannot
    /// tell answers `true`, so a WebView never takes the fallback chain.
    fn is_connected(&self) -> bool {
        match self {
            Trigger::Mounted(data) => platform::element(data).is_connected(),
            Trigger::Active(element) => element.is_connected(),
            Trigger::Kept(_) => true,
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
    /// Attach to the trigger's `onmounted`, for an overlay the caller renders
    /// conditionally. **Arm it once**: [`restore`](Self::restore) keeps it.
    /// No component calls it; the e2e unit `focus_return` covers it.
    pub fn remember(&self, event: Event<MountedData>) {
        let mut trigger = self.trigger;
        trigger.set(Some(Trigger::Mounted(event.data.clone())));
    }

    /// Remembers whatever holds focus now, for a trigger this overlay never sees.
    /// Call it **synchronously in the opening handler, on every open**:
    /// [`restore`](Self::restore) consumes it.
    pub fn remember_active(&self) {
        let active = document()
            .and_then(|document| document.active_element())
            .map(Rc::from);
        let mut trigger = self.trigger;
        trigger.set(active.map(Trigger::Active));
    }

    /// As [`remember_active`](Self::remember_active), which a WebView keeps page-side.
    pub(crate) fn remember_focused(&self) {
        let active = document().and_then(|document| document.active_element());
        let mut trigger = self.trigger;
        trigger.set(match active {
            Some(active) => Some(Trigger::Active(Rc::from(active))),
            None => platform::keep_focused().map(Trigger::Kept),
        });
    }

    /// Remembers `element` as a snapshot, consumed by the next
    /// [`restore`](Self::restore) like [`remember_active`](Self::remember_active).
    pub(crate) fn remember_element(&self, element: ElementHandle) {
        let mut trigger = self.trigger;
        trigger.set(Some(Trigger::Active(Rc::new(element))));
    }

    /// Where focus lands if the trigger is gone on close (a deleted row's table).
    /// **Appends**, nearest first; a repeat is ignored, so a render may call it.
    pub fn fallback(&self, element: ElementHandle) {
        let mut fallbacks = self.fallbacks;
        if fallbacks.peek().contains(&element) {
            return;
        }
        fallbacks.write().push(element);
    }

    /// Hands focus back to the trigger, or else the first connected
    /// [`fallback`](Self::fallback). A snapshot is consumed, an element kept (todos 37, 44).
    pub fn restore(&self) {
        let mut signal = self.trigger;
        let Some(trigger) = (*signal.peek()).clone() else {
            return;
        };
        if matches!(trigger, Trigger::Active(_) | Trigger::Kept(_)) {
            signal.write().take();
        }

        // Read outside the `spawn`: Blitz locks the document while tasks drain.
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
        // Nothing left: no blur, the browser's own pick beats `<body>`.
        let Some(target) = target else {
            return;
        };

        // Spawned: focusing inside the closing event's dispatch panics. Through
        // the backend, not `set_focus`, which Blitz must defer (todo 189).
        spawn(async move {
            let _ = match target {
                Trigger::Mounted(data) => platform::element(&data).focus(),
                Trigger::Active(element) => element.focus(),
                Trigger::Kept(token) => {
                    platform::focus_kept(token);
                    Ok(())
                }
            };
        });
    }
}

/// A [`FocusReturn`] for an overlay this component opens.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::use_focus_return;
/// # fn app() -> Element {
/// let focus = use_focus_return();
/// let mut open = use_signal(|| false);
///
/// rsx! {
///     button {
///         onclick: move |_| {
///             focus.remember_active();
///             open.set(true);
///         },
///         "Open"
///     }
///     if open() {
///         div { role: "dialog",
///             button {
///                 onclick: move |_| {
///                     open.set(false);
///                     focus.restore();
///                 },
///                 "Close"
///             }
///         }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/accessibility/use-focus-return>
pub fn use_focus_return() -> FocusReturn {
    FocusReturn {
        trigger: use_signal(|| None),
        fallbacks: use_signal(Vec::new),
    }
}
