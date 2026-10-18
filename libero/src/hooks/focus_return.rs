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

    /// Whether focus moved here: connected, and the renderer took the command.
    fn take_focus(&self) -> bool {
        if !self.is_connected() {
            return false;
        }
        match self {
            Trigger::Mounted(data) => platform::element(data).focus().is_ok(),
            Trigger::Active(element) => element.focus().is_ok(),
            Trigger::Kept(token) => {
                platform::focus_kept(*token);
                true
            }
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
        self.restore_with(spawn);
    }

    /// [`restore`](Self::restore) from a scope being dropped: the focus task
    /// runs at the root, where it outlives that scope.
    pub(crate) fn restore_detached(&self) {
        self.restore_with(|task| {
            dioxus::core::spawn_forever(task);
        });
    }

    fn restore_with<T>(
        &self,
        spawn: impl FnOnce(std::pin::Pin<Box<dyn Future<Output = ()>>>) -> T,
    ) {
        let mut signal = self.trigger;
        let Ok(trigger) = signal.try_peek().map(|trigger| (*trigger).clone()) else {
            return;
        };
        let Some(trigger) = trigger else {
            return;
        };
        if matches!(trigger, Trigger::Active(_) | Trigger::Kept(_))
            && let Ok(mut trigger) = signal.try_write()
        {
            trigger.take();
        }

        // Known-gone ones dropped here: Blitz locks the document while tasks
        // drain, so there the task's own check can only answer "not known gone".
        let fallbacks = self
            .fallbacks
            .try_peek()
            .map(|fallbacks| fallbacks.clone())
            .unwrap_or_default();
        let targets: Vec<Trigger> = std::iter::once(trigger)
            .chain(
                fallbacks
                    .into_iter()
                    .map(|element| Trigger::Active(Rc::new(element))),
            )
            .filter(Trigger::is_connected)
            .collect();
        // Nothing left: no blur, the browser's own pick beats `<body>`.
        if targets.is_empty() {
            return;
        }

        // Spawned: focusing inside the closing event's dispatch panics. Through
        // the backend, not `set_focus`, which Blitz must defer (todo 189).
        spawn(Box::pin(async move {
            // Now, so the closing render's effects already see focus back.
            let landed = targets.iter().position(Trigger::take_focus);
            // Once that render is applied, a target it removed hands over to the next one.
            dioxus::core::queue_effect(move || {
                let from = match landed {
                    Some(at) if targets[at].is_connected() => return,
                    Some(at) => at + 1,
                    None => 0,
                };
                targets[from..].iter().any(Trigger::take_focus);
            });
        }));
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
