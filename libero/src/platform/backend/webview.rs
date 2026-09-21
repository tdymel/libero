//! A WebView (wry: desktop, Android): Rust holds no DOM handle, so the page's
//! script runs over `eval`; elements stay on the [`mounted`](super::mounted) floor.
//!
//! A server build shares this cfg, so every accessor first asks [`runs_scripts`].

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::{Rc, Weak};

use dioxus::core::{Runtime, ScopeId, Task, spawn_forever};
use dioxus::document::{Document, eval};
use dioxus::prelude::{Key, Modifiers, spawn};

use crate::platform::a11y_media::A11yMediaSubscription;
use crate::platform::{
    A11yMediaApi, ColorSchemeApi, ColorSchemeSubscription, Dimensions, DocumentApi, ElementApi,
    KeyChord, KeySubscription, KeyboardApi, PlatformError, Read, ScrollApi, ScrollSubscription,
    clipboard::{ClipboardApi, Write},
    keyboard::{CLICKED_INPUT_TYPES, warn_reserved_chord},
};
use crate::tokens::{AccessibilityPreferences, ColorScheme, ColorSchemeSetting, Contrast};

thread_local! {
    /// [`runs_scripts`]'s answer per document: one thread may serve SSR and liveview.
    static RUNS_SCRIPTS: RefCell<Vec<(Weak<dyn Document>, bool)>> =
        const { RefCell::new(Vec::new()) };
}

/// Whether a page runs the current document's scripts: a server's or the no-op
/// document fails the send.
fn runs_scripts() -> bool {
    // Outside a scope (a drop, a task's wake-up) the root's document answers.
    let Some(document) = Runtime::try_current().and_then(|runtime| {
        let scope = runtime.try_current_scope_id().unwrap_or(ScopeId::ROOT);
        runtime.consume_context::<Rc<dyn Document>>(scope)
    }) else {
        return false;
    };
    let key = Rc::downgrade(&document);
    let known = RUNS_SCRIPTS.with_borrow(|known| {
        known
            .iter()
            .find(|(seen, _)| Weak::ptr_eq(seen, &key))
            .map(|(_, runs)| *runs)
    });
    if let Some(runs) = known {
        return runs;
    }
    let runs = document
        .eval("await dioxus.recv();".to_string())
        .send(())
        .is_ok();
    RUNS_SCRIPTS.with_borrow_mut(|known| {
        known.retain(|(seen, _)| seen.strong_count() > 0);
        known.push((key, runs));
    });
    runs
}

pub(super) fn document() -> Option<&'static dyn DocumentApi> {
    runs_scripts().then_some(&DOCUMENT as &'static dyn DocumentApi)
}

pub(super) fn scroll() -> Option<&'static dyn ScrollApi> {
    runs_scripts().then_some(&SCROLL as &'static dyn ScrollApi)
}

pub(crate) fn clipboard() -> Option<&'static dyn ClipboardApi> {
    runs_scripts().then_some(&CLIPBOARD as &'static dyn ClipboardApi)
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

/// Hands `answer` the media query's result now and on every change. The script
/// starts here: from inside `spawn_forever` it never ran.
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
    if !runs_scripts() {
        return false;
    }
    if let Some(reduced) = REDUCED_MOTION.get() {
        return reduced;
    }
    REDUCED_MOTION.set(Some(false));
    watch_media(crate::sx::REDUCED_MOTION, |reduced| {
        REDUCED_MOTION.set(Some(reduced))
    });
    false
}

pub(super) fn a11y_media() -> Option<&'static dyn A11yMediaApi> {
    runs_scripts().then_some(&A11Y_MEDIA as &'static dyn A11yMediaApi)
}

/// The page's accessibility media queries, as the web backend reads them.
struct WebViewA11yMedia;

static A11Y_MEDIA: WebViewA11yMedia = WebViewA11yMedia;

type A11yCallback = Rc<dyn Fn(AccessibilityPreferences)>;

thread_local! {
    static A11Y: Cell<Option<AccessibilityPreferences>> = const { Cell::new(None) };
    static A11Y_CALLBACKS: RefCell<Vec<(u64, A11yCallback)>> = const { RefCell::new(Vec::new()) };
    static NEXT_A11Y_CALLBACK: Cell<u64> = const { Cell::new(0) };
}

