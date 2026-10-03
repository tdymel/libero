use crate::components::{Control, Demo, DemoFile, DemoValues, DocPage, a11y};
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
                .limits([
                    "A denial is usually permanent for the site: the browser does not ask again, and libero cannot open its settings.",
                    "The Linux desktop WebView (WebKitGTK) denies every request, because wry answers no permission request there.",
                    "macOS's WebView grants every page itself; the system's camera prompt is the only consent gate, so ask in your own UI first.",
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
                    ", both a "
                    Code { source: "FileData" }
                    " like a picked file. Options apply on the next "
                    Code { source: "start()" }
                    ". "
                    Code { source: "camera_permission()" }
                    " and "
                    Code { source: "microphone_permission()" }
                    " read the permission state, kept current where the platform reports changes."
                }
                Text {
                    Code { source: "use_user_media_devices()" }
                    " lists "
                    Code { source: "cameras()" }
                    " and "
                    Code { source: "microphones()" }
                    ", kept current as devices come and go, and "
                    Code { source: "refresh()" }
                    " lists them again; labels and ids fill in once a start is granted. "
                    Code { source: "switch_camera(id)" }
                    " reopens a live stream on another camera, "
                    Code { source: "camera_id()" }
                    " names the live one, and the "
                    Code { source: "facing" }
                    " option picks a phone's front or back. "
                    Code { source: "libero::utils::data_url(&file)" }
                    " turns a photo or clip into a "
                    Code { source: "src" }
                    ". Switch the camera off for an audio-only recording. A recording crosses a WebView's IPC in 1 s chunks and is dropped past "
                    Code { source: "max_bytes" }
                    " (50 MB by default) with "
                    Code { source: "TooLarge" }
                    "; in a browser it stays in the page until you read it."
                }
                Text {
                    "Web: a secure context (HTTPS or localhost). Android: declare "
                    Code { source: "[permissions] camera" }
                    " and "
                    Code { source: "microphone" }
                    " in Dioxus.toml, plus "
                    Code { source: "\"android.permission.MODIFY_AUDIO_SETTINGS\"" }
                    " under "
                    Code { source: "[android.permissions]" }
                    ": without it every microphone request is denied. The system asks on the first start. Windows is untested. Blitz and a server render have no capture API: "
                    Code { source: "is_supported()" }
                    " stays false and a start fails with "
                    Code { source: "Unsupported" }
                    ". Screen capture and speaker choice are not covered."
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
                    rsx! { CaptureBooth { key: "{camera}{microphone}", camera, microphone } }
                },
            }
        }
    }
}
