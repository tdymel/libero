//! `use_user_media`: its state in text, a preview, a snapshot and a recording.

use dioxus::prelude::*;
use libero::hooks::{UserMediaOptions, use_user_media, use_user_media_devices};

use crate::Routes;

pub const ROUTES: Routes = &[("/use-user-media", || rsx! { Captured {} })];

#[component]
fn Captured() -> Element {
    let mut audio = use_signal(|| false);
    let mut media = use_user_media(UserMediaOptions {
        microphone: audio(),
        ..Default::default()
    });
    let mut devices = use_user_media_devices();
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
            ..media.attributes()
        }
        button { id: "start", onclick: move |_| media.start(), "Start" }
        button { id: "stop", onclick: move |_| media.stop(), "Stop" }
        button { id: "audio", onclick: move |_| audio.toggle(), "Audio" }
        button { id: "snapshot", onclick: move |_| media.snapshot(), "Snapshot" }
        button { id: "record", onclick: move |_| media.record(), "Record" }
        button { id: "finish", onclick: move |_| media.finish(), "Finish" }
        button { id: "refresh", onclick: move |_| devices.refresh(), "Refresh" }
        p { id: "supported", "{media.is_supported()}" }
        p { id: "camera", "{media.camera_permission():?}" }
        p { id: "live", "{media.is_live()}" }
        p { id: "pending", "{media.is_pending()}" }
        p { id: "recording", "{media.is_recording()}" }
        p { id: "error", "{media.error():?}" }
        p { id: "photo", "{size(media.photo())}" }
        p { id: "recorded", "{size(media.recording())}" }
        p { id: "cameras", "{devices.cameras().len()}" }
        p { id: "devices-supported", "{devices.is_supported()}" }
    }
}
