use crate::components::{Control, Demo, DemoFile, DemoValues, DocPage, Remount, a11y};
use dioxus::prelude::*;
use libero::components::{Code, Text};

mod demo;
use demo::CaptureBooth;

#[component]
pub fn UseUserMediaPage() -> Element {
    rsx! {
        DocPage {
            title: "User media",
            source: "libero/src/hooks/user_media.rs",
            markdown: "/md/use_user_media.md",
            plays_media: true,
            accessibility: a11y()
                .handles([
                    "Mounting never prompts: the browser or OS asks only on start, which you call from a user's action.",
                    "Stop and unmount stop every track, so the device's camera light and recording indicator go off with them.",
                    "It announces nothing, and never hides the platform's own recording indicator.",
                ])
                .must([
                    "Start from a visible control whose label says what the camera or microphone is for, never on page load.",
                    "Show a visible \"camera on\" and \"recording\" state, with a control that stops each.",
                    "Announce start, stop and refusal once in a status region, never per frame or recording second.",
                    "Keep the preview `<video>` muted and give it an `aria_label`; a live microphone played back echoes.",
                    "Give a refused user another way on, such as a file upload, and say how to re-enable the camera in the browser or system settings.",
                ])
                .example("A \"Start camera\" button for a profile photo: the browser asks only on press, the muted preview is named \"Camera preview\", a status line says \"Camera on\", and Stop camera turns the light off.")
                .limits([
                    "A denial is usually permanent for the site: the browser does not ask again, and libero cannot open its settings.",
                    "The Linux desktop WebView (WebKitGTK) denies every request, because wry answers no permission request there.",
                    "macOS's WebView grants every page itself; the system's camera prompt is the only consent gate, so ask in your own UI first.",
                    "Web needs a secure context (HTTPS or localhost). Android needs `[permissions] camera` and `microphone` in Dioxus.toml, plus `android.permission.MODIFY_AUDIO_SETTINGS` under `[android.permissions]`, or every microphone request is denied. Windows' WebView2 counts dioxus's page as insecure and has no capture API until the app marks it secure.",
                    "Blitz and a server render have no capture API: `is_supported()` stays false and a start fails with `Unsupported`. A recording is dropped past `max_bytes` (50 MB) with `TooLarge`. Screen capture and speaker choice are not covered.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_user_media(options) -> UserMedia" }
                    " opens the camera and microphone. "
                    Code { source: "start()" }
                    " asks and opens them, "
                    Code { source: "stop()" }
                    " closes them. Spread "
                    Code { source: "attributes()" }
                    " on your own "
                    Code { source: "video" }
                    " to show the stream. "
                    Code { source: "snapshot()" }
                    " takes a PNG into "
                    Code { source: "photo()" }
                    "; "
                    Code { source: "record()" }
                    " and "
                    Code { source: "finish()" }
                    " fill "
                    Code { source: "recorded()" }
                    ". "
                    Code { source: "use_user_media_devices()" }
                    " lists "
                    Code { source: "cameras()" }
                    " and "
                    Code { source: "microphones()" }
                    ", and "
                    Code { source: "switch_camera(id)" }
                    " reopens the stream on another."
                }
            },

            Demo {
                component: "CaptureBooth",
                children_text: "",
                file: DemoFile(include_str!("use_user_media/demo.rs")),
                // Printed at any value: `camera` is required.
                controls: vec![
                    Control::switch("camera")
                        .default("true")
                        .code(|_, values| vec![format!("camera: {}", values.str("camera"))]),
                    // Camera off records audio, so the microphone is on whatever the switch says.
                    Control::switch("microphone")
                        .code(|_, values| vec![format!("microphone: {}", values.str("microphone"))])
                        .hidden_when(|values| values.str("camera") != "true"),
                ],
                render: move |values: DemoValues| {
                    let camera = values.str("camera") == "true";
                    let microphone = values.str("microphone") == "true";
                    rsx! {
                        Remount { on: "{camera}{microphone}",
                            CaptureBooth { camera, microphone }
                        }
                    }
                },
            }
        }
    }
}
