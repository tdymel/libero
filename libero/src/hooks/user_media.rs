use std::{cell::RefCell, rc::Rc};

use dioxus::core::{Attribute, AttributeValue};
use dioxus::html::FileData;
use dioxus::prelude::*;

use crate::platform::{
    CAPTURE_ATTR, CaptureEvent, CaptureSession, CaptureSubscription, DeviceList, MediaDevice,
    PermissionKind, PermissionState, PermissionSubscription, UserMediaError, capture, constraints,
    file_from_blob, file_from_bytes, next_observe_tag, permission,
};

/// What [`use_user_media`] opens on [`start`](UserMedia::start).
#[derive(Debug, Clone, PartialEq)]
pub struct UserMediaOptions {
    pub camera: bool,
    pub microphone: bool,
    /// A [`MediaDevice::id`] from [`use_user_media_devices`]; `None` lets the platform pick.
    pub camera_id: Option<String>,
    /// The side a phone's camera faces; ignored while `camera_id` (or a
    /// [`switch_camera`](UserMedia::switch_camera)) names one. A hint: a laptop's
    /// single camera opens anyway.
    pub facing: Option<CameraFacing>,
    pub microphone_id: Option<String>,
    /// A longer recording is dropped with [`UserMediaError::TooLarge`]; `None` sets no cap.
    pub max_bytes: Option<u64>,
}

impl Default for UserMediaOptions {
    /// The camera alone, recordings up to 50 MB.
    fn default() -> Self {
        Self {
            camera: true,
            microphone: false,
            camera_id: None,
            microphone_id: None,
            facing: None,
            max_bytes: Some(50 * 1024 * 1024),
        }
    }
}

/// Which way a camera faces, on a phone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CameraFacing {
    /// Towards the user: the selfie camera.
    User,
    /// Away from the user: the back camera.
    Environment,
}

impl CameraFacing {
    /// The `facingMode` name.
    fn name(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Environment => "environment",
        }
    }
}

/// The camera and microphone, opened only on [`start`](Self::start): mounting
/// never prompts. Spread [`attributes`](Self::attributes) on your preview `<video>`.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::use_user_media;
/// # fn app() -> Element {
/// let mut camera = use_user_media(Default::default());
///
/// rsx! {
///     video { autoplay: true, muted: true, playsinline: true, ..camera.attributes() }
///     button {
///         onclick: move |_| if camera.is_live() { camera.stop() } else { camera.start() },
///         if camera.is_live() { "Turn camera off" } else { "Turn camera on" }
///     }
///     button { disabled: !camera.is_live(), onclick: move |_| camera.snapshot(), "Take photo" }
/// }
/// # }
/// ```
#[derive(Clone, Copy)]
pub struct UserMedia {
    options: Signal<UserMediaOptions>,
    /// The [`switch_camera`](UserMedia::switch_camera) choice, over `options.camera_id`.
    chosen_camera: Signal<Option<String>>,
    camera: Signal<Option<String>>,
    tag: u64,
    supported: Signal<bool>,
    camera_permission: Signal<PermissionState>,
    microphone_permission: Signal<PermissionState>,
    live: Signal<bool>,
    pending: Signal<bool>,
    recording: Signal<bool>,
    error: Signal<Option<UserMediaError>>,
    photo: Signal<Option<FileData>>,
    recorded: Signal<Option<FileData>>,
    chunks: Signal<Vec<u8>>,
    session: Signal<Option<Box<dyn CaptureSession>>>,
}

impl UserMedia {
    /// Opens the camera and microphone the options ask for, prompting if the
    /// user has not answered yet. Call it from a user's action.
    pub fn start(&mut self) {
        if *self.pending.peek() {
            return;
        }
        let options = self.options.peek().clone();
        let camera_id = self.chosen_camera.peek().clone().or(options.camera_id);
        fn wanted(on: bool, id: &Option<String>) -> Option<Option<&str>> {
            on.then_some(id.as_deref())
        }
        let Some(constraints) = constraints(
            wanted(options.camera, &camera_id),
            wanted(options.microphone, &options.microphone_id),
            options.facing.map(CameraFacing::name),
        ) else {
            self.error.set(Some(UserMediaError::Constraint));
            return;
        };
        if !self.ensure_session() {
            self.settle(CaptureEvent::Failed(UserMediaError::Unsupported));
            return;
        }
        if let Some(session) = &*self.session.peek() {
            session.open(constraints);
        }
        self.pending.set(true);
        self.error.set(None);
    }

