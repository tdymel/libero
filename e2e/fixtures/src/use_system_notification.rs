//! `use_system_notification` and `use_push_subscription`: their state in text,
//! a request, a show, a close and a subscribe toggle.

use dioxus::prelude::*;
use libero::hooks::{
    NotificationAction, PushOptions, SystemNotification, use_push_subscription,
    use_system_notification,
};

use crate::Routes;

pub const ROUTES: Routes = &[("/use-system-notification", || rsx! { Notifying {} })];

/// A throwaway P-256 public key; the push service may still refuse it offline.
const VAPID_KEY: &str =
    "BEApFB2FJLU4nU88TlwcYxftQLOwEfitLq0Soa1vU2frFGcPu9AAabiV8zj0k7J-WrGOUMAaAmsMnnlNrMsWHZk";

#[component]
fn Notifying() -> Element {
    let mut notifier = use_system_notification();
    let mut push = use_push_subscription(PushOptions {
        service_worker: "/sw.js".into(),
        vapid_public_key: VAPID_KEY.into(),
    });
    let mut clicks = use_signal(|| 0);
    let clicked = use_callback(move |()| clicks += 1);
    let mut action = use_signal(String::new);
    let acted = use_callback(move |id: String| action.set(id));

    rsx! {
        button { id: "request", onclick: move |_| notifier.request(), "Request" }
        button {
            id: "show",
            onclick: move |_| notifier.show(SystemNotification {
                body: Some("From the fixture".into()),
                tag: Some("fixture".into()),
                on_click: Some(clicked),
                ..SystemNotification::new("Fixture")
            }),
            "Show"
        }
        button {
            id: "show-actions",
            onclick: move |_| notifier.show(SystemNotification {
                tag: Some("fixture".into()),
                actions: vec![
                    NotificationAction::new("open", "Open"),
                    NotificationAction::new("retry", "Retry"),
                ],
                on_action: Some(acted),
                ..SystemNotification::new("Fixture")
            }),
            "Show with actions"
        }
        button { id: "close", onclick: move |_| notifier.close("fixture"), "Close" }
        button { id: "subscribe", onclick: move |_| push.subscribe(), "Subscribe" }
        p { id: "supported", "{notifier.is_supported()}" }
        p { id: "permission", "{notifier.permission():?}" }
        p { id: "error", "{notifier.error():?}" }
        p { id: "pending", "{notifier.is_pending()}" }
        p { id: "clicks", "{clicks}" }
        p { id: "action", "{action}" }
        p { id: "push-supported", "{push.is_supported()}" }
        p { id: "push-error", "{push.error():?}" }
        p { id: "push-pending", "{push.is_pending()}" }
        p { id: "subscription", "{push.subscription().is_some()}" }
    }
}
