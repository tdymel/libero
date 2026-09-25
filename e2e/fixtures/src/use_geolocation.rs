//! `use_geolocation`: its state in text, a one-shot request and a watch toggle.

use std::time::Duration;

use dioxus::prelude::*;
use libero::hooks::{GeolocationOptions, use_geolocation};

use crate::Routes;

pub const ROUTES: Routes = &[("/use-geolocation", || rsx! { Located {} })];

#[component]
fn Located() -> Element {
    // High accuracy: the emulator's `geo fix` feeds only the GPS provider.
    let mut location = use_geolocation(GeolocationOptions {
        high_accuracy: true,
        timeout: Some(Duration::from_secs(10)),
        ..Default::default()
    });
    let position = location.position().map_or("none".to_string(), |fix| {
        format!(
            "{:.3}, {:.3} ±{:.0}",
            fix.latitude, fix.longitude, fix.accuracy
        )
    });

    rsx! {
        button { id: "locate", onclick: move |_| location.request(), "Locate" }
        button {
            id: "watch",
            onclick: move |_| if location.is_watching() { location.stop() } else { location.watch() },
            if location.is_watching() { "Stop" } else { "Watch" }
        }
        p { id: "supported", "{location.is_supported()}" }
        p { id: "permission", "{location.permission():?}" }
        p { id: "position", "{position}" }
        p { id: "error", "{location.error():?}" }
        p { id: "pending", "{location.is_pending()}" }
        p { id: "watching", "{location.is_watching()}" }
    }
}
