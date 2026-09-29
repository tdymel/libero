//! Notifications the operating system draws, from the Notifications API.

use std::{future::Future, pin::Pin};

use dioxus::prelude::Callback;

use super::PermissionState;

/// One system notification. Showing another with the same `tag` replaces it.
#[derive(Clone, PartialEq, Default)]
pub struct SystemNotification {
    pub title: String,
    pub body: Option<String>,
    /// An image URL.
    pub icon: Option<String>,
    pub tag: Option<String>,
    /// Asks the platform for no sound or vibration. A hint only.
    pub silent: bool,
    /// Runs when the user clicks it while this component is mounted, after the
    /// click focused the window. Behind a service worker, only if it posts the click back.
    pub on_click: Option<Callback<()>>,
}

impl SystemNotification {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }
}

/// Why a notification did not show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemNotificationError {
    /// No Notifications API on this target.
    Unsupported,
    /// The permission is not granted.
    Denied,
    /// The platform refused for another reason.
    Failed,
}

impl SystemNotificationError {
    #[cfg_attr(any(feature = "native", target_os = "android"), allow(dead_code))]
    pub(crate) fn from_name(name: &str) -> Self {
        match name {
            "Unsupported" => Self::Unsupported,
            "Denied" => Self::Denied,
            _ => Self::Failed,
        }
    }
}

/// What the user did to a shown notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NotificationEvent {
    Click,
    Close,
}

impl NotificationEvent {
    #[cfg_attr(any(feature = "native", target_os = "android"), allow(dead_code))]
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "click" => Some(Self::Click),
            "close" => Some(Self::Close),
            _ => None,
        }
    }
}

/// A shown notification. Dropping it stops its events but leaves it on screen.
pub(crate) trait ShownNotification {
    fn close(&self);
}

pub(crate) type Shown =
    Pin<Box<dyn Future<Output = Result<Box<dyn ShownNotification>, SystemNotificationError>>>>;

pub(crate) type Answer<T> = Pin<Box<dyn Future<Output = T>>>;

pub(crate) trait SystemNotificationApi {
    /// `Notification.permission`; `None` where the page has no `Notification`.
    fn probe(&self) -> Answer<Option<PermissionState>>;
    /// Prompts if the user has not answered yet.
    fn request(&self) -> Answer<PermissionState>;
    fn show(
        &self,
        notification: &SystemNotification,
        events: Box<dyn Fn(NotificationEvent)>,
    ) -> Shown;
}

/// Maps `Notification.permission`; `default` means a request would prompt.
#[cfg_attr(any(feature = "native", target_os = "android"), allow(dead_code))]
pub(crate) fn permission_of(name: Option<&str>) -> Option<PermissionState> {
    Some(match name? {
        "default" => PermissionState::Prompt,
        state => PermissionState::from_name(state),
    })
}

/// Shared by the web and the WebView. `show` resolves to an error name or
/// `{ close, detach }`; without a `Notification` constructor (Chrome on Android)
/// it falls back to the page's service worker, which reports clicks and closes
/// by posting `{ libero: data.libero, event: 'click' | 'close' }` to the page.
#[cfg_attr(any(feature = "native", target_os = "android"), allow(dead_code))]
pub(crate) const NOTIFICATION_SCRIPT: &str = "
    const probe = () => typeof Notification === 'undefined' ? null : Notification.permission;
    const request = async () => typeof Notification === 'undefined'
        ? null : await Notification.requestPermission();
    const show = async (title, options, send) => {
        if (typeof Notification === 'undefined') return 'Unsupported';
        if (Notification.permission !== 'granted') return 'Denied';
        try {
            const shown = new Notification(title, options);
            shown.onclick = () => { window.focus(); send('click'); };
            shown.onclose = () => send('close');
            return {
                close: () => shown.close(),
                detach: () => { shown.onclick = null; shown.onclose = null; },
            };
        } catch (error) {
            const workers = navigator.serviceWorker;
            const registration = await workers?.getRegistration();
            if (!registration) return 'Failed';
            const token = `${Date.now()}-${Math.random()}`;
            const data = { libero: token };
            try { await registration.showNotification(title, { ...options, data }); } catch (error) { return 'Failed'; }
            const listen = (event) => {
                if (event.data?.libero !== token) return;
                if (event.data.event === 'click') { window.focus(); send('click'); }
                if (event.data.event === 'close') send('close');
            };
            workers.addEventListener('message', listen);
            workers.startMessages();
            const close = async () => {
                for (const shown of await registration.getNotifications()) {
                    if (shown.data?.libero === token) shown.close();
                }
            };
            return { close, detach: () => workers.removeEventListener('message', listen) };
        }
    };";

