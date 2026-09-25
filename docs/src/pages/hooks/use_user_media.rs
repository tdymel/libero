use crate::components::{Demo, DemoValues, DocPage, Wrap, a11y};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Flex, Switch, Text},
    hooks::{UserMedia, UserMediaError, UserMediaOptions, use_user_media, use_user_media_devices},
};

/// The hook in one component, as `CameraBooth` renders it.
fn code(_: &DemoValues, _: &str) -> String {
    r#"let mut with_audio = use_signal(|| false);
let mut media = use_user_media(UserMediaOptions { microphone: with_audio(), ..Default::default() });
let devices = use_user_media_devices();
// Announced once per change, never per frame.
let status = match (media.error(), media.is_recording(), media.is_live()) {
    (Some(UserMediaError::Denied), _, _) => "Camera refused",
    (Some(_), _, _) => "Camera unavailable",
    (None, true, _) => "Recording",
    (None, false, true) => "Camera on",
    (None, false, false) if media.is_pending() => "Waiting for permission",
    (None, false, false) => "Camera off",
};

rsx! {
    Flex { direction: "column", gap: "sm",
        video {
            aria_label: "Camera preview",
            autoplay: true, muted: true, playsinline: true,
            width: "320", height: "240",
            ..media.attributes(),
        }
        Switch { label: "With microphone", checked: with_audio(), onchange: move |on| with_audio.set(on) }
        Flex { direction: "row", gap: "sm",
            Button {
                onclick: move |_| if media.is_live() { media.stop() } else { media.start() },
                if media.is_live() { "Turn camera off" } else { "Turn camera on" }
            }
            Button { variant: "outlined", disabled: !media.is_live(), onclick: move |_| media.snapshot(), "Take photo" }
            Button {
                variant: "outlined",
                disabled: !media.is_live(),
                onclick: move |_| if media.is_recording() { media.finish() } else { media.record() },
                if media.is_recording() { "Stop recording" } else { "Record" }
            }
        }
        div { role: "status", "{status}" }
        if let Some(photo) = media.photo() {
            Text { "Photo: {photo.name()}, {photo.size() / 1024} KB" }
        }
        if let Some(clip) = media.recording() {
            Text { "Recording: {clip.name()}, {clip.size() / 1024} KB" }
        }
        Text { size: "sm", "Cameras: {devices.cameras().len()}" }
    }
}"#
    .to_string()
}

fn status(media: &UserMedia) -> &'static str {
    match (media.error(), media.is_recording(), media.is_live()) {
        (Some(UserMediaError::Denied), _, _) => "Camera refused",
        (Some(_), _, _) => "Camera unavailable",
        (None, true, _) => "Recording",
        (None, false, true) => "Camera on",
        (None, false, false) if media.is_pending() => "Waiting for permission",
        (None, false, false) => "Camera off",
    }
}

#[component]
fn CameraBooth() -> Element {
    let mut with_audio = use_signal(|| false);
    let mut media = use_user_media(UserMediaOptions {
        microphone: with_audio(),
        ..Default::default()
    });
    let devices = use_user_media_devices();
    let status = status(&media);
    let permission = format!("{:?}", media.camera_permission());

    rsx! {
        Flex { direction: "column", gap: "sm",
            video {
                aria_label: "Camera preview",
                autoplay: true,
                muted: true,
                playsinline: true,
                width: "320",
                height: "240",
                style: "background: #000; max-width: 100%;",
                ..media.attributes(),
            }
            Switch {
                label: "With microphone",
                checked: with_audio(),
                onchange: move |on| with_audio.set(on),
            }
            Flex { direction: "row", gap: "sm",
                Button {
                    onclick: move |_| if media.is_live() { media.stop() } else { media.start() },
                    if media.is_live() { "Turn camera off" } else { "Turn camera on" }
                }
                Button {
                    variant: "outlined",
                    disabled: !media.is_live(),
                    onclick: move |_| media.snapshot(),
                    "Take photo"
                }
                Button {
                    variant: "outlined",
                    disabled: !media.is_live(),
                    onclick: move |_| if media.is_recording() { media.finish() } else { media.record() },
                    if media.is_recording() { "Stop recording" } else { "Record" }
                }
            }
            div { role: "status", "{status}" }
            if let Some(photo) = media.photo() {
                Text { "Photo: {photo.name()}, {photo.size() / 1024} KB" }
            }
            if let Some(clip) = media.recording() {
                Text { "Recording: {clip.name()}, {clip.size() / 1024} KB" }
            }
            Text { size: "sm", "Cameras: {devices.cameras().len()}, camera permission: {permission}" }
        }
    }
}

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
                    Code { source: "recording()" }
                    ", both a "
                    Code { source: "FileData" }
                    " like a picked file. Options apply on the next "
                    Code { source: "start()" }
                    "."
                }
                Text {
                    Code { source: "use_user_media_devices()" }
                    " lists "
                    Code { source: "cameras()" }
                    " and "
                    Code { source: "microphones()" }
                    " for a picker feeding "
                    Code { source: "camera_id" }
                    "; labels stay empty until a grant, so "
                    Code { source: "refresh()" }
                    " after one. A recording crosses a WebView's IPC in 1 s chunks and is dropped past "
                    Code { source: "max_bytes" }
                    " (50 MB by default) with "
                    Code { source: "TooLarge" }
                    "."
                }
                Text {
                    "Web: a secure context (HTTPS or localhost). Android: declare "
                    Code { source: "[permissions] camera" }
                    " and "
                    Code { source: "microphone" }
                    " in Dioxus.toml; the system asks on the first start. Windows is untested. Blitz and a server render have no capture API: "
                    Code { source: "is_supported()" }
                    " stays false and a start fails with "
                    Code { source: "Unsupported" }
                    ". Screen capture and speaker choice are not covered."
                }
            },

            Demo {
                component: "use_user_media",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { CameraBooth {} },
                wrap: Wrap(code),
            }
        }
    }
}
