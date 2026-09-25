use crate::components::{Demo, DemoValues, DocPage, Wrap, a11y};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Flex, Text},
    hooks::{
        PermissionState, PushOptions, SystemNotification, SystemNotificationError, SystemNotifier,
        use_push_subscription, use_system_notification,
    },
};

/// A throwaway public key: its private half was never kept, so nothing can push to the demo.
const DEMO_VAPID_KEY: &str =
    "BEApFB2FJLU4nU88TlwcYxftQLOwEfitLq0Soa1vU2frFGcPu9AAabiV8zj0k7J-WrGOUMAaAmsMnnlNrMsWHZk";

/// The hooks in one component, as `Notify` renders them.
fn code(_: &DemoValues, _: &str) -> String {
    r#"let mut notifier = use_system_notification();
let mut push = use_push_subscription(PushOptions {
    service_worker: "/sw.js".into(),
    vapid_public_key: "<your server's VAPID public key>".into(),
});
let mut clicks = use_signal(|| 0);
let clicked = use_callback(move |()| clicks += 1);
// The page says it too: a system notification is never the only channel.
let status = match notifier.error() {
    Some(SystemNotificationError::Denied) => "Notifications refused".to_string(),
    Some(SystemNotificationError::Unsupported) => "No system notifications here".to_string(),
    Some(SystemNotificationError::Failed) => "The notification did not show".to_string(),
    None if clicks() > 0 => format!("Notification clicked {} times", clicks()),
    None => String::new(),
};

rsx! {
    Flex { direction: "column", align: "flex-start", gap: "sm",
        Flex { gap: "sm", wrap: "wrap",
            Button { onclick: move |_| notifier.request(), "Allow notifications" }
            Button {
                onclick: move |_| notifier.show(SystemNotification {
                    body: Some("Sent from the libero docs".into()),
                    tag: Some("demo".into()),
                    on_click: Some(clicked),
                    ..SystemNotification::new("Hello")
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
            // POST it to the app's server, which pushes with its VAPID private key.
            Text { size: "sm", "Push endpoint: {subscription.endpoint}" }
        }
    }
}"#
    .to_string()
}

fn status(notifier: &SystemNotifier, clicks: u32) -> String {
    match notifier.error() {
        Some(SystemNotificationError::Denied) => "Notifications refused".to_string(),
        Some(SystemNotificationError::Unsupported) => "No system notifications here".to_string(),
        Some(SystemNotificationError::Failed) => "The notification did not show".to_string(),
        None if clicks > 0 => format!("Notification clicked {clicks} times"),
        None => String::new(),
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

#[component]
fn Notify() -> Element {
    let mut notifier = use_system_notification();
    let mut push = use_push_subscription(PushOptions {
        service_worker: "/sw.js".into(),
        vapid_public_key: DEMO_VAPID_KEY.into(),
    });
    let mut clicks = use_signal(|| 0);
    let clicked = use_callback(move |()| clicks += 1);
    let status = status(&notifier, clicks());
    let permission = permission_text(notifier.permission());
    let push_status = match (push.is_supported(), push.error()) {
        (false, _) => "unsupported here".to_string(),
        (true, Some(error)) => format!("{error:?}"),
        (true, None) if push.subscription().is_some() => "subscribed".to_string(),
        (true, None) => "not subscribed".to_string(),
    };

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            Flex { gap: "sm", wrap: "wrap",
                Button { onclick: move |_| notifier.request(), "Allow notifications" }
                Button {
                    onclick: move |_| notifier.show(SystemNotification {
                        body: Some("Sent from the libero docs".into()),
                        tag: Some("demo".into()),
                        on_click: Some(clicked),
                        ..SystemNotification::new("Hello")
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
                Text { size: "sm", "Push endpoint: {subscription.endpoint}" }
            }
            Text { size: "sm", "Permission: {permission}. Push: {push_status}." }
        }
    }
}

#[component]
pub fn UseSystemNotificationPage() -> Element {
    rsx! {
        DocPage {
            title: "System notifications",
            source: "libero/src/hooks/system_notification.rs",
            markdown: "/md/use_system_notification.md",
            accessibility: a11y()
                .handles([
                    "Mounting never prompts: the browser asks only on request, show after a grant, or subscribe, which you call from a user's action.",
                    "Unmounting stops click handling; shown notifications stay for the user to dismiss.",
                    "It announces nothing: the notification is outside the page, with no live region.",
                ])
                .must([
                    "Never make a system notification the only channel: say the same in the page, where screen reader and keyboard users already are.",
                    "Never ask for the permission on page load; ask from a visible control whose label says what you will notify about.",
                    "Announce a refusal or failure once in a status region, as the demo does, and say how to re-enable notifications in the browser settings.",
                    "Let the user stop push from the page as well as in the browser: unsubscribe and tell your server.",
                ])
                .limits([
                    "A denial is usually permanent for the site: the browser does not ask again, and libero cannot open its settings.",
                    "The Android WebView has no Notifications API, and libero declares no POST_NOTIFICATIONS: native notifications and FCM need app-level Kotlin.",
                    "Where only a service worker may show notifications (Chrome on Android), clicks go to the worker, not on_click.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_system_notification() -> SystemNotifier" }
                    " shows notifications the operating system draws, outside the page; the in-app toasts are "
                    Code { source: "Notifications" }
                    ". "
                    Code { source: "request()" }
                    " asks for the permission, "
                    Code { source: "show(SystemNotification)" }
                    " shows one, "
                    Code { source: "close(tag)" }
                    " closes it. Read "
                    Code { source: "permission()" }
                    ", "
                    Code { source: "error()" }
                    ", "
                    Code { source: "is_pending()" }
                    " and "
                    Code { source: "is_supported()" }
                    "; all are reactive."
                }
                Text {
                    Code { source: "use_push_subscription(PushOptions) -> PushSubscription" }
                    " registers your service worker and subscribes with your server's VAPID public key. "
                    Code { source: "subscription()" }
                    " is the "
                    Code { source: "PushEndpoint" }
                    " your server stores and pushes to; the worker shows what arrives. Sending, VAPID signing, FCM and APNs stay on your server. The demo's key is a throwaway public key: no private key exists anywhere in the repo, so nothing ever pushes to it. A sample worker is "
                    Code { source: "docs/public/sw.js" }
                    "."
                }
                Text {
                    "Web: a secure context (HTTPS or localhost). Desktop WebViews: system notifications go through the page's "
                    Code { source: "Notification" }
                    " where the WebView has one (Linux WebKitGTK denies, macOS and Windows are untested); push is web only. Android, Blitz and a server render: "
                    Code { source: "is_supported()" }
                    " stays false and calls fail with "
                    Code { source: "Unsupported" }
                    "."
                }
            },

            Demo {
                component: "use_system_notification",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { Notify {} },
                wrap: Wrap(code),
            }
        }
    }
}
