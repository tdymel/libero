//! Android's WebView: the web's engine, but Rust holds no handle on its DOM.
//! What needs no element handle goes through the page's own script, over
//! `document::eval` - the same way [`fetch_text`](crate::platform::fetch_text)
//! does. Elements stay on the [`mounted`](super::mounted) floor.
//!
//! Android only: a desktop WebView shares the cfg with a server build, which
//! has no page to run a script in.

use std::cell::Cell;

use dioxus::core::{Runtime, Task, spawn_forever};
use dioxus::document::eval;
use dioxus::prelude::spawn;

use crate::platform::{
    Dimensions, DocumentApi, ElementApi, PlatformError, Read, ScrollApi, ScrollSubscription,
    clipboard::{ClipboardApi, Write},
};

/// `None` outside a dioxus runtime: every answer is a script the runtime sends.
pub(super) fn document() -> Option<&'static dyn DocumentApi> {
    Runtime::try_current().map(|_| &DOCUMENT as &'static dyn DocumentApi)
}

pub(super) fn scroll() -> Option<&'static dyn ScrollApi> {
    Runtime::try_current().map(|_| &SCROLL as &'static dyn ScrollApi)
}

pub(crate) fn clipboard() -> Option<&'static dyn ClipboardApi> {
    Runtime::try_current().map(|_| &CLIPBOARD as &'static dyn ClipboardApi)
}

struct WebViewDocument;

static DOCUMENT: WebViewDocument = WebViewDocument;

impl DocumentApi for WebViewDocument {
    /// No element handle can name the answer (todo 5).
    fn active_element(&self) -> Option<Box<dyn ElementApi>> {
        None
    }

    fn viewport(&self) -> Read<Dimensions> {
        let read = eval("return [window.innerWidth, window.innerHeight];");
        Box::pin(async move {
            let (width, height) = read
                .join::<(f64, f64)>()
                .await
                .map_err(|_| PlatformError::Unsupported)?;
            Ok(Dimensions { width, height })
        })
    }

    /// Queued: the script runs before any edit dioxus sends after it.
    fn set_root_attribute(&self, name: &str, value: Option<&str>) -> bool {
        let script = eval(
            "const [name, value] = await dioxus.recv();
            const root = document.documentElement;
            value === null ? root.removeAttribute(name) : root.setAttribute(name, value);",
        );
        script.send((name, value)).is_ok()
    }
}

struct WebViewScroll;

static SCROLL: WebViewScroll = WebViewScroll;

/// Capture phase on the window, as on the web: a scroll does not bubble. One
/// message per frame at most, since each crosses the IPC.
const ON_SCROLL: &str = "let queued = false;
    const tick = () => {
        if (queued) return;
        queued = true;
        requestAnimationFrame(() => { queued = false; dioxus.send(null); });
    };
    window.addEventListener('scroll', tick, { capture: true, passive: true });
    await dioxus.recv();
    window.removeEventListener('scroll', tick, { capture: true });";

impl ScrollApi for WebViewScroll {
    fn on_scroll(&self, callback: Box<dyn Fn()>) -> Box<dyn ScrollSubscription> {
        let script = eval(ON_SCROLL);
        let task = spawn(async move {
            let mut script = script;
            while script.recv::<()>().await.is_ok() {
                callback();
            }
        });
        Box::new(WebViewScrollSubscription { script, task })
    }
}

struct WebViewScrollSubscription {
    script: dioxus::document::Eval,
    task: Task,
}

impl ScrollSubscription for WebViewScrollSubscription {}

impl Drop for WebViewScrollSubscription {
    fn drop(&mut self) {
        self.task.cancel();
        let _ = self.script.send(());
    }
}

struct WebViewClipboard;

static CLIPBOARD: WebViewClipboard = WebViewClipboard;

impl ClipboardApi for WebViewClipboard {
    fn write_text(&self, text: String) -> Write {
        // The WebView denies `clipboard-write` unless the app grants it, so a
        // rejected write falls back to `execCommand`, which the tap's activation allows.
        let script = eval(
            "const text = await dioxus.recv();
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
        let sent = script.send(text);
        Box::pin(async move {
            sent.map_err(|_| PlatformError::Unsupported)?;
            match script.join::<bool>().await {
                Ok(true) => Ok(()),
                Ok(false) => Err(PlatformError::Denied),
                Err(_) => Err(PlatformError::Unsupported),
            }
        })
    }
}

thread_local! {
    /// The media query's last answer; `None` until a listener is running.
    static REDUCED_MOTION: Cell<Option<bool>> = const { Cell::new(None) };
}

/// `false` until the page first answers, then kept live by a listener.
pub(super) fn prefers_reduced_motion() -> bool {
    if let Some(reduced) = REDUCED_MOTION.get() {
        return reduced;
    }
    if Runtime::try_current().is_none() {
        return false;
    }
    REDUCED_MOTION.set(Some(false));
    spawn_forever(async {
        let mut script = eval(&format!(
            "const query = window.matchMedia('{}');
            dioxus.send(query.matches);
            query.addEventListener('change', () => dioxus.send(query.matches));
            await new Promise(() => {{}});",
            crate::sx::REDUCED_MOTION
        ));
        while let Ok(reduced) = script.recv::<bool>().await {
            REDUCED_MOTION.set(Some(reduced));
        }
    });
    false
}
