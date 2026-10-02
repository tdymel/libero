use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y};
use dioxus::html::FileData;
use dioxus::prelude::*;
use libero::{
    components::{Audio, Button, Code, Flex, Image, NativeSelect, Text, Video},
    hooks::{
        MediaDevice, UserMedia, UserMediaError, UserMediaOptions, use_user_media,
        use_user_media_devices,
    },
    sx::sx,
    utils::data_url,
};

/// The hook in one component, as `CaptureBooth` renders it.
fn code(values: &DemoValues, _: &str) -> String {
    let camera = values.str("camera") == "true";
    let microphone = values.str("microphone") == "true" || !camera;
    format!(
        r#"let mut media = use_user_media(UserMediaOptions {{
    camera: {camera},
    microphone: {microphone},
    ..Default::default()
}});
let devices = use_user_media_devices();
let device = if {camera} {{ "Camera" }} else {{ "Microphone" }};
let noun = device.to_lowercase();
// Announced once per change, never per frame.
let status = match (media.error(), media.is_recording(), media.is_live()) {{
    (Some(UserMediaError::Denied), _, _) => format!("{{device}} refused"),
    (Some(_), _, _) => format!("{{device}} unavailable"),
    (None, true, _) => "Recording".to_string(),
    (None, false, true) => format!("{{device}} on"),
    (None, false, false) if media.is_pending() => "Waiting for permission".to_string(),
    (None, false, false) => format!("{{device}} off"),
}};
// Ids and labels fill in once a start is granted.
let cameras: Vec<MediaDevice> = devices.cameras().into_iter().filter(|c| !c.id.is_empty()).collect();
let ids: Vec<String> = cameras.iter().map(|c| c.id.clone()).collect();

rsx! {{
    // A recording's `Video` is 40rem wide unsized: the cap keeps it, and the photo, in a phone's width.
    Flex {{ direction: "column", gap: "sm", sx: sx().max_width("100%"),
        if {camera} {{
            video {{
                aria_label: "Camera preview",
                autoplay: true, muted: true, playsinline: true,
                width: "320", height: "240",
                style: "background: #000; max-width: 100%;",
                ..media.attributes(),
            }}
        }}
        Flex {{ direction: "row", gap: "sm", align: "center", wrap: "wrap",
            Button {{
                onclick: move |_| if media.is_live() {{ media.stop() }} else {{ media.start() }},
                if media.is_live() {{ "Turn {{noun}} off" }} else {{ "Turn {{noun}} on" }}
            }}
            if {camera} {{
                Button {{ variant: "outlined", disabled: !media.is_live(), onclick: move |_| media.snapshot(), "Take photo" }}
            }}
            Button {{
                variant: "outlined",
                disabled: !media.is_live(),
                onclick: move |_| if media.is_recording() {{ media.finish() }} else {{ media.record() }},
                if media.is_recording() {{ "Stop recording" }} else {{ "Record" }}
            }}
        }}
        // Only where there is another camera to switch to.
        if {camera} && media.is_live() && cameras.len() > 1 {{
            NativeSelect {{
                label: "Camera",
                placeholder: "Default camera",
                options: ids,
                option_label: move |id: String| camera_name(&cameras, &id),
                value: media.camera_id(),
                onchange: move |id: String| media.switch_camera(id),
            }}
        }}
        div {{ role: "status", "{{status}}" }}
        // `data_url` reads the file once; fine for a photo or a short clip.
        if let Some(photo) = media.photo() {{
            Preview {{ file: photo }}
        }}
        if let Some(clip) = media.recorded() {{
            Preview {{ file: clip }}
        }}
    }}
}}

/// The camera's label, or "Camera 2" where the platform gives none.
fn camera_name(cameras: &[MediaDevice], id: &str) -> String {{
    let at = cameras.iter().position(|c| c.id == id).unwrap_or_default();
    match cameras.get(at) {{
        Some(camera) if !camera.label.is_empty() => camera.label.clone(),
        _ => format!("Camera {{}}", at + 1),
    }}
}}

#[component]
fn Preview(file: dioxus::html::FileData) -> Element {{
    let src = use_resource(use_reactive!(|file| async move {{ libero::utils::data_url(&file).await }}));
    let kind = file.content_type().unwrap_or_default();
    let Some(Some(src)) = src() else {{ return rsx! {{}} }};
    rsx! {{
        if kind.starts_with("image/") {{
            Image {{ src, alt: "Your photo", sx: sx().width("160px").height("auto") }}
        }} else if kind.starts_with("audio/") {{
            Audio {{ src, label: "Your recording" }}
        }} else {{
            Video {{ src, label: "Your recording" }}
        }}
    }}
}}"#
    )
}

