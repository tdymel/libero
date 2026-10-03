use dioxus::html::FileData;
use dioxus::prelude::*;
use libero::components::{Audio, Button, Flex, Image, NativeSelect, Video};
use libero::hooks::{
    MediaDevice, UserMedia, UserMediaError, UserMediaOptions, use_user_media,
    use_user_media_devices,
};
use libero::sx::sx;
use libero::utils::data_url;

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

// demo-code: start
/// What the status region says, once per change, never per frame.
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

/// Camera off records audio only, so the microphone is on whatever `microphone` says.
#[component]
pub fn CaptureBooth(camera: bool, #[props(default)] microphone: bool) -> Element {
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
        // A recording's `Video` is 40rem wide unsized: the cap keeps it, and the photo, in a phone's width.
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
// demo-code: end