    /// Stops every track, so the device's light goes off. A running recording
    /// finishes first. Also on unmount.
    pub fn stop(&mut self) {
        if let Some(session) = &*self.session.peek() {
            session.close();
        }
        self.pending.set(false);
        self.live.set(false);
        self.camera.set(None);
    }

    /// Opens the camera `id` (a [`MediaDevice::id`]) in place of the live one,
    /// which stops first: a phone opens one camera at a time. A running
    /// recording finishes into [`recording`](Self::recording) first. Not live,
    /// the next [`start`](Self::start) opens it. Kept until `options.camera_id` changes.
    ///
    /// ```rust
    /// # use libero::hooks::{UserMedia, UserMediaDevices};
    /// // The camera after the live one, for a "Switch camera" button.
    /// fn next_camera(mut media: UserMedia, devices: UserMediaDevices) {
    ///     let cameras = devices.cameras();
    ///     let at = cameras.iter().position(|c| Some(&c.id) == media.camera_id().as_ref());
    ///     if let Some(next) = cameras.get(at.map_or(0, |at| (at + 1) % cameras.len())) {
    ///         media.switch_camera(next.id.clone());
    ///     }
    /// }
    /// ```
    pub fn switch_camera(&mut self, id: impl Into<String>) {
        self.chosen_camera.set(Some(id.into()));
        if *self.live.peek() || *self.pending.peek() {
            self.stop();
            self.start();
        }
    }

    /// Takes a PNG of the preview's current frame into [`photo`](Self::photo).
    /// Needs a live camera shown in the `<video>`.
    pub fn snapshot(&mut self) {
        match &*self.session.peek() {
            Some(session) if *self.live.peek() => session.snapshot(),
            _ => self.error.set(Some(UserMediaError::Failed)),
        }
    }

    /// Starts recording the stream; [`finish`](Self::finish) ends it into
    /// [`recording`](Self::recording). Does nothing unless live.
    pub fn record(&mut self) {
        if !*self.live.peek() || *self.recording.peek() {
            return;
        }
        if let Some(session) = &*self.session.peek() {
            self.chunks.write().clear();
            session.record(self.options.peek().max_bytes);
        }
    }

    pub fn finish(&mut self) {
        if let Some(session) = &*self.session.peek() {
            session.finish();
        }
    }

    /// Starts the session script on first use; `false` where nothing captures.
    fn ensure_session(&mut self) -> bool {
        if self.session.peek().is_some() {
            return true;
        }
        let Some(api) = capture() else {
            return false;
        };
        let this = *self;
        let session = api.session(
            self.tag,
            Box::new(move |event| {
                let mut this = this;
                this.settle(event);
            }),
        );
        self.session.set(Some(session));
        true
    }