/// What the status region says, `device` being "Camera" or "Microphone".
fn status(media: &UserMedia, device: &str) -> String {
    match (media.error(), media.is_recording(), media.is_live()) {
        (Some(UserMediaError::Denied), _, _) => format!("{device} refused"),
        (Some(_), _, _) => format!("{device} unavailable"),
        (None, true, _) => "Recording".to_string(),
        (None, false, true) => format!("{device} on"),
        (None, false, false) if media.is_pending() => "Waiting for permission".to_string(),
        (None, false, false) => format!("{device} off"),
    }
}

/// The camera's label, or "Camera 2" where the platform gives none.
fn camera_name(cameras: &[MediaDevice], id: &str) -> String {
    let at = cameras.iter().position(|c| c.id == id).unwrap_or_default();
    match cameras.get(at) {
        Some(camera) if !camera.label.is_empty() => camera.label.clone(),
        _ => format!("Camera {}", at + 1),
    }
}

/// A photo as an image, a recording as a player, from the file's own bytes.
#[component]
fn Preview(file: FileData) -> Element {
    let src = use_resource(use_reactive!(|file| async move { data_url(&file).await }));
    let kind = file.content_type().unwrap_or_default();
    let Some(Some(src)) = src() else {
        return rsx! {};
    };
    rsx! {
        if kind.starts_with("image/") {
            // `Image` fills its box: a `width` attribute loses to that.
            Image { src, alt: "Your photo", sx: sx().width("160px").height("auto") }
        } else if kind.starts_with("audio/") {
            Audio { src, label: "Your recording" }
        } else {
            Video { src, label: "Your recording" }
        }
    }
}

#[component]
fn CaptureBooth(camera: bool, microphone: bool) -> Element {
    let mut media = use_user_media(UserMediaOptions {
        camera,
        microphone: microphone || !camera,
        ..Default::default()
    });
    let devices = use_user_media_devices();
    let device = if camera { "Camera" } else { "Microphone" };
    let status = status(&media, device);
    let noun = device.to_lowercase();
    // Ids and labels fill in once a start is granted.
    let cameras: Vec<MediaDevice> = devices
        .cameras()
        .into_iter()
        .filter(|c| !c.id.is_empty())
        .collect();
    let ids: Vec<String> = cameras.iter().map(|c| c.id.clone()).collect();

    rsx! {
        Flex { direction: "column", gap: "sm", sx: sx().max_width("100%"),
            if camera {
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
            }
            Flex { direction: "row", gap: "sm", align: "center", wrap: "wrap",
                Button {
                    onclick: move |_| if media.is_live() { media.stop() } else { media.start() },
                    if media.is_live() { "Turn {noun} off" } else { "Turn {noun} on" }
                }
                if camera {
                    Button {
                        variant: "outlined",
                        disabled: !media.is_live(),
                        onclick: move |_| media.snapshot(),
                        "Take photo"
                    }
                }
                Button {
                    variant: "outlined",
                    disabled: !media.is_live(),
                    onclick: move |_| if media.is_recording() { media.finish() } else { media.record() },
                    if media.is_recording() { "Stop recording" } else { "Record" }
                }
            }
            if camera && media.is_live() && cameras.len() > 1 {
                NativeSelect {
                    label: "Camera",
                    placeholder: "Default camera",
                    options: ids,
                    option_label: move |id: String| camera_name(&cameras, &id),
                    value: media.camera_id(),
                    onchange: move |id: String| media.switch_camera(id),
                }
            }
            div { role: "status", "{status}" }
            if let Some(photo) = media.photo() {
                Preview { file: photo }
            }
            if let Some(clip) = media.recorded() {
                Preview { file: clip }
            }
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
                component: "use_user_media",
                children_text: "",
                // Options, not props: `code` prints them into the hook call.
                controls: vec![
                    Control::switch("camera").default("true").code(|_, _| vec![]),
                    // Camera off records audio, so the microphone is on whatever the switch says.
                    Control::switch("microphone")
                        .code(|_, _| vec![])
                        .hidden_when(|values| values.str("camera") != "true"),
                ],
                render: move |values: DemoValues| {
                    let camera = values.str("camera") == "true";
                    let microphone = values.str("microphone") == "true";
                    rsx! { CaptureBooth { key: "{camera}{microphone}", camera, microphone } }
                },
                wrap: Wrap(code),
            }
        }
    }
}
