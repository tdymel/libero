# User media

Crate: `libero`
Import: `use libero::hooks::{CameraFacing, MediaDevice, PermissionState, UserMedia, UserMediaDevices, UserMediaError, UserMediaOptions, use_user_media, use_user_media_devices};` and `use libero::utils::data_url;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/user_media.rs>
Index: [index.md](index.md) lists every other page
Description: The camera and microphone with a preview, a PNG snapshot, a chunked video or audio recording and a camera switch; never prompts on mount.

`use_user_media(options) -> UserMedia` opens the camera and microphone.
`start()` asks and opens them, `stop()` closes them. Spread `attributes()` on
your own `video` to show the stream. `snapshot()` takes a PNG into `photo()`;
`record()` and `finish()` fill `recording()`, both a `FileData` like a picked
file. Options apply on the next `start()`.

`use_user_media_devices()` lists `cameras()` and `microphones()`; labels and
ids fill in once a start is granted. `switch_camera(id)` reopens a live stream
on another camera, `camera_id()` names the live one, and the `facing` option
picks a phone's front or back. `libero::utils::data_url(&file)` turns a photo
or clip into a `src`. Switch the camera off for an audio-only recording. A
recording crosses a WebView's IPC in 1 s chunks and is dropped past
`max_bytes` (50 MB by default) with `TooLarge`.

Web: a secure context (HTTPS or localhost). Android: declare
`[permissions] camera` and `microphone` in Dioxus.toml, plus
`"android.permission.MODIFY_AUDIO_SETTINGS"` under `[android.permissions]`:
without it every microphone request is denied. The system asks on the first
start. Windows is untested. Blitz and a server render have no capture
API: `is_supported()` stays false and a start fails with `Unsupported`. Screen
capture and speaker choice are not covered.

## Usage

```rust
use dioxus::html::FileData;
use dioxus::prelude::*;
use libero::{
    components::{ActionIcon, Audio, Button, Flex, Image, Pictogram, Video},
    hooks::{UserMediaError, UserMediaOptions, use_user_media, use_user_media_devices},
    utils::data_url,
};
use pictogram_icons_lucide as lucide;

#[component]
fn CaptureBooth() -> Element {
    let mut media = use_user_media(UserMediaOptions { camera: true, microphone: false, ..Default::default() });
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
    // The camera after the live one, on a phone front and back.
    let mut switch = move || {
        let cameras = devices.cameras();
        let at = cameras.iter().position(|c| Some(&c.id) == media.camera_id().as_ref());
        if let Some(next) = cameras.get(at.map_or(0, |at| (at + 1) % cameras.len())) {
            media.switch_camera(next.id.clone());
        }
    };

    rsx! {
        Flex { direction: "column", gap: "sm",
            video {
                aria_label: "Camera preview",
                autoplay: true, muted: true, playsinline: true,
                width: "320", height: "240",
                ..media.attributes(),
            }
            Flex { direction: "row", gap: "sm", align: "center",
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
                // Only where there is another camera to switch to.
                if media.is_live() && devices.cameras().len() > 1 {
                    ActionIcon {
                        aria_label: "Switch camera",
                        tooltip: true,
                        onclick: move |_| switch(),
                        Pictogram { icon: lucide::switch_camera::outlined }
                    }
                }
            }
            div { role: "status", "{status}" }
            if let Some(photo) = media.photo() {
                Preview { file: photo }
            }
            if let Some(clip) = media.recording() {
                Preview { file: clip }
            }
        }
    }
}

/// `data_url` holds the whole file: fine for a photo or a short clip.
#[component]
fn Preview(file: FileData) -> Element {
    let src = use_resource(use_reactive!(|file| async move { data_url(&file).await }));
    let kind = file.content_type().unwrap_or_default();
    let Some(Some(src)) = src() else { return rsx! {} };
    rsx! {
        if kind.starts_with("image/") {
            Image { src, alt: "Your photo", width: "160" }
        } else if kind.starts_with("audio/") {
            Audio { src, label: "Your recording" }
        } else {
            Video { src, label: "Your recording" }
        }
    }
}
```

## API

```rust,ignore
pub fn use_user_media(options: UserMediaOptions) -> UserMedia
pub fn use_user_media_devices() -> UserMediaDevices

pub struct UserMediaOptions {
    pub camera: bool,
    pub microphone: bool,
    pub camera_id: Option<String>,
    pub microphone_id: Option<String>,
    pub facing: Option<CameraFacing>,
    pub max_bytes: Option<u64>,
}

