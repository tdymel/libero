//! `use_user_media`: its state in text, a preview, a snapshot and a recording.

use dioxus::prelude::*;
use libero::hooks::{
    UserMediaOptions, use_element, use_fullscreen, use_user_media, use_user_media_devices,
};

use crate::Routes;

pub const ROUTES: Routes = &[("/use-user-media", || rsx! { Captured {} })];

/// Counts the recorder's non-empty chunks into `body[data-chunks]`, from 0 at each record:
/// a test waits for one instead of sleeping out a chunk.
const COUNT_CHUNKS: &str = "document.body.dataset.chunks = 0;
    if (!window.__countsChunks) {
        window.__countsChunks = true;
        const add = MediaRecorder.prototype.addEventListener;
        MediaRecorder.prototype.addEventListener = function (type, listener, ...rest) {
            if (type !== 'dataavailable') return add.call(this, type, listener, ...rest);
            return add.call(this, type, (event) => {
                if (event.data.size) document.body.dataset.chunks = Number(document.body.dataset.chunks) + 1;
                return listener(event);
            }, ...rest);
        };
    }";

#[component]
fn Captured() -> Element {
    let mut audio = use_signal(|| false);
    let mut camera = use_signal(|| true);
    let mut media = use_user_media(UserMediaOptions {
        camera: camera(),
        microphone: audio(),
        ..Default::default()
    });
    let mut devices = use_user_media_devices();
    // A second hook's attributes on the same preview (1237).
    let preview = use_element();
    let fullscreen = use_fullscreen(preview);
    let size = |file: Option<dioxus::html::FileData>| {
        file.map_or("none".to_string(), |file| {
            format!("{} {}", file.name(), file.size() > 0)
        })
    };

    rsx! {
        // A fixed box: a stream's arrival must not move the buttons under a click.
        video {
            id: "preview",
            autoplay: true,
            muted: true,
            playsinline: true,
            width: "160",
            height: "120",
            onmounted: preview.mount(),
            ..media.attributes(),
            ..fullscreen.attributes()
        }
        button { id: "fullscreen", onclick: move |_| fullscreen.toggle(), "Fullscreen" }
        button { id: "start", onclick: move |_| media.start(), "Start" }
        button { id: "stop", onclick: move |_| media.stop(), "Stop" }
        button { id: "audio", onclick: move |_| audio.toggle(), "Audio" }
        button { id: "camera-toggle", onclick: move |_| camera.toggle(), "Camera" }
        button { id: "snapshot", onclick: move |_| media.snapshot(), "Snapshot" }
        button {
            id: "record",
            onclick: move |_| async move {
                let _ = document::eval(COUNT_CHUNKS).await;
                media.record();
            },
            "Record"
        }
        button { id: "finish", onclick: move |_| media.finish(), "Finish" }
        button { id: "refresh", onclick: move |_| devices.refresh(), "Refresh" }
        // The camera after the live one, as a phone's switch button picks it (1347).
        button {
            id: "switch",
            onclick: move |_| {
                let cameras = devices.cameras();
                let at = cameras.iter().position(|camera| Some(&camera.id) == media.camera_id().as_ref());
                if let Some(next) = cameras.get(at.map_or(0, |at| (at + 1) % cameras.len())) {
                    media.switch_camera(next.id.clone());
                }
            },
            "Switch"
        }
        p { id: "supported", "{media.is_supported()}" }
        p { id: "camera", "{media.camera_permission():?}" }
        p { id: "live", "{media.is_live()}" }
        p { id: "pending", "{media.is_pending()}" }
        p { id: "recording", "{media.is_recording()}" }
        p { id: "error", "{media.error():?}" }
        p { id: "photo", "{size(media.photo())}" }
        p { id: "recorded", "{size(media.recording())}" }
        p { id: "recorded-type", {media.recording().and_then(|file| file.content_type()).unwrap_or_default()} }
        p { id: "cameras", "{devices.cameras().len()}" }
        p { id: "labelled", "{devices.cameras().iter().filter(|camera| !camera.label.is_empty()).count()}" }
        p { id: "devices-supported", "{devices.is_supported()}" }
        p { id: "camera-id", {media.camera_id().unwrap_or_default()} }
        p { id: "is-fullscreen", "{fullscreen.is_fullscreen()}" }
    }
}
