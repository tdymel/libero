use dioxus::prelude::*;
use wasm_bindgen_futures::JsFuture;

/// Writes text to the system clipboard - the only DOM-adjacent operation in
/// this crate that isn't element-scoped (no selector/`ElementApi` applies),
/// so it gets its own tiny trait here instead of living on `DomApi`. Only
/// this hook uses it, so it stays private to this module.
trait ClipboardApi {
    fn write_text(&self, text: &str);
}

struct WebClipboardHandle;

impl ClipboardApi for WebClipboardHandle {
    fn write_text(&self, text: &str) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let promise = window.navigator().clipboard().write_text(text);
        wasm_bindgen_futures::spawn_local(async move {
            let _ = JsFuture::from(promise).await;
        });
    }
}

static WEB_CLIPBOARD_HANDLE: WebClipboardHandle = WebClipboardHandle;

fn clipboard_api() -> &'static dyn ClipboardApi {
    &WEB_CLIPBOARD_HANDLE
}

/// Writes to the system clipboard and tracks a transient "just copied" flag
/// - callers decide when to clear it (e.g. `onmouseleave`).
#[derive(Clone, Copy)]
pub struct Clipboard {
    copied: Signal<bool>,
}

impl Clipboard {
    pub fn copy(&mut self, text: impl Into<String>) {
        clipboard_api().write_text(&text.into());
        self.copied.set(true);
    }

    pub fn reset(&mut self) {
        self.copied.set(false);
    }

    pub fn copied(&self) -> bool {
        (self.copied)()
    }
}

pub fn use_clipboard() -> Clipboard {
    Clipboard {
        copied: use_signal(|| false),
    }
}
