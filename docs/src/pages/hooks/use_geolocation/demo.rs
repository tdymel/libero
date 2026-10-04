use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Text},
    hooks::{Geolocation, GeolocationError, GeolocationOptions, PermissionState, use_geolocation},
};

fn status(location: &Geolocation) -> &'static str {
    match (location.error(), location.position()) {
        (Some(GeolocationError::Denied), _) => "Location refused",
        (Some(_), _) => "No location found",
        (None, Some(_)) if location.is_watching() => "Following your location",
        (None, Some(_)) => "Location found",
        (None, None) if location.is_pending() => "Locating",
        (None, None) => "",
    }
}

// demo-code: start
#[component]
pub fn ShareLocation() -> Element {
    let mut location = use_geolocation(GeolocationOptions::default());
    let status = status(&location);
    let permission = match location.permission() {
        PermissionState::Granted => "granted",
        PermissionState::Denied => "denied",
        PermissionState::Prompt => "not asked yet",
        PermissionState::Unknown => "unknown",
        PermissionState::Unsupported => "unsupported here",
    };

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            Flex { gap: "sm",
                Button { onclick: move |_| location.request(), "Share location" }
                Button {
                    variant: "outlined",
                    onclick: move |_| if location.is_watching() { location.stop() } else { location.watch() },
                    if location.is_watching() { "Stop following" } else { "Follow" }
                }
            }
            div { role: "status", "{status}" }
            if let Some(fix) = location.position() {
                Text { "{fix.latitude:.2}, {fix.longitude:.2}, within {fix.accuracy:.0} m" }
            }
            Text { size: "sm", "Permission: {permission}" }
        }
    }
}
// demo-code: end
