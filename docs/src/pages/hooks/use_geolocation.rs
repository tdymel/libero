use crate::components::{Demo, DemoFile, DemoValues, DocPage, DocSection, a11y};
use dioxus::prelude::*;
use libero::components::{Code, Text};

mod demo;
use demo::ShareLocation;

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
                .example("A \"Find stores near me\" button that asks for the location on press: a status line says \"Found 3 stores\" once, and a refused user gets a postcode field instead.")
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
            },

            Demo {
                component: "ShareLocation",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { ShareLocation {} },
                file: DemoFile(include_str!("use_geolocation/demo.rs")),
            }

            DocSection {
                title: "Platforms",
                Text {
                    "Web: a secure context (HTTPS or localhost). Android: declare "
                    Code { source: "[permissions] location" }
                    " in Dioxus.toml; the system asks on the first request. macOS and Windows WebViews are untested. Blitz and a server render have no Geolocation API: "
                    Code { source: "is_supported()" }
                    " stays false and a request fails with "
                    Code { source: "Unsupported" }
                    "."
                }
            }
        }
    }
}
