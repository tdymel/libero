//! Camera and microphone capture. The stream is a page object Rust cannot hold,
//! so one script keeps it, shows it in the tagged `<video>` and sends events back.

use dioxus::html::FileData;
use serde_json::Value;

/// Carries the preview `<video>`'s tag; not [`OBSERVE_ATTR`](super::OBSERVE_ATTR),
/// which an `ElementHandle` spread on the same video sets too (1237).
pub(crate) const CAPTURE_ATTR: &str = "data-lsx-capture";

/// Why the camera or microphone could not be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserMediaError {
    /// No capture API on this target, or off a secure context.
    Unsupported,
    /// The user or the platform refused.
    Denied,
    /// No such camera or microphone.
    NotFound,
    /// The device is busy elsewhere, or the system blocked it.
    InUse,
    /// The chosen device cannot be had with these options.
    Constraint,
    /// A recording outgrew its `max_bytes`; it was dropped.
    TooLarge,
    /// Anything else, such as no frame to snapshot yet.
    Failed,
}

impl UserMediaError {
    /// From a `DOMException.name`, or the script's own names.
    #[cfg_attr(feature = "native", allow(dead_code))]
    pub(crate) fn from_name(name: &str) -> Self {
        match name {
            "NotAllowedError" | "SecurityError" => Self::Denied,
            "NotFoundError" => Self::NotFound,
            "NotReadableError" | "AbortError" => Self::InUse,
            "OverconstrainedError" | "TypeError" => Self::Constraint,
            "Unsupported" => Self::Unsupported,
            "TooLarge" => Self::TooLarge,
            _ => Self::Failed,
        }
    }
}

/// One camera or microphone. `label` stays empty until the user granted one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaDevice {
    pub id: String,
    pub label: String,
}

/// The page's capture devices.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DeviceList {
    pub(crate) cameras: Vec<MediaDevice>,
    pub(crate) microphones: Vec<MediaDevice>,
}

impl DeviceList {
    /// From the script's `[kind, id, label]` rows.
    #[cfg_attr(feature = "native", allow(dead_code))]
    pub(crate) fn from_rows(rows: &Value) -> Self {
        let mut list = Self::default();
        for row in rows.as_array().into_iter().flatten() {
            let text = |at: usize| row.get(at).and_then(Value::as_str).unwrap_or_default();
            let device = MediaDevice {
                id: text(1).to_string(),
                label: text(2).to_string(),
            };
            match text(0) {
                "videoinput" => list.cameras.push(device),
                "audioinput" => list.microphones.push(device),
                _ => {}
            }
        }
        list
    }
}

/// What a capture script reports.
#[cfg_attr(feature = "native", allow(dead_code))]
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CaptureEvent {
    /// The camera's device id, `None` without a camera track.
    Live(Option<String>),
    /// Every track ended: unplugged, revoked, or stopped by the system.
    Ended,
    Failed(UserMediaError),
    /// A frame's type and bytes; `None` without a frame.
    Photo(Option<(String, Vec<u8>)>),
    Recording,
    Chunk(Vec<u8>),
    /// The recording's type, after its last chunk.
    Recorded(String),
    RecordFailed(UserMediaError),
}

impl CaptureEvent {
    /// From a script message; `bytes` is its `bytes` member, already decoded.
    #[cfg_attr(feature = "native", allow(dead_code))]
    pub(crate) fn from_message(message: &Value, bytes: Option<Vec<u8>>) -> Option<Self> {
        let text = |name: &str| message.get(name).and_then(Value::as_str);
        Some(if let Some(camera) = message.get("live") {
            Self::Live(camera.as_str().map(str::to_string))
        } else if message.get("ended").is_some() {
            Self::Ended
        } else if let Some(name) = text("error") {
            Self::Failed(UserMediaError::from_name(name))
        } else if let Some(photo) = message.get("photo") {
            Self::Photo(
                photo
                    .as_str()
                    .zip(bytes)
                    .map(|(kind, bytes)| (kind.to_string(), bytes)),
            )
        } else if message.get("recording").is_some() {
            Self::Recording
        } else if message.get("chunk").is_some() {
            Self::Chunk(bytes?)
        } else if let Some(kind) = text("recorded") {
            Self::Recorded(kind.to_string())
        } else {
            Self::RecordFailed(UserMediaError::from_name(text("recordError")?))
        })
    }
}

