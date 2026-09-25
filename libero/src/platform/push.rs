//! A web push subscription, from the Push API. Sending stays on the app's server.

use std::{future::Future, pin::Pin};

/// Where the app's service worker lives and which server may push to it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PushOptions {
    /// The worker script's URL, e.g. `/sw.js`; it handles `push` events.
    pub service_worker: String,
    /// The server's VAPID public key, base64url.
    pub vapid_public_key: String,
}

/// What the app's server needs to push to this browser: POST it there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushEndpoint {
    pub endpoint: String,
    /// The browser's public key, base64url.
    pub p256dh: String,
    /// The shared authentication secret, base64url.
    pub auth: String,
}

/// Why no subscription came.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushError {
    /// No service worker or Push API on this target.
    Unsupported,
    /// The notification permission is not granted.
    Denied,
    /// The worker did not register, or the push service refused.
    Failed,
}

impl PushError {
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn from_name(name: &str) -> Self {
        match name {
            "Unsupported" => Self::Unsupported,
            "Denied" => Self::Denied,
            _ => Self::Failed,
        }
    }
}

pub(crate) type Subscribed<T> = Pin<Box<dyn Future<Output = Result<T, PushError>>>>;

pub(crate) trait PushApi {
    /// The page's subscription, without prompting.
    fn current(&self) -> Subscribed<Option<PushEndpoint>>;
    /// Registers the worker and subscribes, prompting for the permission.
    fn subscribe(&self, options: &PushOptions) -> Subscribed<PushEndpoint>;
    fn unsubscribe(&self) -> Subscribed<()>;
}

/// Each answers an error name, `null` or `[endpoint, p256dh, auth]`.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const PUSH_SCRIPT: &str = "
    const endpoint = (subscription) => {
        if (!subscription) return null;
        const { endpoint, keys } = subscription.toJSON();
        return [endpoint, keys.p256dh, keys.auth];
    };
    const failure = (error) => error?.name === 'NotAllowedError' ? 'Denied'
        : error?.name === 'NotSupportedError' ? 'Unsupported' : 'Failed';
    const subscription = async () =>
        (await navigator.serviceWorker.getRegistration())?.pushManager.getSubscription();
    const key = (base64url) => {
        const base64 = base64url.replace(/-/g, '+').replace(/_/g, '/');
        const text = atob(base64 + '='.repeat((4 - base64.length % 4) % 4));
        return Uint8Array.from(text, (char) => char.charCodeAt(0));
    };
    const active = (registration) => new Promise((done, fail) => {
        const worker = registration.installing ?? registration.waiting;
        if (registration.active || !worker) return done();
        worker.addEventListener('statechange', () => {
            if (worker.state === 'activated') done();
            if (worker.state === 'redundant') fail(new Error('redundant'));
        });
    });
    const current = async () => {
        try { return endpoint(await subscription()); } catch (error) { return failure(error); }
    };
    const subscribe = async (script, vapid) => {
        try {
            const registration = await navigator.serviceWorker.register(script);
            await active(registration);
            return endpoint(await registration.pushManager.subscribe({
                userVisibleOnly: true,
                applicationServerKey: key(vapid),
            }));
        } catch (error) { return failure(error); }
    };
    const unsubscribe = async () => {
        try { await (await subscription())?.unsubscribe(); return null; } catch (error) { return failure(error); }
    };";

#[cfg(target_arch = "wasm32")]
mod web {
    use js_sys::{Array, Function, Promise, Reflect};
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;

    use super::{PUSH_SCRIPT, PushApi, PushEndpoint, PushError, PushOptions, Subscribed};

    /// Whether the page has a service worker container and a Push API: both need
    /// a secure context.
    pub(super) fn has_push() -> bool {
        let Some(window) = web_sys::window() else {
            return false;
        };
        let defined = |target: &JsValue, name: &str| {
            Reflect::get(target, &JsValue::from_str(name)).is_ok_and(|value| !value.is_undefined())
        };
        defined(&window.navigator(), "serviceWorker") && defined(&window, "PushManager")
    }

    /// Runs `call` after the shared script and decodes its answer.
    fn run(arguments: &str, call: &str, values: &[JsValue]) -> Subscribed<Option<PushEndpoint>> {
        let script = Function::new_with_args(arguments, &format!("{PUSH_SCRIPT}\nreturn {call};"));
        let values: Array = values.iter().collect();
        let answer = script.apply(&JsValue::NULL, &values);
        Box::pin(async move {
            let promise = answer
                .ok()
                .and_then(|answer| answer.dyn_into::<Promise>().ok())
                .ok_or(PushError::Failed)?;
            let answer = JsFuture::from(promise)
                .await
                .map_err(|_| PushError::Failed)?;
            if let Some(error) = answer.as_string() {
                return Err(PushError::from_name(&error));
            }
            if answer.is_null() || answer.is_undefined() {
                return Ok(None);
            }
            let fields = Array::from(&answer);
            let field = |index| fields.get(index).as_string().ok_or(PushError::Failed);
            Ok(Some(PushEndpoint {
                endpoint: field(0)?,
                p256dh: field(1)?,
                auth: field(2)?,
            }))
        })
    }

    pub(super) struct WebPush;

    impl PushApi for WebPush {
        fn current(&self) -> Subscribed<Option<PushEndpoint>> {
            run("", "current()", &[])
        }

        fn subscribe(&self, options: &PushOptions) -> Subscribed<PushEndpoint> {
            let subscribed = run(
                "script, vapid",
                "subscribe(script, vapid)",
                &[
                    JsValue::from_str(&options.service_worker),
                    JsValue::from_str(&options.vapid_public_key),
                ],
            );
            Box::pin(async move { subscribed.await?.ok_or(PushError::Failed) })
        }

        fn unsubscribe(&self) -> Subscribed<()> {
            let done = run("", "unsubscribe()", &[]);
            Box::pin(async move { done.await.map(|_| ()) })
        }
    }

    pub(super) static PUSH: WebPush = WebPush;
}

/// `None` without a service worker and Push API: Blitz, a WebView, a server,
/// an insecure origin. Call it after mount.
pub(crate) fn push() -> Option<&'static dyn PushApi> {
    #[cfg(target_arch = "wasm32")]
    return web::has_push().then_some(&web::PUSH as &'static dyn PushApi);
    #[cfg(not(target_arch = "wasm32"))]
    return None;
}
