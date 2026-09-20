//! Android's WebView: the web's engine, but Rust holds no handle on its DOM.
//! What needs no element handle goes through the page's own script, over
//! `document::eval` - the same way [`fetch_text`](crate::platform::fetch_text)
//! does. Elements stay on the [`mounted`](super::mounted) floor.
//!
//! Android only: a desktop WebView shares the cfg with a server build, which
//! has no page to run a script in.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;

use dioxus::core::{Runtime, ScopeId, Task, spawn_forever};
use dioxus::document::eval;
use dioxus::prelude::{Key, Modifiers, spawn};

use crate::platform::{
    ColorSchemeApi, ColorSchemeSubscription, Dimensions, DocumentApi, ElementApi, KeyChord,
    KeySubscription, KeyboardApi, PlatformError, Read, ScrollApi, ScrollSubscription,
    clipboard::{ClipboardApi, Write},
    keyboard::{CLICKED_INPUT_TYPES, warn_reserved_chord},
};
use crate::tokens::{COLOR_SCHEME_STORAGE_KEY, ColorScheme, ColorSchemeSetting};

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
    /// As [`REDUCED_MOTION`], for the system colour scheme.
    static SCHEME: Cell<Option<ColorScheme>> = const { Cell::new(None) };
    static SCHEME_CALLBACKS: RefCell<Vec<(u64, SchemeCallback)>> =
        const { RefCell::new(Vec::new()) };
    static NEXT_SCHEME_CALLBACK: Cell<u64> = const { Cell::new(0) };
}

type SchemeCallback = Rc<dyn Fn(ColorScheme)>;

/// Hands `answer` the media query's result now and on every change, for the
/// app's lifetime. The script is started here: from inside `spawn_forever` it
/// never ran.
fn watch_media(query: &'static str, answer: impl Fn(bool) + 'static) {
    let script = eval(
        "const query = window.matchMedia(await dioxus.recv());
        dioxus.send(query.matches);
        query.addEventListener('change', () => dioxus.send(query.matches));
        await new Promise(() => {});",
    );
    let _ = script.send(query);
    spawn_forever(async move {
        let mut script = script;
        while let Ok(matches) = script.recv::<bool>().await {
            answer(matches);
        }
    });
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
    watch_media(crate::sx::REDUCED_MOTION, |reduced| {
        REDUCED_MOTION.set(Some(reduced))
    });
    false
}

pub(super) fn color_scheme() -> Option<&'static dyn ColorSchemeApi> {
    Runtime::try_current().map(|_| &COLOR_SCHEME as &'static dyn ColorSchemeApi)
}

/// The system scheme from the page's media query. The override is kept in a
/// file rather than `localStorage`: the provider reads it once, at mount,
/// before any script could answer.
struct WebViewColorScheme;

static COLOR_SCHEME: WebViewColorScheme = WebViewColorScheme;

/// Light until the page first answers; a dark answer then reaches every
/// [`on_change`](ColorSchemeApi::on_change) subscriber.
fn watch_scheme() {
    if SCHEME.get().is_some() {
        return;
    }
    SCHEME.set(Some(ColorScheme::Light));
    watch_media(crate::theme::DARK_SCHEME_QUERY, |dark| {
        let scheme = if dark {
            ColorScheme::Dark
        } else {
            ColorScheme::Light
        };
        if SCHEME.replace(Some(scheme)) == Some(scheme) {
            return;
        }
        let callbacks: Vec<_> = SCHEME_CALLBACKS
            .with_borrow(|callbacks| callbacks.iter().map(|(_, call)| call.clone()).collect());
        for callback in callbacks {
            callback(scheme);
        }
    });
}

/// The app's own files directory, `None` where it cannot be named. User 0's
/// path: under a secondary Android user the override lives for the session.
fn scheme_file() -> Option<PathBuf> {
    let cmdline = std::fs::read("/proc/self/cmdline").ok()?;
    // The package name, minus a `:process` suffix and the NUL padding.
    let name = cmdline.split(|byte| *byte == 0).next()?;
    let package = std::str::from_utf8(name).ok()?.split(':').next()?;
    if package.is_empty() || package.contains('/') {
        return None;
    }
    Some(PathBuf::from(format!(
        "/data/data/{package}/files/{COLOR_SCHEME_STORAGE_KEY}"
    )))
}

impl ColorSchemeApi for WebViewColorScheme {
    fn system(&self) -> ColorScheme {
        watch_scheme();
        SCHEME.get().unwrap_or(ColorScheme::Light)
    }

    fn on_change(&self, callback: Box<dyn Fn(ColorScheme)>) -> Box<dyn ColorSchemeSubscription> {
        watch_scheme();
        let id = NEXT_SCHEME_CALLBACK.replace(NEXT_SCHEME_CALLBACK.get() + 1);
        SCHEME_CALLBACKS.with_borrow_mut(|callbacks| callbacks.push((id, Rc::from(callback))));
        Box::new(WebViewColorSchemeSubscription(id))
    }

    fn stored(&self) -> Option<ColorSchemeSetting> {
        let stored = std::fs::read_to_string(scheme_file()?).ok()?;
        Some(ColorSchemeSetting::parse(stored.trim()))
    }