/// `getUserMedia` constraints for the chosen devices; `None` to open neither.
/// `facing` (`user`, `environment`) applies to a camera chosen by no id.
pub(crate) fn constraints(
    camera: Option<Option<&str>>,
    microphone: Option<Option<&str>>,
    facing: Option<&str>,
) -> Option<Value> {
    let track = |wanted: Option<Option<&str>>, facing: Option<&str>| match (wanted, facing) {
        (None, _) => Value::Bool(false),
        (Some(None), None) => Value::Bool(true),
        (Some(None), Some(facing)) => serde_json::json!({ "facingMode": { "ideal": facing } }),
        (Some(Some(id)), _) => serde_json::json!({ "deviceId": { "exact": id } }),
    };
    (camera.is_some() || microphone.is_some()).then(
        || serde_json::json!({ "video": track(camera, facing), "audio": track(microphone, None) }),
    )
}

/// A running capture script. Dropping it stops every track.
pub(crate) trait CaptureSession {
    /// Calls the script's `command`.
    fn call(&self, command: &str, argument: Value);
}

impl dyn CaptureSession {
    /// Opens `constraints`, replacing a stream already open.
    pub(crate) fn open(&self, constraints: Value) {
        self.call("open", constraints);
    }

    /// Ends a recording, then stops every track.
    pub(crate) fn close(&self) {
        self.call("close", Value::Null);
    }

    pub(crate) fn snapshot(&self) {
        self.call("snapshot", Value::Null);
    }

    pub(crate) fn record(&self, max_bytes: Option<u64>) {
        self.call("record", serde_json::json!(max_bytes));
    }

    pub(crate) fn finish(&self) {
        self.call("finish", Value::Null);
    }
}

/// Dropping it stops listening for device changes.
pub(crate) trait CaptureSubscription {}

pub(crate) trait CaptureApi {
    /// A script that shows its stream in the element carrying `tag`.
    fn session(&self, tag: u64, callback: Box<dyn Fn(CaptureEvent)>) -> Box<dyn CaptureSession>;
    /// Calls `callback` with the devices now and on each change; `None` without the API.
    fn on_devices(&self, callback: Box<dyn Fn(Option<DeviceList>)>)
    -> Box<dyn CaptureSubscription>;
}

