use serde_json::{Value, json};

use super::{Slot, eval_with, runs_scripts};
use crate::platform::share::{SHARE_SCRIPT, ShareApi, ShareData, Sharing, outcome_of, text_fields};
use crate::utils::encode_base64;

/// Desktop WebViews and iOS; Android's WebView has no `navigator.share`.
pub(crate) fn share() -> Option<&'static dyn ShareApi> {
    runs_scripts().then_some(&SHARE as &'static dyn ShareApi)
}

struct WebViewShare;

static SHARE: WebViewShare = WebViewShare;

impl ShareApi for WebViewShare {
    fn share(&self, data: ShareData) -> Sharing {
        let files: Vec<Value> = data
            .files
            .iter()
            .map(|file| json!([file.name, file.mime, encode_base64(&file.bytes)]))
            .collect();
        let [title, text, url] = text_fields(&data);
        // Parked, not `join`ed: the sheet can outlast `CHANNEL_HOLD_MS`.
        let slot = Slot::new();
        let script = eval_with(
            json!([title, text, url, files]),
            &format!(
                "{SHARE_SCRIPT}\ndioxus.send(await share(...data));\n{}",
                slot.park("")
            ),
        );
        Box::pin(async move {
            let mut script = script;
            let answer = script.recv::<String>().await.ok();
            drop(slot);
            outcome_of(answer)
        })
    }
}
