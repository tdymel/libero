use crate::components::{Demo, DemoValues, DocPage, Wrap, a11y};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Flex, Text},
    hooks::{Geolocation, GeolocationError, PermissionState, use_geolocation},
};

/// The hook in one component, as `ShareLocation` renders it.
fn code(_: &DemoValues, _: &str) -> String {
    r#"let mut location = use_geolocation(GeolocationOptions::default());
// Announced once per outcome, never per fix.
let status = match (location.error(), location.position()) {
    (Some(GeolocationError::Denied), _) => "Location refused",
    (Some(_), _) => "No location found",
    (None, Some(_)) if location.is_watching() => "Following your location",
    (None, Some(_)) => "Location found",
    (None, None) if location.is_pending() => "Locating",
    (None, None) => "",
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
    }
}"#
    .to_string()
}

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

#[component]
fn ShareLocation() -> Element {
    let mut location = use_geolocation(Default::default());
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

#[component]
pub fn UseGeolocationPage() -> Element {
    rsx! {
        DocPage {
            title: "Geolocation",
            source: "libero/src/hooks/geolocation.rs",
            markdown: "/md/use_geolocation.md",
            accessibility: a11y()
                .handles([
                    "Mounting never prompts: the browser or OS asks only on request or watch, which you call from a user's action.",
                    "A watch ends on stop, on a denial and on unmount, so the device's location indicator goes off with it.",
                    "It announces nothing and logs no coordinates.",
                ])
                .must([
                    "Ask from a visible control whose label says what the location is for, never on page load.",
                    "Announce the outcome once in a status region (found, refused, not found), never every fix, as the demo does.",
                    "Show a visible \"following your location\" state while a watch runs, with a control that stops it.",
                    "Give a refused user another way on, such as typing an address, and say how to re-enable location in the browser or system settings.",
                    "Round coordinates you show, and keep them out of logs and storage unless the user agreed.",
                ])
                .limits([
                    "A denial is usually permanent for the site: the browser does not ask again, and libero cannot open its settings.",
                    "The Linux desktop WebView (WebKitGTK) denies every request, because wry answers no permission request there.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_geolocation(options) -> Geolocation" }
                    " reads the device's position. "
                    Code { source: "request()" }
                    " asks for one fix, "
                    Code { source: "watch()" }
                    " follows it until "
                    Code { source: "stop()" }
                    ". Read "
                    Code { source: "position()" }
                    ", "
                    Code { source: "error()" }
                    ", "
                    Code { source: "permission()" }
                    ", "
                    Code { source: "is_pending()" }
                    " and "
                    Code { source: "is_watching()" }
                    "; all are reactive."
                }
                Text {
                    Code { source: "accuracy" }
                    " is a radius in metres: a fix from Wi-Fi or IP can be kilometres wide. "
                    Code { source: "high_accuracy" }
                    " asks for GPS, slower and costlier on battery."
                }
                Text {
                    "Web: a secure context (HTTPS or localhost). Android: declare "
                    Code { source: "[permissions] location" }
                    " in Dioxus.toml; the system asks on the first request. macOS and Windows WebViews are untested. Blitz and a server render have no Geolocation API: "
                    Code { source: "is_supported()" }
                    " stays false and a request fails with "
                    Code { source: "Unsupported" }
                    "."
                }
            },

            Demo {
                component: "use_geolocation",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { ShareLocation {} },
                wrap: Wrap(code),
            }
        }
    }
}
