use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Text},
    hooks::{
        NotificationAction, PermissionState, PushError, PushOptions, SystemNotification,
        SystemNotificationError, SystemNotifier, use_push_subscription, use_system_notification,
    },
    sx::sx,
};

use crate::site::LOGO;

/// A throwaway public key: its private half was never kept, so nothing can push to the demo.
const DEMO_VAPID_KEY: &str =
    "BEApFB2FJLU4nU88TlwcYxftQLOwEfitLq0Soa1vU2frFGcPu9AAabiV8zj0k7J-WrGOUMAaAmsMnnlNrMsWHZk";

fn status(notifier: &SystemNotifier, last: String) -> String {
    match notifier.error() {
        Some(SystemNotificationError::Denied) => "Notifications refused".to_string(),
        Some(SystemNotificationError::Unsupported) => "No system notifications here".to_string(),
        Some(SystemNotificationError::Failed) => "The notification did not show".to_string(),
        None => last,
    }
}

fn permission_text(state: PermissionState) -> &'static str {
    match state {
        PermissionState::Granted => "granted",
        PermissionState::Denied => "denied",
        PermissionState::Prompt => "not asked yet",
        PermissionState::Unknown => "unknown",
        PermissionState::Unsupported => "unsupported here",
    }
}

// demo-code: start
#[component]
pub fn Notify() -> Element {
    let mut notifier = use_system_notification();
    let mut push = use_push_subscription(PushOptions {
        service_worker: "/sw.js".into(),
        vapid_public_key: DEMO_VAPID_KEY.into(),
    });
    let mut last = use_signal(String::new);
    let clicked = use_callback(move |()| last.set("Notification clicked".into()));
    let acted = use_callback(move |action: String| last.set(format!("Action pressed: {action}")));
    let status = status(&notifier, last());
    let permission = permission_text(notifier.permission());
    let push_status = match (push.is_supported(), push.error()) {
        (false, _) | (true, Some(PushError::Unsupported)) => "unsupported here",
        (true, Some(PushError::Denied)) => "refused, notifications are not allowed",
        (true, Some(PushError::Failed)) => "the subscription failed",
        (true, None) if push.subscription().is_some() => "subscribed",
        (true, None) => "not subscribed",
    };

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            Flex { gap: "sm", wrap: "wrap",
                Button { onclick: move |_| notifier.request(), "Allow notifications" }
                Button {
                    onclick: move |_| notifier.show(SystemNotification {
                        body: Some("3 tests failed in the nightly run".into()),
                        icon: Some(LOGO.to_string()),
                        tag: Some("demo".into()),
                        on_click: Some(clicked),
                        actions: vec![
                            NotificationAction::new("report", "Open report"),
                            NotificationAction::new("retry", "Run again"),
                        ],
                        on_action: Some(acted),
                        ..SystemNotification::new("Build finished")
                    }),
                    "Notify"
                }
                Button { variant: "outlined", onclick: move |_| notifier.close("demo"), "Close it" }
                Button {
                    variant: "outlined",
                    onclick: move |_| if push.subscription().is_some() { push.unsubscribe() } else { push.subscribe() },
                    if push.subscription().is_some() { "Unsubscribe from push" } else { "Subscribe to push" }
                }
            }
            div { role: "status", "{status}" }
            if let Some(subscription) = push.subscription() {
                // An endpoint is one unbroken 100-200 character word (1.4.10).
                Text {
                    size: "sm",
                    sx: sx().with("overflow-wrap", "anywhere"),
                    "Push endpoint: {subscription.endpoint}"
                }
            }
            Text { size: "sm", "Permission: {permission}. Push: {push_status}." }
        }
    }
}
// demo-code: end
