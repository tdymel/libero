use crate::components::{Demo, DemoValues, DocPage, Wrap, a11y};
use crate::site::LOGO;
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Flex, Text},
    hooks::{
        NotificationAction, PermissionState, PushError, PushOptions, SystemNotification,
        SystemNotificationError, SystemNotifier, use_push_subscription, use_system_notification,
    },
    sx::sx,
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
let mut last = use_signal(String::new);
let clicked = use_callback(move |()| last.set("Notification clicked".into()));
let acted = use_callback(move |action: String| last.set(format!("Action pressed: {action}")));
// The page says it too: a system notification is never the only channel.
let status = match notifier.error() {
    Some(SystemNotificationError::Denied) => "Notifications refused".to_string(),
    Some(SystemNotificationError::Unsupported) => "No system notifications here".to_string(),
    Some(SystemNotificationError::Failed) => "The notification did not show".to_string(),
    None => last(),
};
let permission = match notifier.permission() {
    PermissionState::Granted => "granted",
    PermissionState::Denied => "denied",
    PermissionState::Prompt => "not asked yet",
    PermissionState::Unknown => "unknown",
    PermissionState::Unsupported => "unsupported here",
};
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
                    icon: Some("/icon.png".into()),
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
            // POST it to the app's server, which pushes with its VAPID private key.
            Text {
                size: "sm",
                sx: sx().with("overflow-wrap", "anywhere"),
                "Push endpoint: {subscription.endpoint}"
            }
        }
        Text { size: "sm", "Permission: {permission}. Push: {push_status}." }
    }
}"#
    .to_string()
}

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

#[component]
fn Notify() -> Element {
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
                    "Android reads Prompt until the first request and Denied after a refusal, also after a restart; it cannot tell a dismissed dialog from a refusal.",
                    "Android shows icon as the large icon, the status bar the app's libero_notification drawable, else the launcher icon. A tap on a notification from before a restart only opens the app.",
                    "An action on Android reopens the app, as a tap does. The web shows actions only through a service worker, which must post the press back; without one the notification shows without them.",
                    "Blitz cannot raise the window on a click: on_click runs, the window stays where it is.",
                    "Where only a service worker may show notifications (Chrome on Android), on_click runs only if the app's worker posts the click back, as the sample sw.js does.",
                    "A click after the page closed runs nothing in the page: only a worker can open a tab then.",
                    "A desktop WebView raises its window on a click only with libero's desktop feature; without it, window.focus() may not.",
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
                    " closes it; its "
                    Code { source: "actions" }
                    " are buttons whose id comes back in "
                    Code { source: "on_action" }
                    ". Read "
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
                    ". Push on Android is Firebase Cloud Messaging in the app itself, with no libero code: the markdown version of this page has a recipe."
                }
                Text {
                    "Web: a secure context (HTTPS or localhost). Desktop WebView on Linux: with libero's "
                    Code { source: "desktop" }
                    " feature, the desktop's notification server over D-Bus, as for Blitz; without it WebKitGTK denies every request. macOS and Windows WebViews go through the page's "
                    Code { source: "Notification" }
                    ", untested. Push is web only. A click focuses the page's window, then runs "
                    Code { source: "on_click" }
                    "; behind a service worker only if the worker posts it back, as the sample worker does. Android: the system's notifications over JNI, after "
                    Code { source: "notifications = { description = \"..\" }" }
                    " under "
                    Code { source: "[permissions]" }
                    " in the app's "
                    Code { source: "Dioxus.toml" }
                    "; a tap reopens the app and runs "
                    Code { source: "on_click" }
                    ". The status bar icon is a drawable named "
                    Code { source: "libero_notification" }
                    " in the app's resources, a white shape on transparent; without it the launcher icon shows. Actions are untested on macOS and Windows. Blitz on Linux: the desktop's notification server over D-Bus, no permission to ask. Blitz on macOS and Windows, and a server render: "
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
