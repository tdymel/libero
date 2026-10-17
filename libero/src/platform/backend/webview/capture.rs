use std::rc::Rc;

use dioxus::prelude::spawn;
use serde_json::{Value, json};

use super::{Slot, WebViewListener, decode_base64, eval, eval_with, runs_scripts};
use crate::platform::capture::{
    CAPTURE_ATTR, CAPTURE_SCRIPT, CaptureApi, CaptureEvent, CaptureSession, CaptureSubscription,
    DEVICES_SCRIPT, DeviceList,
};

pub(crate) fn capture() -> Option<&'static dyn CaptureApi> {
    runs_scripts().then_some(&CAPTURE as &'static dyn CaptureApi)
}

struct WebViewCapture;

static CAPTURE: WebViewCapture = WebViewCapture;

/// `send` and `encode` for the capture scripts: bytes cross the IPC as base64.
const CAPTURE_SEND: &str = "const send = (message) => dioxus.send(message);
    const encode = (bytes) => {
        let binary = '';
        for (let at = 0; at < bytes.length; at += 0x8000) {
            binary += String.fromCharCode(...bytes.subarray(at, at + 0x8000));
        }
        return btoa(binary);
    };";

impl CaptureApi for WebViewCapture {
    /// Parked, so a prompt or a recording may outlast [`CHANNEL_HOLD_MS`].
    fn session(&self, tag: u64, callback: Box<dyn Fn(CaptureEvent)>) -> Box<dyn CaptureSession> {
        let slot = Rc::new(Slot::new());
        let script = eval_with(
            json!([CAPTURE_ATTR, tag.to_string()]),
            &format!(
                "{CAPTURE_SEND}
                {CAPTURE_SCRIPT}
                {}
                close();",
                slot.park("open, close, snapshot, record, finish")
            ),
        );
        let task = spawn(async move {
            let mut script = script;
            while let Ok(mut message) = script.recv::<Value>().await {
                let bytes = message
                    .get_mut("bytes")
                    .map(Value::take)
                    .and_then(|bytes| decode_base64(bytes.as_str()?));
                if let Some(event) = CaptureEvent::from_message(&message, bytes) {
                    callback(event);
                }
            }
        });
        Box::new(WebViewCaptureSession(WebViewListener { task, _slot: slot }))
    }

    fn on_devices(
        &self,
        callback: Box<dyn Fn(Option<DeviceList>)>,
    ) -> Box<dyn CaptureSubscription> {
        let slot = Slot::new();
        let script = eval(&format!(
            "{CAPTURE_SEND}
            {DEVICES_SCRIPT}
            {}
            stop();",
            slot.park("")
        ));
        let task = spawn(async move {
            let mut script = script;
            while let Ok(rows) = script.recv::<Value>().await {
                callback((!rows.is_null()).then(|| DeviceList::from_rows(&rows)));
            }
        });
        Box::new(WebViewListener {
            task,
            _slot: Rc::new(slot),
        })
    }
}

struct WebViewCaptureSession(WebViewListener);

impl CaptureSession for WebViewCaptureSession {
    fn call(&self, command: &str, argument: Value) {
        self.0._slot.call(command, argument);
    }
}

impl CaptureSubscription for WebViewListener {}
