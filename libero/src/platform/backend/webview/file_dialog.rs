use dioxus::html::FileData;
use serde_json::json;

use super::{Slot, decode_base64, eval_with, runs_scripts};
use crate::platform::file_dialog::{FileDialogApi, Picked, held_file};
use crate::platform::image_crop::{CROP_SCRIPT, Cropped, Fractions, ImageCropApi, cropped_name};
use crate::utils::encode_base64;

pub(crate) fn file_dialog() -> Option<&'static dyn FileDialogApi> {
    runs_scripts().then_some(&FILE_DIALOG as &'static dyn FileDialogApi)
}

pub(crate) fn image_crop() -> Option<&'static dyn ImageCropApi> {
    runs_scripts().then_some(&IMAGE_CROP as &'static dyn ImageCropApi)
}

struct WebViewFileDialog;

static FILE_DIALOG: WebViewFileDialog = WebViewFileDialog;

/// A detached input: dioxus hands a click on one of its own inputs to the host,
/// which picks nothing on Android. Sends `[name, type, lastModified, base64]`s.
const PICK_FILES: &str = "const [accept, multiple, capture] = data;
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = accept;
    input.multiple = multiple;
    if (capture !== null) input.setAttribute('capture', capture);
    const files = await new Promise((done) => {
        input.addEventListener('change', () => done([...input.files]));
        input.addEventListener('cancel', () => done([]));
        input.click();
    });
    const encode = (bytes) => {
        let binary = '';
        for (let at = 0; at < bytes.length; at += 0x8000) {
            binary += String.fromCharCode(...bytes.subarray(at, at + 0x8000));
        }
        return btoa(binary);
    };
    dioxus.send(await Promise.all(files.map(async (file) =>
        [file.name, file.type, file.lastModified, encode(new Uint8Array(await file.arrayBuffer()))])));";

impl FileDialogApi for WebViewFileDialog {
    fn open(&self, accept: &str, multiple: bool, capture: Option<&str>) -> Picked {
        // Parked, not `join`ed: the dialog and the encoding can outlast `CHANNEL_HOLD_MS`.
        let slot = Slot::new();
        let script = eval_with(
            json!([accept, multiple, capture]),
            &format!("{PICK_FILES}\n{}", slot.park("")),
        );
        Box::pin(async move {
            let mut script = script;
            let picked = script
                .recv::<Vec<(String, String, f64, String)>>()
                .await
                .unwrap_or_default();
            drop(slot);
            picked
                .into_iter()
                .filter_map(|(name, content_type, modified, encoded)| {
                    let bytes = decode_base64(&encoded)?;
                    Some(held_file(name, content_type, modified as u64, bytes))
                })
                .collect()
        })
    }
}

struct WebViewImageCrop;

static IMAGE_CROP: WebViewImageCrop = WebViewImageCrop;

impl ImageCropApi for WebViewImageCrop {
    fn crop(&self, file: FileData, rect: Fractions, max: Option<u32>) -> Cropped {
        Box::pin(async move {
            let bytes = file.read_bytes().await.ok()?;
            let from = file.content_type().unwrap_or_default();
            // Both ways as base64: the IPC carries JSON only.
            // Parked, not `join`ed: a large image can crop past `CHANNEL_HOLD_MS`.
            let slot = Slot::new();
            let script = eval_with(
                json!([encode_base64(&bytes), from, rect, max]),
                &format!(
                    "{CROP_SCRIPT}
                    const [encoded, type, rect, max] = data;
                    const bytes = Uint8Array.from(atob(encoded), (ch) => ch.charCodeAt(0));
                    const blob = await crop(bytes, type, rect, max);
                    const out = new Uint8Array(await blob.arrayBuffer());
                    let binary = '';
                    for (let at = 0; at < out.length; at += 0x8000) {{
                        binary += String.fromCharCode(...out.subarray(at, at + 0x8000));
                    }}
                    dioxus.send([blob.type, btoa(binary)]);
                    {}",
                    slot.park("")
                ),
            );
            let mut script = script;
            let answer = script.recv::<(String, String)>().await.ok();
            drop(slot);
            let (to, encoded) = answer?;
            let name = cropped_name(&file.name(), &from, &to);
            Some(held_file(
                name,
                to,
                file.last_modified(),
                decode_base64(&encoded)?,
            ))
        })
    }
}
