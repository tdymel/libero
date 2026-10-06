//! `Skeleton`, for the reduced-motion checks and the docs' grace recipe.

use std::time::Duration;

use dioxus::prelude::*;
use libero::components::{Box, Button, Flex, Skeleton, Text};
use libero::platform::timer;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/skeleton", || rsx! { SkeletonPage {} }),
    (
        "/skeleton-grace/fast",
        || rsx! { GracePage { latency: 50 } },
    ),
    (
        "/skeleton-grace/slow",
        || rsx! { GracePage { latency: 1000 } },
    ),
    ("/skeleton-fixed", || rsx! { FixedPage {} }),
];

/// A loaded skeleton, offset from the corner, around a `position: fixed` child.
#[component]
fn FixedPage() -> Element {
    rsx! {
        Box { padding: "64px",
            Skeleton { visible: false, animate: false,
                div {
                    id: "pinned",
                    style: "position: fixed; top: 0; left: 0; width: 10px; height: 10px; background: black;",
                }
            }
        }
    }
}

/// A standalone shape and a wrapper, both pulsing until "Reload" starts a
/// wait: the pulse is motion an interaction starts.
#[component]
fn SkeletonPage() -> Element {
    let mut loading = use_signal(|| false);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button {
                id: "reload",
                variant: "outlined",
                onclick: move |_| loading.toggle(),
                if loading() { "Stop loading" } else { "Reload" }
            }
            Box { id: "profile", "aria-busy": if loading() { "true" } else { "false" },
                Flex { direction: "row", gap: "sm", align: "center",
                    Skeleton { id: "avatar", visible: loading(), circle: true, height: "40px",
                        Text { "AL" }
                    }
                    Skeleton { id: "name", visible: loading(),
                        Text { id: "name-text", "Ada Lovelace" }
                    }
                }
            }
        }
    }
}

/// "Load" mounts the card, so a test can watch from before its first frame.
#[component]
fn GracePage(latency: u64) -> Element {
    let mut mounted = use_signal(|| false);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "load", variant: "outlined", onclick: move |_| mounted.set(true), "Load" }
            if mounted() {
                GraceCard { latency }
            }
        }
    }
}

/// The docs page's grace recipe (todo 107), with a timer standing in for the
/// fetch so its latency is known.
#[component]
fn GraceCard(latency: u64) -> Element {
    let mut answered = use_signal(|| false);
    let mut fetch = use_signal(|| {
        timer().map(|timer| {
            timer.after(
                Duration::from_millis(latency),
                std::boxed::Box::new(move || answered.set(true)),
            )
        })
    });
    use_drop(move || fetch.set(None));
    let loading = !answered();

    let mut slow = use_signal(|| false);
    let mut grace = use_signal(|| {
        timer().map(|timer| {
            timer.after(
                Duration::from_millis(200),
                std::boxed::Box::new(move || slow.set(true)),
            )
        })
    });
    use_drop(move || grace.set(None));

    rsx! {
        div {
            id: "region",
            "aria-busy": loading,
            opacity: if loading && !slow() { "0" } else { "1" },
            Skeleton { id: "card", visible: loading,
                Text { "Ada Lovelace" }
            }
        }
    }
}