/// The session script, run by a browser and a WebView alike: `data` is
/// `[attr, tag]`, `send` posts a message, `encode` readies bytes for it.
#[cfg(not(feature = "native"))]
pub(crate) const CAPTURE_SCRIPT: &str = "const [attr, tag] = data;
    const GRANTED = 'lsx-capture-granted';
    const TYPES = ['video/webm;codecs=vp9,opus', 'video/webm', 'video/mp4', 'audio/webm;codecs=opus', 'audio/webm', 'audio/mp4'];
    let stream = null, ticket = 0, recorder = null, dropped = false, queue = Promise.resolve();
    const find = () => document.querySelector('[' + attr + '=\"' + tag + '\"]');
    const show = () => {
        const video = find();
        if (video && video.srcObject !== stream) video.srcObject = stream;
    };
    // While live: the `<video>` may render after the stream opens, or render anew.
    const watching = new MutationObserver(show);
    const release = () => {
        watching.disconnect();
        stream?.getTracks().forEach((track) => track.stop());
        stream = null;
        show();
    };
    const finish = () => {
        if (recorder && recorder.state !== 'inactive') recorder.stop();
    };
    const close = () => {
        ticket++;
        finish();
        release();
    };
    const open = async (constraints) => {
        const mine = ++ticket;
        if (!navigator.mediaDevices?.getUserMedia) return send({ error: 'Unsupported' });
        let next;
        try {
            next = await navigator.mediaDevices.getUserMedia(constraints);
        } catch (error) {
            if (mine === ticket) send({ error: error?.name ?? 'Error' });
            return;
        }
        // Closed, or opened again, while the prompt waited.
        if (mine !== ticket) return next.getTracks().forEach((track) => track.stop());
        finish();
        release();
        stream = next;
        for (const track of next.getTracks()) {
            track.addEventListener('ended', () => {
                if (stream !== next || next.getTracks().some((t) => t.readyState === 'live')) return;
                finish();
                release();
                send({ ended: true });
            });
        }
        watching.observe(document.documentElement, { childList: true, subtree: true, attributes: true, attributeFilter: [attr] });
        show();
        send({ live: next.getVideoTracks()[0]?.getSettings?.().deviceId ?? null });
        // A grant reveals the device ids and labels: device lists read them again.
        dispatchEvent(new Event(GRANTED));
    };
    const snapshot = async () => {
        const video = find();
        if (!stream?.getVideoTracks().length || !video?.videoWidth) return send({ photo: null });
        const canvas = document.createElement('canvas');
        canvas.width = video.videoWidth;
        canvas.height = video.videoHeight;
        canvas.getContext('2d').drawImage(video, 0, 0);
        const blob = await new Promise((done) => canvas.toBlob(done, 'image/png'));
        if (!blob) return send({ photo: null });
        send({ photo: blob.type, bytes: encode(new Uint8Array(await blob.arrayBuffer())) });
    };
    const record = (max) => {
        if (!stream || recorder) return;
        const media = stream.getVideoTracks().length ? 'video' : 'audio';
        const type = TYPES.find((t) => t.startsWith(media) && MediaRecorder.isTypeSupported?.(t));
        let current;
        try {
            current = new MediaRecorder(stream, type ? { mimeType: type } : {});
        } catch (error) {
            return send({ recordError: 'Failed' });
        }
        recorder = current;
        dropped = false;
        let size = 0;
        // Chained: each chunk's bytes are read async, and must leave in order.
        current.addEventListener('dataavailable', (event) => {
            if (dropped || !event.data.size) return;
            size += event.data.size;
            if (max !== null && size > max) {
                dropped = true;
                current.stop();
                queue = queue.then(() => send({ recordError: 'TooLarge' }));
                return;
            }
            const data = event.data;
            queue = queue.then(async () => send({ chunk: true, bytes: encode(new Uint8Array(await data.arrayBuffer())) }));
        });
        current.addEventListener('stop', () => {
            queue = queue.then(() => {
                if (recorder === current) recorder = null;
                if (!dropped) send({ recorded: current.mimeType || type || media + '/webm' });
            });
        });
        current.start(1000);
        send({ recording: true });
    };
    const session = { open, close, snapshot, record, finish };";

/// The device list script: `send`s `[kind, id, label]` rows now, on each
/// change and after each capture grant, or `null` without the API; `stop` ends it.
#[cfg(not(feature = "native"))]
pub(crate) const DEVICES_SCRIPT: &str = "const devices = navigator.mediaDevices;
    const list = async () => {
        if (!devices?.enumerateDevices) return send(null);
        const all = await devices.enumerateDevices().catch(() => []);
        send(all.map((d) => [d.kind, d.deviceId, d.label]));
    };
    list();
    devices?.addEventListener('devicechange', list);
    addEventListener('lsx-capture-granted', list);
    const stop = () => {
        devices?.removeEventListener('devicechange', list);
        removeEventListener('lsx-capture-granted', list);
    };";

