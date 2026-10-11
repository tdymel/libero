use dioxus::prelude::*;

use crate::platform::{FcmError, FcmMessage, FcmOptions};

/// This Android app's Firebase Cloud Messaging token and the messages it receives
/// while running. The app's server sends them; [`use_system_notification`](super::use_system_notification)
/// shows one if the app wants it shown. Registers only on [`register`](Self::register).
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::{FcmOptions, use_fcm};
/// # fn app() -> Element {
/// let mut fcm = use_fcm(FcmOptions {
///     application_id: "1:123:android:abc".into(),
///     api_key: "<current_key>".into(),
///     project_id: "my-app".into(),
///     sender_id: "123".into(),
/// });
///
/// rsx! {
///     button { onclick: move |_| fcm.register(), "Get pushes" }
///     if let Some(token) = fcm.token() {
///         // POST `token` to the app's server.
///         p { "Token {token}" }
///     }
///     if let Some(title) = fcm.last_message().and_then(|message| message.title) {
///         p { "{title}" }
///     }
/// }
/// # }
/// ```
#[derive(Clone, Copy)]
pub struct Fcm {
    #[cfg_attr(
        not(all(target_os = "android", not(feature = "native"))),
        allow(dead_code)
    )]
    options: CopyValue<FcmOptions>,
    supported: Signal<bool>,
    token: Signal<Option<String>>,
    message: Signal<Option<FcmMessage>>,
    pending: Signal<bool>,
    error: Signal<Option<FcmError>>,
    /// The answer count a waiting `register` was made at.
    asked_at: CopyValue<Option<u64>>,
}

impl Fcm {
    /// Starts Firebase with the options and asks for the token; no prompt. Does
    /// nothing while an earlier call waits.
    pub fn register(&mut self) {
        if *self.pending.peek() {
            return;
        }
        #[cfg(all(target_os = "android", not(feature = "native")))]
        {
            self.asked_at
                .set(Some(crate::platform::fcm_state().answers));
            self.pending.set(true);
            let registered = crate::platform::fcm_register(self.options.peek().clone());
            let mut this = *self;
            spawn(async move {
                if let Err(error) = registered.await {
                    this.asked_at.set(None);
                    this.pending.set(false);
                    this.error.set(Some(FcmError::Failed(error)));
                }
            });
        }
        #[cfg(not(all(target_os = "android", not(feature = "native"))))]
        self.error.set(Some(FcmError::Unsupported));
    }

    #[cfg_attr(
        not(all(target_os = "android", not(feature = "native"))),
        allow(dead_code)
    )]
    fn apply(&mut self, state: crate::platform::FcmState) {
        if *self.token.peek() != state.token {
            self.token.set(state.token.clone());
        }
        if *self.message.peek() != state.message {
            self.message.set(state.message);
        }
        let answered = matches!(*self.asked_at.peek(), Some(at) if state.answers > at);
        if answered {
            self.asked_at.set(None);
            self.pending.set(false);
            let error = match (state.token, state.error) {
                (Some(_), _) => None,
                (None, error) => Some(FcmError::Failed(error.unwrap_or_default())),
            };
            self.error.set(error);
        }
    }

    /// Whether this target has the Firebase bridge: an Android WebView app.
    /// `false` until mounted. Reactive.
    pub fn is_supported(&self) -> bool {
        (self.supported)()
    }

    /// This install's token for the app's server; Firebase may replace it. Reactive.
    pub fn token(&self) -> Option<String> {
        (self.token)()
    }

    /// The newest message received while the app ran. Reactive.
    pub fn last_message(&self) -> Option<FcmMessage> {
        (self.message)()
    }

    /// Whether a [`register`](Self::register) waits for its answer. Reactive.
    pub fn is_pending(&self) -> bool {
        (self.pending)()
    }

    /// Why the last call failed; the next token clears it. Reactive.
    pub fn error(&self) -> Option<FcmError> {
        (self.error)()
    }
}

/// An [`Fcm`] for this component, behind the `fcm` feature. `options` apply to the
/// next `register`. Android only: elsewhere [`is_supported`](Fcm::is_supported)
/// stays `false`. Announces nothing.
pub fn use_fcm(options: FcmOptions) -> Fcm {
    let mut fcm_options = use_hook(|| CopyValue::new(options.clone()));
    if *fcm_options.peek() != options {
        fcm_options.set(options);
    }
    #[allow(unused_mut)]
    let mut fcm = Fcm {
        options: fcm_options,
        supported: use_signal(|| false),
        token: use_signal(|| None),
        message: use_signal(|| None),
        pending: use_signal(|| false),
        error: use_signal(|| None),
        asked_at: use_hook(|| CopyValue::new(None)),
    };
    use_effect(move || {
        #[cfg(all(target_os = "android", not(feature = "native")))]
        {
            fcm.supported.set(true);
            let mut changes = crate::platform::fcm_watch();
            spawn(async move {
                crate::platform::fcm_flush();
                fcm.apply(crate::platform::fcm_state());
                while changes.next().await {
                    fcm.apply(crate::platform::fcm_state());
                }
            });
        }
    });
    fcm
}
