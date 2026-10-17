use serde_json::json;

use super::{eval_with, runs_scripts};
use crate::platform::SaveOutcome;
use crate::platform::save_file::{SaveFileApi, Saving};
use crate::utils::encode_base64;

/// A liveview or fullstack tab: the browser downloads it.
#[cfg(all(
    not(target_os = "android"),
    not(feature = "native"),
    not(feature = "desktop")
))]
pub(crate) fn save_file() -> Option<&'static dyn SaveFileApi> {
    runs_scripts().then_some(&SAVE_FILE as &'static dyn SaveFileApi)
}

struct PageSaveFile;

static SAVE_FILE: PageSaveFile = PageSaveFile;

impl SaveFileApi for PageSaveFile {
    fn save(&self, name: &str, mime: &str, bytes: Vec<u8>) -> Saving {
        eval_with(
            json!([name, mime, encode_base64(&bytes)]),
            "const [name, type, base64] = data;
            const bytes = Uint8Array.from(atob(base64), (char) => char.charCodeAt(0));
            const url = URL.createObjectURL(new Blob([bytes], { type }));
            const link = document.createElement('a');
            link.href = url;
            link.download = name;
            document.body.append(link);
            link.click();
            link.remove();
            setTimeout(() => URL.revokeObjectURL(url), 40000);",
        );
        Box::pin(std::future::ready(SaveOutcome::Saved))
    }
}