    fn store(&self, setting: ColorSchemeSetting) {
        let Some(path) = scheme_file() else {
            return;
        };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(path, setting.as_str());
    }
}

struct WebViewColorSchemeSubscription(u64);

impl ColorSchemeSubscription for WebViewColorSchemeSubscription {}

impl Drop for WebViewColorSchemeSubscription {
    fn drop(&mut self) {
        SCHEME_CALLBACKS.with_borrow_mut(|callbacks| callbacks.retain(|(id, _)| *id != self.0));
    }
}

pub(super) fn keyboard() -> Option<&'static dyn KeyboardApi> {
    Runtime::try_current().map(|_| &KEYBOARD as &'static dyn KeyboardApi)
}

/// Key presses at the window, in the bubble phase as on Blitz: a press a
/// handler below took (its default prevented) is not the document's, and one
/// whose propagation a handler stopped never arrives.
struct WebViewKeyboard;

static KEYBOARD: WebViewKeyboard = WebViewKeyboard;

thread_local! {
    /// The chords each subscribing scope has taken, by `Key` name and modifiers.
    static TAKEN_CHORDS: RefCell<HashMap<ScopeId, HashSet<String>>> =
        RefCell::new(HashMap::new());
}

/// The answer crosses the IPC after the press is over, so the script prevents
/// a chord once its subscription has taken it: from the second press on.
const ON_KEY: &str = "const [skipTyping, clicked, seeded] = await dioxus.recv();
    const taken = new Set(seeded);
    const typing = (target) => {
        if (!(target instanceof Element)) return false;
        if (target.tagName === 'TEXTAREA' || target.tagName === 'SELECT') return true;
        if (target.tagName === 'INPUT') {
            return !clicked.includes((target.getAttribute('type') ?? '').trim().toLowerCase());
        }
        return target.isContentEditable === true;
    };
    const onKey = (event) => {
        if (event.defaultPrevented || event.isComposing) return;
        if (skipTyping && typing(event.target)) return;
        const chord = [event.key, event.ctrlKey, event.shiftKey, event.altKey, event.metaKey];
        const name = chord.join(' ');
        if (taken.has(name)) event.preventDefault();
        dioxus.send([name, ...chord, event.repeat]);
    };
    window.addEventListener('keydown', onKey);
    for (let name; (name = await dioxus.recv()) !== null; ) taken.add(name);
    window.removeEventListener('keydown', onKey);";

impl WebViewKeyboard {
    fn listen(
        &self,
        skip_text_entry: bool,
        callback: Box<dyn Fn(KeyChord) -> bool>,
    ) -> Box<dyn KeySubscription> {
        // Per scope: a hotkey re-subscribes on every open and close, and would
        // otherwise start over without its chord each time.
        let scope = Runtime::try_current().and_then(|runtime| runtime.try_current_scope_id());
        let mut taken = TAKEN_CHORDS
            .with_borrow(|taken| scope.and_then(|scope| taken.get(&scope).cloned()))
            .unwrap_or_default();
        let script = eval(ON_KEY);
        let _ = script.send((skip_text_entry, CLICKED_INPUT_TYPES, &taken));
        let task = spawn(async move {
            let mut script = script;
            while let Ok((name, key, ctrl, shift, alt, meta, repeat)) = script
                .recv::<(String, String, bool, bool, bool, bool, bool)>()
                .await
            {
                let key = key.parse().unwrap_or(Key::Unidentified);
                let mut modifiers = Modifiers::empty();
                modifiers.set(Modifiers::CONTROL, ctrl);
                modifiers.set(Modifiers::SHIFT, shift);
                modifiers.set(Modifiers::ALT, alt);
                modifiers.set(Modifiers::META, meta);
                let chord_key = key.clone();
                if !callback(KeyChord {
                    key,
                    modifiers,
                    repeat,
                }) {
                    continue;
                }
                if skip_text_entry {
                    warn_reserved_chord(&chord_key, modifiers);
                }
                if taken.insert(name.clone()) {
                    if let Some(scope) = scope {
                        TAKEN_CHORDS.with_borrow_mut(|taken| {
                            taken.entry(scope).or_default().insert(name.clone())
                        });
                    }
                    let _ = script.send(name);
                }
            }
        });
        Box::new(WebViewKeySubscription { script, task })
    }
}

impl KeyboardApi for WebViewKeyboard {
    fn on_key(&self, callback: Box<dyn Fn(KeyChord) -> bool>) -> Box<dyn KeySubscription> {
        self.listen(true, callback)
    }

    fn on_key_unfiltered(
        &self,
        callback: Box<dyn Fn(KeyChord) -> bool>,
    ) -> Box<dyn KeySubscription> {
        self.listen(false, callback)
    }
}

struct WebViewKeySubscription {
    script: dioxus::document::Eval,
    task: Task,
}

impl KeySubscription for WebViewKeySubscription {}

impl Drop for WebViewKeySubscription {
    fn drop(&mut self) {
        self.task.cancel();
        let _ = self.script.send(());
    }
}