/// The defaults until the page first answers; every answer then reaches the subscribers.
fn watch_a11y() {
    if A11Y.get().is_some() {
        return;
    }
    A11Y.set(Some(AccessibilityPreferences::default()));
    let script = eval(
        "const queries = (await dioxus.recv()).map((query) => window.matchMedia(query));
        const send = () => dioxus.send(queries.map((query) => query.matches));
        send();
        queries.forEach((query) => query.addEventListener('change', send));
        await new Promise(() => {});",
    );
    let _ = script.send([
        crate::sx::REDUCED_MOTION,
        crate::sx::FORCED_COLORS,
        "(prefers-contrast: more)",
        "(prefers-contrast: less)",
        "(prefers-reduced-transparency: reduce)",
    ]);
    spawn_forever(async move {
        let mut script = script;
        while let Ok([motion, forced, more, less, transparency]) = script.recv::<[bool; 5]>().await
        {
            let preferences = AccessibilityPreferences {
                reduced_motion: motion,
                forced_colors: forced,
                contrast: match (more, less) {
                    (true, _) => Contrast::More,
                    (_, true) => Contrast::Less,
                    _ => Contrast::NoPreference,
                },
                reduced_transparency: transparency,
            };
            if A11Y.replace(Some(preferences)) == Some(preferences) {
                continue;
            }
            let callbacks: Vec<_> = A11Y_CALLBACKS
                .with_borrow(|callbacks| callbacks.iter().map(|(_, call)| call.clone()).collect());
            for callback in callbacks {
                callback(preferences);
            }
        }
    });
}

impl A11yMediaApi for WebViewA11yMedia {
    fn system(&self) -> AccessibilityPreferences {
        watch_a11y();
        A11Y.get().unwrap_or_default()
    }

    fn on_change(
        &self,
        callback: Box<dyn Fn(AccessibilityPreferences)>,
    ) -> Box<dyn A11yMediaSubscription> {
        watch_a11y();
        let id = NEXT_A11Y_CALLBACK.replace(NEXT_A11Y_CALLBACK.get() + 1);
        A11Y_CALLBACKS.with_borrow_mut(|callbacks| callbacks.push((id, Rc::from(callback))));
        Box::new(WebViewA11yMediaSubscription(id))
    }
}

struct WebViewA11yMediaSubscription(u64);

impl A11yMediaSubscription for WebViewA11yMediaSubscription {}

impl Drop for WebViewA11yMediaSubscription {
    fn drop(&mut self) {
        A11Y_CALLBACKS.with_borrow_mut(|callbacks| callbacks.retain(|(id, _)| *id != self.0));
    }
}

pub(super) fn color_scheme() -> Option<&'static dyn ColorSchemeApi> {
    runs_scripts().then_some(&COLOR_SCHEME as &'static dyn ColorSchemeApi)
}

/// The system scheme from the page's media query. The override lives in a file:
/// the provider reads it at mount, before any script could answer.
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

/// Desktop names no app directory yet, so an override is not kept.
#[cfg(not(target_os = "android"))]
fn scheme_file() -> Option<PathBuf> {
    None
}

/// The app's own files directory, `None` where it cannot be named. User 0's
/// path: under a secondary Android user the override lives for the session.
#[cfg(target_os = "android")]
fn scheme_file() -> Option<PathBuf> {
    let cmdline = std::fs::read("/proc/self/cmdline").ok()?;
    // The package name, minus a `:process` suffix and the NUL padding.
    let name = cmdline.split(|byte| *byte == 0).next()?;
    let package = std::str::from_utf8(name).ok()?.split(':').next()?;
    if package.is_empty() || package.contains('/') {
        return None;
    }
    Some(PathBuf::from(format!(
        "/data/data/{package}/files/{}",
        crate::tokens::COLOR_SCHEME_STORAGE_KEY
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
    runs_scripts().then_some(&KEYBOARD as &'static dyn KeyboardApi)
}

/// Key presses at the window, bubbling as on Blitz: a press a handler took or
/// stopped never arrives.
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

#[cfg(test)]
mod tests {
    use dioxus::document::NoOpDocument;
    use dioxus::prelude::*;

    use super::*;

    fn app() -> Element {
        rsx! {}
    }

    fn answers(dom: &VirtualDom) -> [bool; 5] {
        dom.in_scope(ScopeId::ROOT, || {
            [
                document().is_some(),
                scroll().is_some(),
                clipboard().is_some(),
                color_scheme().is_some(),
                keyboard().is_some(),
            ]
        })
    }

    #[test]
    fn plain_ssr_without_a_document_gets_nothing() {
        assert!(document().is_none());
        assert_eq!(answers(&VirtualDom::new(app)), [false; 5]);
    }

    /// Fullstack's server document evaluates through `NoOpDocument`.
    #[test]
    fn a_server_document_gets_nothing() {
        let dom = VirtualDom::new(app).with_root_context(Rc::new(NoOpDocument) as Rc<dyn Document>);
        assert_eq!(answers(&dom), [false; 5]);
        assert!(!dom.in_scope(ScopeId::ROOT, prefers_reduced_motion));
    }
}
