//! `Notifications`.

use dioxus::prelude::*;
use libero::{
    components::{
        Button, Flex, Input, NotificationData, NotificationLive, NotificationOptions,
        NotificationScope, Notifications, Paper, Text, use_notifications, use_notifications_with,
    },
    theme::{AutoClose, Placement},
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/notifications", || rsx! { NotificationsPage {} }),
    (
        "/notifications-clear",
        || rsx! { NotificationsClearPage {} },
    ),
    ("/notifications-host", || rsx! { NotificationsHostPage {} }),
    (
        "/notifications-host-inside",
        || rsx! { NotificationsHostInsidePage {} },
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

/// A contained host whose `placement` changes, and which unmounts from inside
/// one of its notifications (todo 577). `#notify` sits outside the host, so it
/// survives it; it only counts, and `Feeder` inside the host shows.
#[component]
fn NotificationsHostPage() -> Element {
    let mounted = use_context_provider(|| Signal::new(true));
    let mut placement = use_signal(|| Placement::BottomEnd);
    let mut requested = use_context_provider(|| Signal::new(0u32));

    rsx! {
        Button { id: "notify", onclick: move |_| requested += 1, "Notify" }
        Button { id: "move", onclick: move |_| placement.set(Placement::TopStart), "Move" }
        if mounted() {
            Notifications { contained: true, placement: Input::Value(placement()),
                Feeder {}
            }
        }
    }
}

#[component]
fn Feeder() -> Element {
    let notify = use_notifications_with(|s: NotificationScope<String>| {
        let mut mounted = use_context::<Signal<bool>>();
        rsx! {
            Paper {
                Text { "{s.args()}" }
                Button { class: "drop-host", onclick: move |_| mounted.set(false), "Drop host" }
            }
        }
    });
    let requested = use_context::<Signal<u32>>();
    use_effect(move || {
        let count = requested();
        if count > 0 {
            notify.show_with(
                format!("Message {count}"),
                NotificationOptions {
                    auto_close: Some(AutoClose::Never),
                    ..Default::default()
                },
            );
        }
    });
    rsx! {}
}

/// A contained host whose opener sits inside it, so the element focus came
/// from goes with the host (todo 589). `#before` is the focusable before it.
#[component]
fn NotificationsHostInsidePage() -> Element {
    let mounted = use_context_provider(|| Signal::new(true));

    rsx! {
        Button { id: "before", "Before" }
        if mounted() {
            Notifications { contained: true,
                InsideFeeder {}
            }
        }
        Button { id: "after", "After" }
    }
}

#[component]
fn InsideFeeder() -> Element {
    let notify = use_notifications_with(|s: NotificationScope<String>| {
        let mut mounted = use_context::<Signal<bool>>();
        rsx! {
            Paper {
                Text { "{s.args()}" }
                Button { class: "drop-host", onclick: move |_| mounted.set(false), "Drop host" }
            }
        }
    });
    rsx! {
        Button {
            id: "notify",
            onclick: move |_| {
                notify
                    .show_with(
                        "Inside".to_string(),
                        NotificationOptions {
                            auto_close: Some(AutoClose::Never),
                            ..Default::default()
                        },
                    );
            },
            "Notify"
        }
    }
}

/// A delay nothing else on the page schedules, so the test's clock can hold
/// exactly this timer and pass every other one through.
const TIMED_AUTO_CLOSE_MS: u32 = 4321;
