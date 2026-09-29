use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::platform::{
    NotificationEvent, PermissionKind, PermissionState, PermissionSubscription, ShownNotification,
    SystemNotification, SystemNotificationError, permission, raise_window, system_notification,
};

/// A notification still on screen, by the id this hook gave it.
struct Entry {
    id: u64,
    tag: Option<String>,
    on_click: Option<Callback<()>>,
    shown: Box<dyn ShownNotification>,
}

/// Shows notifications the operating system draws, outside the page. Asks for
/// the permission only on [`request`](Self::request): mounting never prompts.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::{PermissionState, SystemNotification, use_system_notification};
/// # fn app() -> Element {
/// let mut notifier = use_system_notification();
///
/// rsx! {
///     if notifier.permission() == PermissionState::Granted {
///         button {
///             onclick: move |_| notifier.show(SystemNotification {
///                 body: Some("3 warnings".into()),
///                 tag: Some("build".into()),
///                 ..SystemNotification::new("Build done")
///             }),
///             "Notify"
///         }
///     } else {
///         button { onclick: move |_| notifier.request(), "Allow notifications" }
///     }
/// }
/// # }
/// ```
#[derive(Clone, Copy)]
pub struct SystemNotifier {
    supported: Signal<bool>,
    permission: Signal<PermissionState>,
    pending: Signal<bool>,
    error: Signal<Option<SystemNotificationError>>,
    shown: Signal<Vec<Entry>>,
    events: Signal<Vec<(u64, NotificationEvent)>>,
    next_id: Signal<u64>,
}

impl SystemNotifier {
    /// Asks for the permission, prompting if the user has not answered yet.
    /// Call it from a user's action. Does nothing while an earlier request waits.
    pub fn request(&mut self) {
        if *self.pending.peek() {
            return;
        }
        let Some(api) = system_notification() else {
            self.fail(SystemNotificationError::Unsupported);
            return;
        };
        let answer = api.request();
        self.pending.set(true);
        let mut this = *self;
        spawn(async move {
            let state = answer.await;
            this.pending.set(false);
            match state {
                PermissionState::Unsupported => this.fail(SystemNotificationError::Unsupported),
                PermissionState::Denied => {
                    this.set_permission(PermissionState::Denied);
                    this.fail(SystemNotificationError::Denied);
                }
                state => {
                    this.error.set(None);
                    this.set_permission(state);
                }
            }
        });
    }

    /// Shows `notification`; a failure lands in [`error`](Self::error).
    pub fn show(&mut self, notification: SystemNotification) {
        let Some(api) = system_notification() else {
            self.fail(SystemNotificationError::Unsupported);
            return;
        };
        let id = *self.next_id.peek();
        self.next_id.set(id + 1);
        // Queued, not handled here: a web event runs outside dioxus, inside the entry's closure.
        let events = self.events;
        let shown = api.show(
            &notification,
            Box::new(move |event| {
                let mut events = events;
                events.write().push((id, event));
            }),
        );
        let SystemNotification { tag, on_click, .. } = notification;
        let mut this = *self;
        spawn(async move {
            match shown.await {
                Ok(shown) => {
                    this.error.set(None);
                    // The platform replaced a notification with the same tag.
                    if tag.is_some() {
                        this.forget(|entry| entry.tag == tag);
                    }
                    this.shown.write().push(Entry {
                        id,
                        tag,
                        on_click,
                        shown,
                    });
                }
                Err(error) => this.fail(error),
            }
        });
    }

    /// Runs the queued events' clicks and forgets the closed notifications.
    fn handle_events(mut self) {
        let events = std::mem::take(&mut *self.events.write());
        for (id, event) in events {
            match event {
                NotificationEvent::Click => {
                    raise_window();
                    let on_click = self
                        .shown
                        .peek()
                        .iter()
                        .find(|entry| entry.id == id)
                        .and_then(|entry| entry.on_click);
                    if let Some(on_click) = on_click {
                        on_click.call(());
                    }
                }
                NotificationEvent::Close => {
                    self.forget(|entry| entry.id == id);
                }
            }
        }
    }

    /// Closes the shown notifications with this `tag`.
    pub fn close(&mut self, tag: &str) {
        let closing = self.forget(|entry| entry.tag.as_deref() == Some(tag));
        for entry in closing {
            entry.shown.close();
        }
    }

    /// Takes the matching entries out; dropping them stops their events.
    fn forget(mut self, matches: impl Fn(&Entry) -> bool) -> Vec<Entry> {
        let mut shown = self.shown.write();
        let (gone, kept) = shown.drain(..).partition(|entry| matches(entry));
        *shown = kept;
        gone
    }

    pub(crate) fn fail(&mut self, error: SystemNotificationError) {
        self.error.set(Some(error));
        match error {
            SystemNotificationError::Unsupported => {
                self.supported.set(false);
                self.set_permission(PermissionState::Unsupported);
            }
            // A show before any request is refused too, yet a request may still prompt.
            SystemNotificationError::Denied | SystemNotificationError::Failed => {}
        }
    }

    fn set_permission(&mut self, state: PermissionState) {
        if *self.permission.peek() != state {
            self.permission.set(state);
        }
    }

    /// Whether this target can show system notifications. `false` until mounted. Reactive.
    pub fn is_supported(&self) -> bool {
        (self.supported)()
    }

    /// The notification permission, kept current where the platform reports changes. Reactive.
    pub fn permission(&self) -> PermissionState {
        (self.permission)()
    }

    /// Whether a [`request`](Self::request) waits for its answer. Reactive.
    pub fn is_pending(&self) -> bool {
        (self.pending)()
    }

    /// Why the last request or show failed; the next success clears it. Reactive.
    pub fn error(&self) -> Option<SystemNotificationError> {
        (self.error)()
    }
}

/// A [`SystemNotifier`] for this component. Unmounting stops click handling
/// but leaves shown notifications on screen. Announces nothing, and a system
/// notification is never the only channel: say the same in the page.
pub fn use_system_notification() -> SystemNotifier {
    let mut notifier = SystemNotifier {
        supported: use_signal(|| false),
        permission: use_signal(PermissionState::default),
        pending: use_signal(|| false),
        error: use_signal(|| None),
        shown: use_signal(Vec::new),
        events: use_signal(Vec::new),
        next_id: use_signal(|| 0),
    };
    use_effect(move || {
        if !notifier.events.read().is_empty() {
            notifier.handle_events();
        }
    });
    let listener: Rc<RefCell<Option<Box<dyn PermissionSubscription>>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    use_drop({
        let listener = listener.clone();
        move || {
            listener.borrow_mut().take();
            notifier.shown.write_unchecked().clear();
        }
    });
    // A server cannot know either answer, so both are read after mount (hydration).
    use_effect(move || {
        let Some(api) = system_notification() else {
            notifier.set_permission(PermissionState::Unsupported);
            return;
        };
        let probe = api.probe();
        let listener = listener.clone();
        spawn(async move {
            let Some(state) = probe.await else {
                notifier.set_permission(PermissionState::Unsupported);
                return;
            };
            notifier.supported.set(true);
            notifier.set_permission(state);
            let Some(api) = permission() else {
                return;
            };
            let changes = api.on_change(
                PermissionKind::Notifications,
                // The Permissions API may not know the name: `Unknown` keeps the probe's answer.
                Box::new(move |state| {
                    let mut notifier = notifier;
                    if state != PermissionState::Unknown {
                        notifier.set_permission(state);
                    }
                }),
            );
            *listener.borrow_mut() = Some(changes);
        });
    });
    notifier
}

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "system_notification_tests.rs"]
mod tests;
