use dioxus::core::Task;
use dioxus::prelude::spawn;
use serde_json::{Value, json};

use super::{Slot, eval, eval_with, runs_scripts};
use crate::platform::permission::PermissionState;
use crate::platform::system_notification::{
    Answer, NOTIFICATION_SCRIPT, NotificationEvent, Shown, ShownNotification, SystemNotification,
    SystemNotificationApi, SystemNotificationError, options_of, permission_of,
};

/// Android has its own, over JNI (`platform::system_notification`).
pub(crate) fn system_notification() -> Option<&'static dyn SystemNotificationApi> {
    runs_scripts().then_some(&SYSTEM_NOTIFICATION as &'static dyn SystemNotificationApi)
}

/// Whether the page is a wry WebView, which defines `window.ipc`; a liveview tab has none.
#[cfg(all(target_os = "linux", feature = "desktop"))]
pub(crate) fn wry_page() -> Answer<bool> {
    static WRY: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if let Some(wry) = WRY.get() {
        return Box::pin(std::future::ready(*wry));
    }
    let read = eval("return typeof window.ipc?.postMessage === 'function';");
    Box::pin(async move {
        match read.join::<bool>().await {
            Ok(wry) => *WRY.get_or_init(|| wry),
            Err(_) => false,
        }
    })
}

struct WebViewSystemNotification;

static SYSTEM_NOTIFICATION: WebViewSystemNotification = WebViewSystemNotification;

impl SystemNotificationApi for WebViewSystemNotification {
    fn probe(&self) -> Answer<Option<PermissionState>> {
        let read = eval(&format!("{NOTIFICATION_SCRIPT}\nreturn probe();"));
        Box::pin(async move {
            permission_of(
                read.join::<Option<String>>()
                    .await
                    .ok()
                    .flatten()
                    .as_deref(),
            )
        })
    }

    /// Parked like [`GeolocationApi::current`]: the prompt can outlast a `join`.
    fn request(&self) -> Answer<PermissionState> {
        let slot = Slot::new();
        let script = eval(&format!(
            "{NOTIFICATION_SCRIPT}
            dioxus.send(await request());
            {}",
            slot.park("")
        ));
        Box::pin(async move {
            let mut script = script;
            let state = script.recv::<Option<String>>().await.ok().flatten();
            drop(slot);
            permission_of(state.as_deref()).unwrap_or(PermissionState::Unsupported)
        })
    }

    /// Sends `shown` or an error name first, then `click` and `close`.
    fn show(
        &self,
        notification: &SystemNotification,
        events: Box<dyn Fn(NotificationEvent)>,
    ) -> Shown {
        let slot = Slot::new();
        let script = eval_with(
            json!([notification.title, options_of(notification)]),
            &format!(
                "{NOTIFICATION_SCRIPT}
                const shown = await show(data[0], data[1], (event) => dioxus.send(event));
                if (typeof shown === 'string') return dioxus.send(shown);
                dioxus.send('shown');
                {}
                shown.detach();",
                slot.park("close: () => shown.close()")
            ),
        );
        Box::pin(async move {
            let mut script = script;
            let first = script.recv::<String>().await;
            match first.as_deref() {
                Ok("shown") => {}
                Ok(error) => return Err(SystemNotificationError::from_name(error)),
                Err(_) => return Err(SystemNotificationError::Unsupported),
            }
            let task = spawn(async move {
                while let Ok(name) = script.recv::<String>().await {
                    if let Some(event) = NotificationEvent::from_name(&name) {
                        events(event);
                    }
                }
            });
            Ok(Box::new(WebViewShown { task, slot }) as Box<dyn ShownNotification>)
        })
    }
}

/// Dropping the slot ends the parked script, which detaches the handlers.
struct WebViewShown {
    task: Task,
    slot: Slot,
}

impl ShownNotification for WebViewShown {
    fn close(&self) {
        self.slot.call("close", Value::Null);
    }
}

impl Drop for WebViewShown {
    fn drop(&mut self) {
        self.task.cancel();
    }
}