    pub(crate) fn settle(&mut self, event: CaptureEvent) {
        match event {
            CaptureEvent::Live(camera) => {
                self.pending.set(false);
                self.live.set(true);
                self.camera.set(camera);
                self.error.set(None);
                self.set_permissions(PermissionState::Granted);
            }
            CaptureEvent::Ended => {
                self.live.set(false);
                self.camera.set(None);
            }
            CaptureEvent::Failed(error) => {
                self.pending.set(false);
                self.error.set(Some(error));
                match error {
                    UserMediaError::Denied => self.set_permissions(PermissionState::Denied),
                    UserMediaError::Unsupported => {
                        self.supported.set(false);
                        self.camera_permission.set(PermissionState::Unsupported);
                        self.microphone_permission.set(PermissionState::Unsupported);
                    }
                    _ => {}
                }
            }
            CaptureEvent::Photo(Some((kind, bytes))) => {
                let file = file_from_bytes(&file_name("photo", &kind), &kind, bytes);
                self.error
                    .set(file.is_none().then_some(UserMediaError::Failed));
                self.photo.set(file);
            }
            CaptureEvent::Photo(None) => self.error.set(Some(UserMediaError::Failed)),
            CaptureEvent::Recording => {
                self.recording.set(true);
                self.recorded.set(None);
            }
            CaptureEvent::Chunk(bytes) => self.chunks.write().extend(bytes),
            CaptureEvent::Recorded(kind, blob) => {
                self.recording.set(false);
                let mut bytes = std::mem::take(&mut *self.chunks.write());
                let kind = kind.split(';').next().unwrap_or_default().to_string();
                let name = file_name("recording", &kind);
                self.recorded.set(match blob {
                    Some(blob) => file_from_blob(&name, &kind, blob),
                    None => {
                        // The growth's slack, up to the recording's size again, would stay with the file.
                        bytes.shrink_to_fit();
                        file_from_bytes(&name, &kind, bytes)
                    }
                });
            }
            CaptureEvent::RecordFailed(error) => {
                self.recording.set(false);
                self.chunks.write().clear();
                self.error.set(Some(error));
            }
        }
    }

    /// Sets the asked-for devices' permissions, as a grant or a denial tells them.
    fn set_permissions(&mut self, state: PermissionState) {
        let options = self.options.peek().clone();
        for (asked, mut permission) in [
            (options.camera, self.camera_permission),
            (options.microphone, self.microphone_permission),
        ] {
            if asked && *permission.peek() != state {
                permission.set(state);
            }
        }
    }

    /// Spread on the preview `<video>` (`..media.attributes()`): the stream shows
    /// in the element carrying them. Keep it `muted`, so the microphone never echoes.
    /// An `ElementHandle`'s attributes may go on the same video.
    pub fn attributes(&self) -> Vec<Attribute> {
        vec![Attribute::new(
            CAPTURE_ATTR,
            AttributeValue::Text(self.tag.to_string()),
            None,
            false,
        )]
    }

    /// Whether this target can capture at all. `false` until mounted. Reactive.
    pub fn is_supported(&self) -> bool {
        (self.supported)()
    }

    /// Kept current where the platform reports changes. Reactive.
    pub fn camera_permission(&self) -> PermissionState {
        (self.camera_permission)()
    }

    /// Kept current where the platform reports changes. Reactive.
    pub fn microphone_permission(&self) -> PermissionState {
        (self.microphone_permission)()
    }

    /// Whether a stream is open: the device's light is on. Reactive.
    pub fn is_live(&self) -> bool {
        (self.live)()
    }

    /// The live camera's [`MediaDevice::id`], `None` when off or audio only.
    /// Reactive.
    pub fn camera_id(&self) -> Option<String> {
        (self.camera)()
    }

    /// Whether a [`start`](Self::start) waits for the user's answer. Reactive.
    pub fn is_pending(&self) -> bool {
        (self.pending)()
    }

    pub fn is_recording(&self) -> bool {
        (self.recording)()
    }

    /// Why the last step failed; the next success clears it. Reactive.
    pub fn error(&self) -> Option<UserMediaError> {
        (self.error)()
    }

    /// The last [`snapshot`](Self::snapshot), a PNG. Reactive.
    pub fn photo(&self) -> Option<FileData> {
        (self.photo)()
    }

    /// The last finished recording: WebM, or MP4 where only that records. Reactive.
    pub fn recording(&self) -> Option<FileData> {
        (self.recorded)()
    }
}

/// `stem` with the extension `content_type` names, as `photo.png`.
fn file_name(stem: &str, content_type: &str) -> String {
    let extension = content_type
        .split(['/', ';'])
        .nth(1)
        .filter(|extension| !extension.is_empty())
        .unwrap_or("bin");
    format!("{stem}.{extension}")
}

