//! The Android WebView saves no Blob download: bytes go to the system share
//! sheet, small text inline (todo 2016), the rest as a file in Downloads (2078).

use super::super::android_share::{on_main_thread, share_file, share_text};
use super::super::share::ShareFile;
use super::{SaveFileApi, SaveOutcome, Saving};

/// Under the Binder's 1 MB transaction cap, which an intent's extras travel through.
const MAX_TEXT: usize = 500_000;

pub(super) struct AndroidSaveFile;

pub(super) static SAVE_FILE: AndroidSaveFile = AndroidSaveFile;

/// What goes to the share sheet: small text inline, anything else as a file.
enum Payload {
    Text(String),
    File(ShareFile),
}

impl Payload {
    fn new(name: String, mime: &str, bytes: Vec<u8>) -> Self {
        match String::from_utf8(bytes) {
            Ok(text) if text.len() <= MAX_TEXT => Self::Text(text),
            Ok(text) => Self::file(name, mime, text.into_bytes()),
            Err(error) => Self::file(name, mime, error.into_bytes()),
        }
    }

    fn file(name: String, mime: &str, bytes: Vec<u8>) -> Self {
        let mime = mime.to_string();
        Self::File(ShareFile { name, mime, bytes })
    }
}

impl SaveFileApi for AndroidSaveFile {
    fn save(&self, name: &str, mime: &str, bytes: Vec<u8>) -> Saving {
        let name = name.to_string();
        let payload = Payload::new(name.clone(), mime, bytes);
        let shared = on_main_thread(move |env, activity| match &payload {
            Payload::Text(text) => {
                share_text(env, activity, &name, text).map_err(|error| error.to_string())
            }
            Payload::File(file) => share_file(env, activity, file),
        });
        Box::pin(async move {
            match shared.await {
                Ok(()) => SaveOutcome::Shared,
                Err(error) => SaveOutcome::Failed(error),
            }
        })
    }
}
