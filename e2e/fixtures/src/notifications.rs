//! `Notifications`.

use dioxus::prelude::*;
use libero::{
    components::{
        Button, Flex, NotificationData, NotificationLive, NotificationOptions, NotificationScope,
        Notifications, Paper, Text, use_notifications, use_notifications_with,
    },
    theme::AutoClose,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/notifications", || rsx! { NotificationsPage {} }),
    (
        "/notifications-clear",
        || rsx! { NotificationsClearPage {} },
    ),
];

/// The component the framework was **not** built for.
///
/// Everything in `Suite` assumes a static page: open a route, measure it,
/// snapshot it. A notification is transient - it appears on an action and
/// removes itself on a timer - so its accessibility tree is a function of time,
/// and its live regions say different things at different moments.
///
/// `#notify` and `#notify-assertive` pin `auto_close` off. A fixture that
/// disappears while being measured is not a test of the component, it is a
/// race. `#notify-timed` does close itself, after [`TIMED_AUTO_CLOSE_MS`], and
/// the test holds that timer on the page's clock and fires it by hand
/// (`tests/all/notifications.rs`), so nothing waits on real time.
#[component]
fn NotificationsPage() -> Element {
    let notify = use_notifications();

    rsx! {
        Notifications {}
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button {
                id: "notify",
                onclick: move |_| {
                    notify
                        .show_with(
                            "Saved to your library",
                            NotificationOptions {
                                auto_close: Some(AutoClose::Never),
                                ..Default::default()
                            },
                        );
                },
                "Notify"
            }
            Button {
                id: "notify-assertive",
                onclick: move |_| {
                    notify
                        .show_with(
                            NotificationData {
                                title: Some("Upload failed".into()),
                                message: "The file is larger than 10 MB".into(),
                                ..Default::default()
                            },
                            NotificationOptions {
                                auto_close: Some(AutoClose::Never),
                                live: NotificationLive::Assertive,
                                ..Default::default()
                            },
                        );
                },
                "Notify assertively"
            }
            Button {
                id: "notify-timed",
                onclick: move |_| {
                    notify
                        .show_with(
                            "Draft saved",
                            NotificationOptions {
                                auto_close: Some(AutoClose::After(TIMED_AUTO_CLOSE_MS)),
                                ..Default::default()
                            },
                        );
                },
                "Notify for a while"
            }
        }
    }
}

/// Two notifications, each with a "Clear all" button, so `clear()` runs with
/// focus inside one (todo 440). Its own route, so `/notifications`' baselines
/// stay as they are.
#[component]
fn NotificationsClearPage() -> Element {
    let notify = use_notifications_with(|s: NotificationScope<String>| {
        let all = use_notifications();
        rsx! {
            Paper {
                Text { "{s.args()}" }
                Button { class: "clear-all", onclick: move |_| all.clear(), "Clear all" }
            }
        }
    });

    rsx! {
        Notifications {}
        Button {
            id: "notify",
            onclick: move |_| {
                for message in ["First", "Second"] {
                    notify
                        .show_with(
                            message,
                            NotificationOptions {
                                auto_close: Some(AutoClose::Never),
                                ..Default::default()
                            },
                        );
                }
            },
            "Notify twice"
        }
    }
}

/// A delay nothing else on the page schedules, so the test's clock can hold
/// exactly this timer and pass every other one through.
const TIMED_AUTO_CLOSE_MS: u32 = 4321;
