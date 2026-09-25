use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::platform::{
    PermissionKind, PermissionState, PermissionSubscription, PushApi, PushEndpoint, PushError,
    PushOptions, permission, push,
};

/// This browser's web push subscription. The app's service worker shows what
/// arrives; the app's server sends it. Subscribes only on
/// [`subscribe`](Self::subscribe): mounting never prompts.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::{PushOptions, use_push_subscription};
/// # fn app() -> Element {
/// let mut push = use_push_subscription(PushOptions {
///     service_worker: "/sw.js".into(),
///     vapid_public_key: "<the server's VAPID public key>".into(),
/// });
///
/// rsx! {
///     button { onclick: move |_| push.subscribe(), "Get pushes" }
///     if let Some(subscription) = push.subscription() {
///         // POST `subscription` to the app's server.
///         p { "Subscribed at {subscription.endpoint}" }
///     }
/// }
/// # }
/// ```
#[derive(Clone, Copy)]
pub struct PushSubscription {
    options: CopyValue<PushOptions>,
    supported: Signal<bool>,
    permission: Signal<PermissionState>,
    subscription: Signal<Option<PushEndpoint>>,
    pending: Signal<bool>,
    error: Signal<Option<PushError>>,
}

impl PushSubscription {
    /// Registers the service worker and subscribes, prompting for the
    /// notification permission if the user has not answered yet. Call it from a
    /// user's action. Does nothing while an earlier call waits.
    pub fn subscribe(&mut self) {
        let Some(api) = self.start() else {
            return;
        };
        let subscribed = api.subscribe(&self.options.peek());
        let mut this = *self;
        spawn(async move {
            let endpoint = subscribed.await;
            this.pending.set(false);
            this.settle(endpoint.map(Some));
        });
    }

    /// Ends the subscription; the app's server should forget its endpoint too.
    pub fn unsubscribe(&mut self) {
        let Some(api) = self.start() else {
            return;
        };
        let done = api.unsubscribe();
        let mut this = *self;
        spawn(async move {
            let done = done.await;
            this.pending.set(false);
            this.settle(done.map(|()| None));
        });
    }

    fn start(&mut self) -> Option<&'static dyn PushApi> {
        if *self.pending.peek() {
            return None;
        }
        let Some(api) = push() else {
            self.settle(Err(PushError::Unsupported));
            return None;
        };
        self.pending.set(true);
        Some(api)
    }

    pub(crate) fn settle(&mut self, answer: Result<Option<PushEndpoint>, PushError>) {
        match answer {
            Ok(endpoint) => {
                if endpoint.is_some() {
                    self.set_permission(PermissionState::Granted);
                }
                self.subscription.set(endpoint);
                self.error.set(None);
            }
            Err(error) => {
                self.error.set(Some(error));
                match error {
                    PushError::Unsupported => {
                        self.supported.set(false);
                        self.set_permission(PermissionState::Unsupported);
                    }
                    PushError::Denied => self.set_permission(PermissionState::Denied),
                    PushError::Failed => {}
                }
            }
        }
    }

    fn set_permission(&mut self, state: PermissionState) {
        if *self.permission.peek() != state {
            self.permission.set(state);
        }
    }

    /// Whether this target has web push. `false` until mounted. Reactive.
    pub fn is_supported(&self) -> bool {
        (self.supported)()
    }

    /// The notification permission push needs, kept current where the platform reports changes. Reactive.
    pub fn permission(&self) -> PermissionState {
        (self.permission)()
    }

    /// The current subscription, read after mount. Reactive.
    pub fn subscription(&self) -> Option<PushEndpoint> {
        (self.subscription)()
    }

    /// Whether a [`subscribe`](Self::subscribe) or
    /// [`unsubscribe`](Self::unsubscribe) waits for its answer. Reactive.
    pub fn is_pending(&self) -> bool {
        (self.pending)()
    }

    /// Why the last call failed; the next success clears it. Reactive.
    pub fn error(&self) -> Option<PushError> {
        (self.error)()
    }
}

/// A [`PushSubscription`] for this component. `options` apply to the next
/// `subscribe`. Web only: elsewhere [`is_supported`](PushSubscription::is_supported)
/// stays `false`. Announces nothing.
pub fn use_push_subscription(options: PushOptions) -> PushSubscription {
    let mut push_options = use_hook(|| CopyValue::new(options.clone()));
    if *push_options.peek() != options {
        push_options.set(options);
    }
    let mut subscription = PushSubscription {
        options: push_options,
        supported: use_signal(|| false),
        permission: use_signal(PermissionState::default),
        subscription: use_signal(|| None),
        pending: use_signal(|| false),
        error: use_signal(|| None),
    };
    let listener: Rc<RefCell<Option<Box<dyn PermissionSubscription>>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    use_drop({
        let listener = listener.clone();
        move || {
            listener.borrow_mut().take();
        }
    });
    // A server cannot know either answer, so both are read after mount (hydration).
    use_effect(move || {
        let Some(api) = push() else {
            subscription.set_permission(PermissionState::Unsupported);
            return;
        };
        subscription.supported.set(true);
        let current = api.current();
        spawn(async move {
            if let Ok(Some(endpoint)) = current.await {
                subscription.subscription.set(Some(endpoint));
            }
        });
        let Some(api) = permission() else {
            return;
        };
        let changes = api.on_change(
            PermissionKind::Notifications,
            Box::new(move |state| {
                let mut subscription = subscription;
                subscription.set_permission(state);
            }),
        );
        *listener.borrow_mut() = Some(changes);
    });
    subscription
}

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "push_subscription_tests.rs"]
mod tests;