#[non_exhaustive]
pub enum CameraFacing { User, Environment }

pub async fn libero::utils::data_url(file: &FileData) -> Option<String>

pub struct MediaDevice {
    pub id: String,
    pub label: String,
}

pub enum UserMediaError { Unsupported, Denied, NotFound, InUse, Constraint, TooLarge, Failed }
pub enum PermissionState { Granted, Denied, Prompt, Unknown, Unsupported }
```

| Option | Default | Description |
|---|---|---|
| `camera` | `true` | Opens a camera on `start()`. |
| `microphone` | `false` | Opens a microphone on `start()`. |
| `camera_id`, `microphone_id` | `None` | A `MediaDevice::id` from `use_user_media_devices`; `None` lets the platform pick. |
| `facing` | `None` | `User` (selfie) or `Environment` (back) on a phone; ignored while `camera_id` or a `switch_camera` names a camera. A hint: a single camera opens anyway. |
| `max_bytes` | `Some(50 MB)` | A longer recording is dropped with `TooLarge`; `None` sets no cap. |

| Method of `UserMedia` | Description |
|---|---|
| `start()` | Opens what the options ask for, prompting if the user has not answered; replaces an open stream. Does nothing while one is pending. |
| `stop()` | Stops every track, so the light goes off; a running recording finishes first. Unmount does the same. |
| `attributes()` | Spread on the preview `video`; the stream shows in the element carrying them. |
| `snapshot()` / `photo()` | A PNG of the preview's current frame. Needs a live camera shown in the `video`. |
| `record()` / `finish()` / `recording()` | Records the stream into WebM, or MP4 where only that records; audio only without a camera. |
| `switch_camera(id)` | Live, stops the camera and reopens on `id` (a phone opens one at a time); a running recording finishes into `recording()` first. Not live, the next `start()` opens it. Kept until `camera_id` changes. |
| `camera_id()` | The live camera's `MediaDevice::id`; `None` when off or audio only. |
| `is_live()`, `is_pending()`, `is_recording()` | A stream is open; a start awaits its answer; a recording runs. |
| `error()` | Why the last step failed; the next success clears it. `Denied` also covers an insecure context. |
| `camera_permission()`, `microphone_permission()` | `Prompt`, `Granted` or `Denied` where the Permissions API answers and follows its changes; `Unknown` where it does not, until a grant or denial; `Unsupported` without a capture API. |
| `is_supported()` | `false` until mounted, then whether a capture API exists. |

| Method of `UserMediaDevices` | Description |
|---|---|
| `cameras()`, `microphones()` | The devices, kept current as they come and go, and listed again when a start is granted (the labels and ids fill in then). |
| `refresh()` | Lists them again. |
| `is_supported()` | `false` until mounted, then whether the page lists devices. |

`UserMedia` and `UserMediaDevices` are `Copy`.

| Platform | Support |
|---|---|
| Web | Full, in a secure context. |
| Android (WebView) | Full with `[permissions] camera` and `microphone` in Dioxus.toml, plus `MODIFY_AUDIO_SETTINGS` under `[android.permissions]` for the microphone. |
| Linux desktop (WebKitGTK) | Every start is `Denied`. |
| macOS desktop | Untested; the WebView grants every page, the system's prompt is the only gate. |
| Windows desktop | Untested. |
| Blitz, server render | `Unsupported`. |

## Accessibility

### Libero handles

- Mounting never prompts: the browser or OS asks only on start, which you call
  from a user's action.
- Stop and unmount stop every track, so the device's camera light and recording
  indicator go off with them.
- It announces nothing, and never hides the platform's own recording indicator.

### You must

- Start from a visible control whose label says what the camera or microphone
  is for, never on page load.
- Show a visible "camera on" and "recording" state, with a control that stops
  each.
- Announce start, stop and refusal once in a status region, never per frame or
  recording second.
- Keep the preview `<video>` muted and give it an `aria_label`; a live
  microphone played back echoes.
- Give a refused user another way on, such as a file upload, and say how to
  re-enable the camera in the browser or system settings.

### Limits

- A denial is usually permanent for the site: the browser does not ask again,
  and libero cannot open its settings.
- The Linux desktop WebView (WebKitGTK) denies every request, because wry
  answers no permission request there.
- macOS's WebView grants every page itself; the system's camera prompt is the
  only consent gate, so ask in your own UI first.