/// The `NotificationOptions` dictionary.
#[cfg_attr(any(feature = "native", target_os = "android"), allow(dead_code))]
pub(crate) fn options_of(notification: &SystemNotification) -> serde_json::Value {
    let mut options = serde_json::json!({ "silent": notification.silent });
    for (name, value) in [
        ("body", &notification.body),
        ("icon", &notification.icon),
        ("tag", &notification.tag),
    ] {
        if let Some(value) = value {
            options[name] = value.as_str().into();
        }
    }
    options
}

#[cfg(target_arch = "wasm32")]
mod web {
    use std::rc::Rc;

    use js_sys::{Function, JSON, Object, Promise, Reflect};
    use wasm_bindgen::{JsCast, JsValue, closure::Closure};
    use wasm_bindgen_futures::JsFuture;

    use super::{
        Answer, NOTIFICATION_SCRIPT, NotificationEvent, Shown, ShownNotification,
        SystemNotification, SystemNotificationApi, SystemNotificationError, options_of,
        permission_of,
    };
    use crate::platform::PermissionState;

    fn script(arguments: &str, body: &str) -> Function {
        Function::new_with_args(arguments, &format!("{NOTIFICATION_SCRIPT}\n{body}"))
    }

    async fn settle(value: Result<JsValue, JsValue>) -> Option<JsValue> {
        let value = value.ok()?;
        match value.dyn_into::<Promise>() {
            Ok(promise) => JsFuture::from(promise).await.ok(),
            Err(value) => Some(value),
        }
    }

    pub(super) fn has_notification() -> bool {
        web_sys::window()
            .and_then(|window| Reflect::get(&window, &JsValue::from_str("Notification")).ok())
            .is_some_and(|notification| !notification.is_undefined())
    }

    pub(super) struct WebSystemNotification;

    impl SystemNotificationApi for WebSystemNotification {
        fn probe(&self) -> Answer<Option<PermissionState>> {
            let state = script("", "return probe();").call0(&JsValue::NULL);
            Box::pin(async move { permission_of(settle(state).await?.as_string().as_deref()) })
        }

        fn request(&self) -> Answer<PermissionState> {
            let state = script("", "return request();").call0(&JsValue::NULL);
            Box::pin(async move {
                settle(state)
                    .await
                    .and_then(|state| permission_of(state.as_string().as_deref()))
                    .unwrap_or(PermissionState::Unsupported)
            })
        }

        fn show(
            &self,
            notification: &SystemNotification,
            events: Box<dyn Fn(NotificationEvent)>,
        ) -> Shown {
            let options = JSON::parse(&options_of(notification).to_string())
                .unwrap_or_else(|_| Object::new().into());
            let send = Closure::<dyn Fn(JsValue)>::new(move |name: JsValue| {
                if let Some(event) = name
                    .as_string()
                    .as_deref()
                    .and_then(NotificationEvent::from_name)
                {
                    events(event);
                }
            });
            let shown = script("title, options, send", "return show(title, options, send);").call3(
                &JsValue::NULL,
                &JsValue::from_str(&notification.title),
                &options,
                send.as_ref(),
            );
            Box::pin(async move {
                let shown = settle(shown).await.ok_or(SystemNotificationError::Failed)?;
                if let Some(error) = shown.as_string() {
                    return Err(SystemNotificationError::from_name(&error));
                }
                Ok(Box::new(WebShown {
                    shown,
                    _send: Rc::new(send),
                }) as Box<dyn ShownNotification>)
            })
        }
    }

    struct WebShown {
        shown: JsValue,
        _send: Rc<Closure<dyn Fn(JsValue)>>,
    }

    impl WebShown {
        fn call(&self, member: &str) {
            if let Ok(method) = Reflect::get(&self.shown, &JsValue::from_str(member))
                && let Ok(method) = method.dyn_into::<Function>()
            {
                let _ = method.call0(&JsValue::NULL);
            }
        }
    }