/// `bytes` as a file a form can post, `None` where no page holds files.
pub(crate) fn file_from_bytes(name: &str, content_type: &str, bytes: Vec<u8>) -> Option<FileData> {
    #[cfg(target_arch = "wasm32")]
    return super::image_crop::web::bytes_file(
        name,
        content_type,
        &js_sys::Uint8Array::from(bytes.as_slice()),
    );
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return Some(super::file_dialog::held_file(
        name.to_string(),
        content_type.to_string(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_millis() as u64),
        bytes,
    ));
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    {
        let _ = (name, content_type, bytes);
        None
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use js_sys::{Function, JSON, Object, Reflect, Uint8Array};
    use serde_json::Value;
    use wasm_bindgen::{JsCast, JsValue, closure::Closure};

    use super::CAPTURE_ATTR;
    use super::{
        CAPTURE_SCRIPT, CaptureApi, CaptureEvent, CaptureSession, CaptureSubscription,
        DEVICES_SCRIPT, DeviceList,
    };

    /// `navigator.mediaDevices`, by name: absent off a secure context.
    pub(super) fn media_devices() -> Option<Object> {
        let navigator = web_sys::window()?.navigator();
        Reflect::get(&navigator, &JsValue::from_str("mediaDevices"))
            .ok()?
            .dyn_into::<Object>()
            .ok()
    }

    /// A message as JSON, its `bytes` member lifted out first.
    fn read(message: &JsValue) -> (Value, Option<Vec<u8>>) {
        let bytes = Reflect::get(message, &"bytes".into())
            .ok()
            .and_then(|bytes| bytes.dyn_into::<Uint8Array>().ok())
            .map(|bytes| bytes.to_vec());
        if bytes.is_some()
            && let Some(object) = message.dyn_ref::<Object>()
        {
            let _ = Reflect::delete_property(object, &"bytes".into());
        }
        let json = JSON::stringify(message)
            .ok()
            .and_then(|text| text.as_string())
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or(Value::Null);
        (json, bytes)
    }

    /// Runs `script`, then returns its `result` object plus `detach`, after
    /// which a late message no longer reaches the dropped closure.
    fn run(
        script: &str,
        result: &str,
        data: JsValue,
        send: &Closure<dyn Fn(JsValue)>,
    ) -> Option<Object> {
        let run = Function::new_with_args(
            "post, encode, data",
            &format!(
                "const send = (message) => post?.(message);
                {script}
                return {{ ...{result}, detach: () => {{ post = null; }} }};"
            ),
        );
        let encode = Function::new_with_args("bytes", "return bytes;");
        run.call3(&JsValue::NULL, send.as_ref(), &encode, &data)
            .ok()?
            .dyn_into::<Object>()
            .ok()
    }

    fn call(target: &Object, name: &str, argument: &JsValue) {
        if let Ok(method) = Reflect::get(target, &JsValue::from_str(name))
            && let Ok(method) = method.dyn_into::<Function>()
        {
            let _ = method.call1(target, argument);
        }
    }

    pub(super) struct WebCapture;

    impl CaptureApi for WebCapture {
        fn session(
            &self,
            tag: u64,
            callback: Box<dyn Fn(CaptureEvent)>,
        ) -> Box<dyn CaptureSession> {
            let send = Closure::<dyn Fn(JsValue)>::new(move |message: JsValue| {
                let (message, bytes) = read(&message);
                if let Some(event) = CaptureEvent::from_message(&message, bytes) {
                    callback(event);
                }
            });
            let data = js_sys::Array::of2(&CAPTURE_ATTR.into(), &tag.to_string().into());
            let session = run(CAPTURE_SCRIPT, "session", data.into(), &send);
            Box::new(WebSession {
                session,
                _send: send,
            })
        }

        fn on_devices(
            &self,
            callback: Box<dyn Fn(Option<DeviceList>)>,
        ) -> Box<dyn CaptureSubscription> {
            let send = Closure::<dyn Fn(JsValue)>::new(move |rows: JsValue| {
                let (rows, _) = read(&rows);
                callback((!rows.is_null()).then(|| DeviceList::from_rows(&rows)));
            });
            let stop = run(DEVICES_SCRIPT, "{ stop }", JsValue::NULL, &send);
            Box::new(WebDevices { stop, _send: send })
        }
    }

    struct WebSession {
        session: Option<Object>,
        _send: Closure<dyn Fn(JsValue)>,
    }

    impl CaptureSession for WebSession {
        fn call(&self, command: &str, argument: Value) {
            let argument = JSON::parse(&argument.to_string()).unwrap_or(JsValue::NULL);
            if let Some(session) = &self.session {
                call(session, command, &argument);
            }
        }
    }

    impl Drop for WebSession {
        fn drop(&mut self) {
            if let Some(session) = &self.session {
                call(session, "close", &JsValue::NULL);
                call(session, "detach", &JsValue::NULL);
            }
        }
    }

    struct WebDevices {
        stop: Option<Object>,
        _send: Closure<dyn Fn(JsValue)>,
    }

    impl CaptureSubscription for WebDevices {}

    impl Drop for WebDevices {
        fn drop(&mut self) {
            if let Some(stop) = &self.stop {
                call(stop, "stop", &JsValue::NULL);
                call(stop, "detach", &JsValue::NULL);
            }
        }
    }

    pub(super) static CAPTURE: WebCapture = WebCapture;
}

/// `None` without `navigator.mediaDevices`: Blitz, a server, an insecure page.
/// Call it after mount: a server render answers `None`, a hydrating client may not.
pub(crate) fn capture() -> Option<&'static dyn CaptureApi> {
    #[cfg(target_arch = "wasm32")]
    return web::media_devices().map(|_| &web::CAPTURE as &'static dyn CaptureApi);
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return None;
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return super::backend::webview_capture();
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{CaptureEvent, DeviceList, MediaDevice, UserMediaError, constraints};

    #[test]
    fn dom_exception_names_map_to_errors() {
        assert_eq!(
            UserMediaError::from_name("NotAllowedError"),
            UserMediaError::Denied
        );
        assert_eq!(
            UserMediaError::from_name("NotFoundError"),
            UserMediaError::NotFound
        );
        assert_eq!(
            UserMediaError::from_name("NotReadableError"),
            UserMediaError::InUse
        );
        assert_eq!(
            UserMediaError::from_name("OverconstrainedError"),
            UserMediaError::Constraint
        );
        assert_eq!(
            UserMediaError::from_name("Whatever"),
            UserMediaError::Failed
        );
    }

    #[test]
    fn messages_become_events() {
        let event = |message, bytes| CaptureEvent::from_message(&message, bytes);
        assert_eq!(
            event(json!({ "live": "c1" }), None),
            Some(CaptureEvent::Live(Some("c1".into())))
        );
        assert_eq!(
            event(json!({ "live": null }), None),
            Some(CaptureEvent::Live(None))
        );
        assert_eq!(
            event(json!({ "error": "NotAllowedError" }), None),
            Some(CaptureEvent::Failed(UserMediaError::Denied))
        );
        assert_eq!(
            event(json!({ "photo": "image/png" }), Some(vec![1])),
            Some(CaptureEvent::Photo(Some(("image/png".into(), vec![1]))))
        );
        assert_eq!(
            event(json!({ "photo": null }), None),
            Some(CaptureEvent::Photo(None))
        );
        assert_eq!(
            event(json!({ "chunk": true }), Some(vec![2])),
            Some(CaptureEvent::Chunk(vec![2]))
        );
        assert_eq!(
            event(json!({ "recordError": "TooLarge" }), None),
            Some(CaptureEvent::RecordFailed(UserMediaError::TooLarge))
        );
        assert_eq!(event(json!({ "other": 1 }), None), None);
    }

    #[test]
    fn device_rows_split_by_kind() {
        let list = DeviceList::from_rows(&json!([
            ["videoinput", "c1", "Front"],
            ["audioinput", "m1", ""],
            ["audiooutput", "s1", "Speaker"],
        ]));
        assert_eq!(
            list.cameras,
            vec![MediaDevice {
                id: "c1".into(),
                label: "Front".into()
            }]
        );
        assert_eq!(list.microphones.len(), 1);
    }

    #[test]
    fn constraints_name_the_chosen_devices() {
        assert_eq!(constraints(None, None, None), None);
        assert_eq!(
            constraints(Some(Some("c1")), Some(None), Some("user")),
            Some(json!({ "video": { "deviceId": { "exact": "c1" } }, "audio": true }))
        );
        assert_eq!(
            constraints(Some(None), None, Some("environment")),
            Some(json!({ "video": { "facingMode": { "ideal": "environment" } }, "audio": false }))
        );
    }
}