/// A [`UserMedia`] for this component. `options` apply to the next `start`
/// and `record`. Announces nothing: say a started or stopped camera and
/// recording once, in a status region.
pub fn use_user_media(options: UserMediaOptions) -> UserMedia {
    let mut options_signal = use_signal(|| options.clone());
    let mut chosen_camera = use_signal(|| None);
    if *options_signal.peek() != options {
        if options_signal.peek().camera_id != options.camera_id {
            chosen_camera.set(None);
        }
        options_signal.set(options);
    }
    let mut media = UserMedia {
        options: options_signal,
        chosen_camera,
        camera: use_signal(|| None),
        tag: use_hook(next_observe_tag),
        supported: use_signal(|| false),
        camera_permission: use_signal(PermissionState::default),
        microphone_permission: use_signal(PermissionState::default),
        live: use_signal(|| false),
        pending: use_signal(|| false),
        recording: use_signal(|| false),
        error: use_signal(|| None),
        photo: use_signal(|| None),
        recorded: use_signal(|| None),
        chunks: use_signal(Vec::new),
        session: use_signal(|| None),
    };
    let listeners: Rc<RefCell<Vec<Box<dyn PermissionSubscription>>>> =
        use_hook(|| Rc::new(RefCell::new(Vec::new())));
    use_drop({
        let listeners = listeners.clone();
        move || {
            listeners.borrow_mut().clear();
            media.session.write_unchecked().take();
        }
    });
    // A server cannot know either answer, so both are read after mount (hydration).
    use_effect(move || {
        let supported = capture().is_some();
        media.supported.set(supported);
        if !supported {
            media.camera_permission.set(PermissionState::Unsupported);
            media
                .microphone_permission
                .set(PermissionState::Unsupported);
            return;
        }
        let Some(api) = permission() else {
            return;
        };
        let follow = |kind, permission: Signal<PermissionState>| {
            api.on_change(
                kind,
                Box::new(move |state| {
                    let mut permission = permission;
                    if *permission.peek() != state {
                        permission.set(state);
                    }
                }),
            )
        };
        *listeners.borrow_mut() = vec![
            follow(PermissionKind::Camera, media.camera_permission),
            follow(PermissionKind::Microphone, media.microphone_permission),
        ];
    });
    media
}

/// The page's cameras and microphones, kept current as devices come and go.
/// Labels stay empty until the user granted a camera or microphone; a
/// [`UserMedia`] going live lists them again.
#[derive(Clone, Copy)]
pub struct UserMediaDevices {
    supported: Signal<bool>,
    list: Signal<DeviceList>,
    generation: Signal<u32>,
}

impl UserMediaDevices {
    /// Lists the devices again.
    pub fn refresh(&mut self) {
        *self.generation.write() += 1;
    }

    /// Whether this target lists devices. `false` until mounted. Reactive.
    pub fn is_supported(&self) -> bool {
        (self.supported)()
    }

    pub fn cameras(&self) -> Vec<MediaDevice> {
        self.list.read().cameras.clone()
    }

    pub fn microphones(&self) -> Vec<MediaDevice> {
        self.list.read().microphones.clone()
    }
}

/// A [`UserMediaDevices`] for this component, for a camera or microphone picker
/// feeding [`UserMediaOptions::camera_id`]. Listing never prompts.
pub fn use_user_media_devices() -> UserMediaDevices {
    let devices = UserMediaDevices {
        supported: use_signal(|| false),
        list: use_signal(DeviceList::default),
        generation: use_signal(|| 0),
    };
    let listener: Rc<RefCell<Option<Box<dyn CaptureSubscription>>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    use_drop({
        let listener = listener.clone();
        move || drop(listener.borrow_mut().take())
    });
    use_effect(move || {
        let _ = (devices.generation)();
        listener.borrow_mut().take();
        let Some(api) = capture() else {
            return;
        };
        let changes = api.on_devices(Box::new(move |list| {
            let (mut supported, mut current) = (devices.supported, devices.list);
            supported.set(list.is_some());
            current.set(list.unwrap_or_default());
        }));
        *listener.borrow_mut() = Some(changes);
    });
    devices
}

#[cfg(test)]
mod tests {
    use super::file_name;

    #[test]
    fn file_names_follow_the_type() {
        assert_eq!(file_name("photo", "image/png"), "photo.png");
        assert_eq!(file_name("recording", "video/webm"), "recording.webm");
        assert_eq!(file_name("recording", ""), "recording.bin");
    }
}