    impl ShownNotification for WebShown {
        fn close(&self) {
            self.call("close");
        }
    }

    impl Drop for WebShown {
        fn drop(&mut self) {
            self.call("detach");
        }
    }

    pub(super) static SYSTEM_NOTIFICATION: WebSystemNotification = WebSystemNotification;
}

#[cfg(all(target_os = "android", not(feature = "native")))]
#[path = "system_notification_android.rs"]
mod android;
#[cfg(all(target_os = "linux", any(feature = "native", feature = "desktop")))]
#[path = "system_notification_freedesktop.rs"]
#[cfg_attr(all(test, feature = "native"), allow(dead_code))]
mod freedesktop;

/// The desktop WebView on Linux: WebKitGTK denies the page's `Notification`, so the
/// session bus shows them; a liveview page keeps the browser's (todo 1470).
#[cfg(all(target_os = "linux", feature = "desktop", not(feature = "native")))]
mod desktop {
    use super::{
        Answer, NotificationEvent, Shown, SystemNotification, SystemNotificationApi,
        SystemNotificationError, freedesktop,
    };
    use crate::platform::PermissionState;
    use crate::platform::backend::{webview_system_notification, wry_page};

    type Api = &'static dyn SystemNotificationApi;

    /// The page's impl is read before the first `await`, while a scope is current.
    async fn route(page: Option<Api>) -> Option<Api> {
        let page = page?;
        Some(if wry_page().await {
            &freedesktop::SYSTEM_NOTIFICATION
        } else {
            page
        })
    }

    pub(super) struct DesktopNotification;

    pub(super) static SYSTEM_NOTIFICATION: DesktopNotification = DesktopNotification;

    impl SystemNotificationApi for DesktopNotification {
        fn probe(&self) -> Answer<Option<PermissionState>> {
            let page = webview_system_notification();
            Box::pin(async move { route(page).await?.probe().await })
        }

        fn request(&self) -> Answer<PermissionState> {
            let page = webview_system_notification();
            Box::pin(async move {
                match route(page).await {
                    Some(api) => api.request().await,
                    None => PermissionState::Unsupported,
                }
            })
        }

        fn show(
            &self,
            notification: &SystemNotification,
            events: Box<dyn Fn(NotificationEvent)>,
        ) -> Shown {
            let (page, notification) = (webview_system_notification(), notification.clone());
            Box::pin(async move {
                let api = route(page)
                    .await
                    .ok_or(SystemNotificationError::Unsupported)?;
                api.show(&notification, events).await
            })
        }
    }
}

/// `None` without a Notifications API: Blitz off Linux, a server, a browser without one.
/// A WebView and Blitz on Linux answer `Some`; [`probe`](SystemNotificationApi::probe) tells.
/// A desktop WebView app turns on `desktop` for native notifications on Linux.
pub(crate) fn system_notification() -> Option<&'static dyn SystemNotificationApi> {
    #[cfg(target_arch = "wasm32")]
    return web::has_notification()
        .then_some(&web::SYSTEM_NOTIFICATION as &'static dyn SystemNotificationApi);
    // The unit tests must not pop notifications on the desktop running them.
    #[cfg(all(target_os = "linux", feature = "native", not(test)))]
    return Some(&freedesktop::SYSTEM_NOTIFICATION);
    #[cfg(all(
        not(target_arch = "wasm32"),
        any(not(target_os = "linux"), test),
        feature = "native"
    ))]
    return None;
    #[cfg(all(target_os = "android", not(feature = "native")))]
    return Some(&android::SYSTEM_NOTIFICATION);
    #[cfg(all(target_os = "linux", feature = "desktop", not(feature = "native")))]
    return super::backend::webview_system_notification()
        .map(|_| &desktop::SYSTEM_NOTIFICATION as &'static dyn SystemNotificationApi);
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "android"),
        not(feature = "native"),
        not(all(target_os = "linux", feature = "desktop"))
    ))]
    return super::backend::webview_system_notification();
}

/// Raises the desktop WebView's window on a click: its `window.focus()` may not (todo 1424).
/// Without a `DesktopContext` (Blitz, liveview, tests) it does nothing.
pub(crate) fn raise_window() {
    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    if let Some(desktop) = dioxus::prelude::try_consume_context::<dioxus_desktop::DesktopContext>()
    {
        desktop.window.set_focus();
    }
}
