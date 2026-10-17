use serde_json::json;

use super::{eval_with, runs_scripts};
use crate::platform::PlatformError;
use crate::platform::clipboard::{ClipboardApi, Write};

pub(crate) fn clipboard() -> Option<&'static dyn ClipboardApi> {
    runs_scripts().then_some(&CLIPBOARD as &'static dyn ClipboardApi)
}

struct WebViewClipboard;

static CLIPBOARD: WebViewClipboard = WebViewClipboard;

impl ClipboardApi for WebViewClipboard {
    fn write_text(&self, text: String) -> Write {
        // The WebView denies `clipboard-write` unless the app grants it, so a
        // rejected write falls back to `execCommand`, which the tap's activation allows.
        let script = eval_with(
            json!(text),
            "const text = data;
            try { await navigator.clipboard.writeText(text); return true; } catch (error) {}
            const area = document.createElement('textarea');
            area.value = text;
            area.setAttribute('readonly', '');
            area.style.cssText = 'position:fixed;opacity:0;pointer-events:none';
            const active = document.activeElement;
            document.body.append(area);
            area.select();
            let copied = false;
            try { copied = document.execCommand('copy'); } catch (error) {}
            area.remove();
            active?.focus?.({ preventScroll: true });
            return copied;",
        );
        Box::pin(async move {
            match script.join::<bool>().await {
                Ok(true) => Ok(()),
                Ok(false) => Err(PlatformError::Denied),
                Err(_) => Err(PlatformError::Unsupported),
            }
        })
    }
}
