//! A WebView (wry: desktop, Android): Rust holds no DOM handle, so the page's
//! script runs over `eval`; elements stay on the [`mounted`](super::mounted) floor.
//!
//! A server build shares this cfg, so every accessor first asks [`runs_scripts`].

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::core::{Runtime, ScopeId, Task, spawn_forever};
use dioxus::document::{Document, Eval, NoOpDocument};
use dioxus::prelude::{Key, Modifiers, spawn};
use serde_json::{Value, json};

use crate::platform::a11y_media::{
    A11yMediaSubscription, REDUCED_MOTION_STORAGE_KEY, kept_reduced_motion, parse_reduced_motion,
};
use crate::platform::{
    A11yMediaApi, ColorSchemeApi, ColorSchemeSubscription, ContentSubscription, Dimensions,
    DocumentApi, ElementApi, KeyChord, KeySubscription, KeyboardApi, MediaQueryApi,
    MediaQuerySubscription, OBSERVE_ATTR, PRESS_MARKER_ATTR, PlatformError, PressApi,
    PressSubscription, Read, ScrollApi, ScrollSubscription,
    clipboard::{ClipboardApi, Write},
    file_dialog::{FileDialogApi, Picked, held_file},
    keyboard::{CLICKED_INPUT_TYPES, takes_arrows, takes_typing, warn_reserved_chord},
};
use crate::tokens::{
    AccessibilityPreferences, COLOR_SCHEME_STORAGE_KEY, ColorScheme, ColorSchemeSetting, Contrast,
};

/// Whether a page runs the current document's scripts: a server's or the no-op
/// document fails the send. Kept in [`PageState`]: Android resets thread-locals.
pub(super) fn runs_scripts() -> bool {
    let Some(page) = page() else {
        return false;
    };
    if let Some(runs) = page.runs_scripts.get() {
        return runs;
    }
    let Some(document) = page_document() else {
        return false;
    };
    // Alive past the send queued behind it: an ended script's query is gone, and
    // the send throws in the page. No `dioxus.recv()`: liveview's spins and froze the tab.
    let runs = document
        .eval("await new Promise((done) => setTimeout(done, 200));".to_string())
        .send(())
        .is_ok();
    page.runs_scripts.set(Some(runs));
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

pub(crate) fn file_dialog() -> Option<&'static dyn FileDialogApi> {
    runs_scripts().then_some(&FILE_DIALOG as &'static dyn FileDialogApi)
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
        eval_with(
            json!([name, value]),
            "const [name, value] = data;
            const root = document.documentElement;
            value === null ? root.removeAttribute(name) : root.setAttribute(name, value);",
        );
        runs_scripts()
    }
}

/// The current scope's document, else the one the page last saw. Desktop provides
/// it in its window's scope, above which a `spawn_forever` task runs (1027).
fn page_document() -> Option<Rc<dyn Document>> {
    let runtime = Runtime::try_current()?;
    let page = page()?;
    let scope = runtime.try_current_scope_id().unwrap_or(ScopeId::ROOT);
    match runtime.consume_context::<Rc<dyn Document>>(scope) {
        Some(document) => {
            page.document.replace(Some(Rc::downgrade(&document)));
            Some(document)
        }
        None => page.document.borrow().as_ref()?.upgrade(),
    }
}

/// Runs `script` in [`page_document`]'s page; a quiet no-op without one.
fn eval(script: &str) -> Eval {
    match page_document() {
        Some(document) => document.eval(script.to_string()),
        None => NoOpDocument.eval(script.to_string()),
    }
}

/// Runs `script` with `data` bound as `data`. Never `dioxus.recv()`: liveview's
/// spins on an empty queue and freezes the tab.
fn eval_with(data: Value, script: &str) -> Eval {
    eval(&format!("const data = {data};\n{script}"))
}

/// A running script's `window` slot, which a later eval reaches it through.
/// `stop` ends the script; dropping this calls it.
struct Slot {
    token: u64,
    document: Weak<dyn Document>,
}

static NEXT_SLOT: AtomicU64 = AtomicU64::new(0);

impl Slot {
    fn new() -> Self {
        Slot {
            token: NEXT_SLOT.fetch_add(1, Ordering::Relaxed),
            document: page_document().map_or_else(
                || Weak::<NoOpDocument>::new() as Weak<dyn Document>,
                |document| Rc::downgrade(&document),
            ),
        }
    }

    /// Ends in `await` until [`Slot`]'s drop; `extra` adds members to the slot.
    fn park(&self, extra: &str) -> String {
        format!(
            "await new Promise((stop) => {{
                (window.__lsxSlots ??= {{}})[{}] = {{ stop, {extra} }};
            }});",
            self.token
        )
    }

    /// Calls the slot's `member` with `argument`, if the script still runs.
    fn call(&self, member: &str, argument: Value) {
        if let Some(document) = self.document.upgrade() {
            document.eval(format!(
                "window.__lsxSlots?.[{}]?.{member}({argument});",
                self.token
            ));
        }
    }
}

impl Drop for Slot {
    fn drop(&mut self) {
        let Some(document) = self.document.upgrade() else {
            return;
        };
        document.eval(format!(
            "const slot = window.__lsxSlots?.[{0}];
            delete window.__lsxSlots?.[{0}];
            slot?.stop();",
            self.token
        ));
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
    window.addEventListener('scroll', tick, { capture: true, passive: true });";

impl ScrollApi for WebViewScroll {
    fn on_scroll(&self, callback: Box<dyn Fn()>) -> Box<dyn ScrollSubscription> {
        let slot = Slot::new();
        let script = eval(&format!(
            "{ON_SCROLL}
            {}
            window.removeEventListener('scroll', tick, {{ capture: true }});",
            slot.park("")
        ));
        let task = spawn(async move {
            let mut script = script;
            while script.recv::<()>().await.is_ok() {
                callback();
            }
        });
        Box::new(WebViewListener {
            task,
            _slot: Rc::new(slot),
        })
    }
}

pub(super) fn media_query() -> Option<&'static dyn MediaQueryApi> {
    runs_scripts().then_some(&MEDIA_QUERY as &'static dyn MediaQueryApi)
}

struct WebViewMediaQuery;

static MEDIA_QUERY: WebViewMediaQuery = WebViewMediaQuery;

impl MediaQueryApi for WebViewMediaQuery {
    /// The first answer crosses the IPC, so it arrives after the call returns.
    fn watch(&self, query: &str, callback: Box<dyn Fn(bool)>) -> Box<dyn MediaQuerySubscription> {
        let slot = Slot::new();
        let script = eval_with(
            json!(query),
            &format!(
                "const list = window.matchMedia(data);
                const send = () => dioxus.send(list.matches);
                send();
                list.addEventListener('change', send);
                {}
                list.removeEventListener('change', send);",
                slot.park("")
            ),
        );
        let task = spawn(async move {
            let mut script = script;
            while let Ok(matches) = script.recv::<bool>().await {
                callback(matches);
            }
        });
        Box::new(WebViewListener {
            task,
            _slot: Rc::new(slot),
        })
    }
}

/// An `IntersectionObserver` on the element carrying the tag, started once it
/// renders. Sends `[isIntersecting, ratio]` of the newest entry; a root without
/// a tag (or one that never renders) falls back to the viewport.
const ON_INTERSECTION: &str = "const [attr, tag, rootTag, margin, thresholds] = data;
    const find = (value) => document.querySelector('[' + attr + '=\"' + value + '\"]');
    let observer = null;
    const start = () => {
        const target = find(tag);
        if (!target) return false;
        const root = rootTag === null ? null : find(rootTag);
        if (rootTag !== null && !root) console.warn('use_intersection: the root carries no attribute');
        observer = new IntersectionObserver((entries) => {
            const latest = entries[entries.length - 1];
            dioxus.send([latest.isIntersecting, latest.intersectionRatio]);
        }, { root, rootMargin: margin, threshold: thresholds });
        observer.observe(target);
        return true;
    };
    const waiting = start() ? null : new MutationObserver(() => {
        if (start()) waiting.disconnect();
    });
    waiting?.observe(document.documentElement, { childList: true, subtree: true, attributes: true, attributeFilter: [attr] });";

/// Observed in the page, keyed by the `tag` attribute the element carries: it
/// clips by every scroller up to the root and sees layout-only changes.
pub(super) fn on_intersection(
    tags: (u64, Option<u64>),
    root_margin: &str,
    thresholds: &[f64],
    callback: Box<dyn Fn(bool, f64)>,
) -> Option<Box<dyn ContentSubscription>> {
    if !runs_scripts() {
        return None;
    }
    let slot = Slot::new();
    let (tag, root_tag) = tags;
    let script = eval_with(
        json!([
            OBSERVE_ATTR,
            tag.to_string(),
            root_tag.map(|tag| tag.to_string()),
            root_margin,
            thresholds
        ]),
        &format!(
            "{ON_INTERSECTION}
            {}
            observer?.disconnect();
            waiting?.disconnect();",
            slot.park("")
        ),
    );
    let task = spawn(async move {
        let mut script = script;
        while let Ok((is_intersecting, ratio)) = script.recv::<(bool, f64)>().await {
            callback(is_intersecting, ratio);
        }
    });
    Some(Box::new(WebViewListener {
        task,
        _slot: Rc::new(slot),
    }))
}

impl ContentSubscription for WebViewListener {}

pub(super) fn press() -> Option<&'static dyn PressApi> {
    runs_scripts().then_some(&PRESS as &'static dyn PressApi)
}

struct WebViewPress;

static PRESS: WebViewPress = WebViewPress;

/// Capture phase, so no handler that stops the press hides it. Sends every
/// marker from the target up to the root.
const ON_PRESS: &str = "const onPress = (event) => {
        const markers = [];
        for (let el = event.target instanceof Element ? event.target : null; el; el = el.parentElement) {
            const value = el.getAttribute(data);
            if (value) markers.push(...value.split(' ').map(Number));
        }
        dioxus.send(markers);
    };
    window.addEventListener('pointerdown', onPress, { capture: true });";

impl PressApi for WebViewPress {
    fn on_press(&self, callback: Box<dyn Fn(Vec<u64>)>) -> Box<dyn PressSubscription> {
        let slot = Slot::new();
        let script = eval_with(
            json!(PRESS_MARKER_ATTR),
            &format!(
                "{ON_PRESS}
                {}
                window.removeEventListener('pointerdown', onPress, {{ capture: true }});",
                slot.park("")
            ),
        );
        let task = spawn(async move {
            let mut script = script;
            while let Ok(markers) = script.recv::<Vec<u64>>().await {
                callback(markers);
            }
        });
        Box::new(WebViewListener {
            task,
            _slot: Rc::new(slot),
        })
    }
}

/// A listening script's task and slot: dropping it stops both.
struct WebViewListener {
    task: Task,
    _slot: Rc<Slot>,
}

impl ScrollSubscription for WebViewListener {}

impl KeySubscription for WebViewListener {}

impl PressSubscription for WebViewListener {}

impl MediaQuerySubscription for WebViewListener {}

impl Drop for WebViewListener {
    fn drop(&mut self) {
        self.task.cancel();
    }
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

struct WebViewFileDialog;

static FILE_DIALOG: WebViewFileDialog = WebViewFileDialog;

/// A detached input: dioxus hands a click on one of its own inputs to the host,
/// which picks nothing on Android. Resolves to `[name, type, lastModified, base64]`s.
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
    return await Promise.all(files.map(async (file) =>
        [file.name, file.type, file.lastModified, encode(new Uint8Array(await file.arrayBuffer()))]));";

impl FileDialogApi for WebViewFileDialog {
    fn open(&self, accept: &str, multiple: bool, capture: Option<&str>) -> Picked {
        let script = eval_with(json!([accept, multiple, capture]), PICK_FILES);
        Box::pin(async move {
            let picked = script
                .join::<Vec<(String, String, f64, String)>>()
                .await
                .unwrap_or_default();
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

/// Standard, padded base64, as `btoa` writes it; `None` on anything else.
fn decode_base64(text: &str) -> Option<Vec<u8>> {
    let text = text.trim_end_matches('=').as_bytes();
    let mut bytes = Vec::with_capacity(text.len() * 3 / 4);
    let (mut buffer, mut bits) = (0u32, 0);
    for &ch in text {
        let value = match ch {
            b'A'..=b'Z' => ch - b'A',
            b'a'..=b'z' => ch - b'a' + 26,
            b'0'..=b'9' => ch - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        };
        buffer = (buffer << 6) | u32::from(value);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            bytes.push((buffer >> bits) as u8);
        }
    }
    Some(bytes)
}

type SchemeCallback = Rc<dyn Fn(ColorScheme)>;
type A11yCallback = Rc<dyn Fn(AccessibilityPreferences)>;

/// One page's answers, kept on its dom's root: liveview runs many sessions on
/// one thread. Each watcher is a task of that dom.
#[derive(Default)]
struct PageState {
    /// [`runs_scripts`]'s answer, `None` until probed.
    runs_scripts: Cell<Option<bool>>,
    /// The document a scope last found, for code running above it.
    document: RefCell<Option<Weak<dyn Document>>>,
    /// The media query's last answer; `None` until a listener is running.
    reduced_motion: Cell<Option<bool>>,
    scheme: Cell<Option<ColorScheme>>,
    scheme_callbacks: RefCell<Vec<(u64, SchemeCallback)>>,
    a11y: Cell<Option<AccessibilityPreferences>>,
    a11y_callbacks: RefCell<Vec<(u64, A11yCallback)>>,
    next_callback: Cell<u64>,
    /// The chords each subscribing scope has taken, by `Key` name and modifiers.
    taken_chords: RefCell<HashMap<ScopeId, HashSet<String>>>,
    /// The focused element, mirrored by [`watch_focus`] once `watching_focus`.
    focused: RefCell<Focused>,
    watching_focus: Cell<bool>,
    /// Whether [`guard_typed_values`] ran on this page.
    guarding_values: Cell<bool>,
}

/// The focused element as the page last reported it: tag, `type`, editable, RTL.
type Focused = Option<(String, Option<String>, bool, bool)>;

impl PageState {
    fn next_id(&self) -> u64 {
        self.next_callback.replace(self.next_callback.get() + 1)
    }
}

/// The current dom's [`PageState`], `None` outside a runtime.
fn page() -> Option<Rc<PageState>> {
    let runtime = Runtime::try_current()?;
    Some(
        runtime
            .consume_context::<Rc<PageState>>(ScopeId::ROOT)
            .unwrap_or_else(|| runtime.provide_context(ScopeId::ROOT, Rc::default())),
    )
}

/// Hands `answer` the media query's result now and on every change. The script
/// starts here: from inside `spawn_forever` it never ran.
fn watch_media(query: &'static str, answer: impl Fn(bool) + 'static) {
    let script = eval_with(
        json!(query),
        "const query = window.matchMedia(data);
        dioxus.send(query.matches);
        query.addEventListener('change', () => dioxus.send(query.matches));
        await new Promise(() => {});",
    );
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
    let Some(page) = page() else {
        return false;
    };
    if let Some(reduced) = page.reduced_motion.get() {
        return reduced;
    }
    page.reduced_motion.set(Some(false));
    watch_media(crate::sx::REDUCED_MOTION, move |reduced| {
        page.reduced_motion.set(Some(reduced))
    });
    false
}

pub(super) fn a11y_media() -> Option<&'static dyn A11yMediaApi> {
    runs_scripts().then_some(&A11Y_MEDIA as &'static dyn A11yMediaApi)
}

/// The page's accessibility media queries, as the web backend reads them.
struct WebViewA11yMedia;

static A11Y_MEDIA: WebViewA11yMedia = WebViewA11yMedia;

/// The defaults until the page first answers; every answer then reaches the subscribers.
fn watch_a11y(page: &Rc<PageState>) {
    if page.a11y.get().is_some() {
        return;
    }
    page.a11y.set(Some(AccessibilityPreferences::default()));
    let page = page.clone();
    let script = eval_with(
        json!([
            crate::sx::REDUCED_MOTION,
            crate::sx::FORCED_COLORS,
            "(prefers-contrast: more)",
            "(prefers-contrast: less)",
            "(prefers-reduced-transparency: reduce)",
        ]),
        "const queries = data.map((query) => window.matchMedia(query));
        const send = () => dioxus.send(queries.map((query) => query.matches));
        send();
        queries.forEach((query) => query.addEventListener('change', send));
        await new Promise(() => {});",
    );
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
            if page.a11y.replace(Some(preferences)) == Some(preferences) {
                continue;
            }
            let callbacks: Vec<_> = (page.a11y_callbacks.borrow().iter())
                .map(|(_, call)| call.clone())
                .collect();
            for callback in callbacks {
                callback(preferences);
            }
        }
    });
}

impl A11yMediaApi for WebViewA11yMedia {
    fn system(&self) -> AccessibilityPreferences {
        let Some(page) = page() else {
            return AccessibilityPreferences::default();
        };
        watch_a11y(&page);
        page.a11y.get().unwrap_or_default()
    }

    fn on_change(
        &self,
        callback: Box<dyn Fn(AccessibilityPreferences)>,
    ) -> Box<dyn A11yMediaSubscription> {
        let Some(page) = page() else {
            return Box::new(WebViewSubscription(Weak::new(), 0));
        };
        watch_a11y(&page);
        let id = page.next_id();
        page.a11y_callbacks
            .borrow_mut()
            .push((id, in_subscriber(callback)));
        Box::new(WebViewSubscription(Rc::downgrade(&page), id))
    }

    fn stored_reduced_motion(&self) -> Option<bool> {
        parse_reduced_motion(&std::fs::read_to_string(app_file(REDUCED_MOTION_STORAGE_KEY)?).ok()?)
    }

    fn store_reduced_motion(&self, reduced: Option<bool>) {
        let Some(path) = app_file(REDUCED_MOTION_STORAGE_KEY) else {
            return;
        };
        let _ = match reduced {
            Some(reduced) => write_app_file(&path, kept_reduced_motion(reduced)),
            None => std::fs::remove_file(path),
        };
    }
}

/// Runs `callback` in the subscribing scope: a watcher's task runs at the root,
/// which owns none of the subscriber's signals.
fn in_subscriber<T: 'static>(callback: Box<dyn Fn(T)>) -> Rc<dyn Fn(T)> {
    let scope = Runtime::try_current().and_then(|runtime| runtime.try_current_scope_id());
    Rc::new(move |value| match (scope, Runtime::try_current()) {
        (Some(scope), Some(runtime)) => runtime.in_scope(scope, || callback(value)),
        _ => callback(value),
    })
}

/// A media callback's page and id; dropping it removes the callback.
struct WebViewSubscription(Weak<PageState>, u64);

impl A11yMediaSubscription for WebViewSubscription {}

impl ColorSchemeSubscription for WebViewSubscription {}

impl Drop for WebViewSubscription {
    fn drop(&mut self) {
        let Some(page) = self.0.upgrade() else {
            return;
        };
        page.a11y_callbacks
            .borrow_mut()
            .retain(|(id, _)| *id != self.1);
        page.scheme_callbacks
            .borrow_mut()
            .retain(|(id, _)| *id != self.1);
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
fn watch_scheme(page: &Rc<PageState>) {
    if page.scheme.get().is_some() {
        return;
    }
    page.scheme.set(Some(ColorScheme::Light));
    let page = page.clone();
    watch_media(crate::theme::DARK_SCHEME_QUERY, move |dark| {
        let scheme = if dark {
            ColorScheme::Dark
        } else {
            ColorScheme::Light
        };
        if page.scheme.replace(Some(scheme)) == Some(scheme) {
            return;
        }
        let callbacks: Vec<_> = (page.scheme_callbacks.borrow().iter())
            .map(|(_, call)| call.clone())
            .collect();
        for callback in callbacks {
            callback(scheme);
        }
    });
}

fn write_app_file(path: &std::path::Path, text: &str) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, text)
}

/// Desktop names no app directory yet, so an override is not kept.
#[cfg(not(target_os = "android"))]
fn app_file(_name: &str) -> Option<PathBuf> {
    None
}

/// `name` in the app's own files directory, `None` where it cannot be named. User 0's
/// path: under a secondary Android user the override lives for the session.
#[cfg(target_os = "android")]
fn app_file(name: &str) -> Option<PathBuf> {
    let cmdline = std::fs::read("/proc/self/cmdline").ok()?;
    // The package name, minus a `:process` suffix and the NUL padding.
    let process = cmdline.split(|byte| *byte == 0).next()?;
    let package = std::str::from_utf8(process).ok()?.split(':').next()?;
    if package.is_empty() || package.contains('/') {
        return None;
    }
    Some(PathBuf::from(format!("/data/data/{package}/files/{name}")))
}

impl ColorSchemeApi for WebViewColorScheme {
    fn system(&self) -> ColorScheme {
        let Some(page) = page() else {
            return ColorScheme::Light;
        };
        watch_scheme(&page);
        page.scheme.get().unwrap_or(ColorScheme::Light)
    }

    fn on_change(&self, callback: Box<dyn Fn(ColorScheme)>) -> Box<dyn ColorSchemeSubscription> {
        let Some(page) = page() else {
            return Box::new(WebViewSubscription(Weak::new(), 0));
        };
        watch_scheme(&page);
        let id = page.next_id();
        page.scheme_callbacks
            .borrow_mut()
            .push((id, in_subscriber(callback)));
        Box::new(WebViewSubscription(Rc::downgrade(&page), id))
    }

    fn stored(&self) -> Option<ColorSchemeSetting> {
        let stored = std::fs::read_to_string(app_file(COLOR_SCHEME_STORAGE_KEY)?).ok()?;
        Some(ColorSchemeSetting::parse(stored.trim()))
    }

    fn store(&self, setting: ColorSchemeSetting) {
        if let Some(path) = app_file(COLOR_SCHEME_STORAGE_KEY) {
            let _ = write_app_file(&path, setting.as_str());
        }
    }
}

pub(super) fn keyboard() -> Option<&'static dyn KeyboardApi> {
    runs_scripts().then_some(&KEYBOARD as &'static dyn KeyboardApi)
}

/// Key presses at the window, bubbling as on Blitz: a press a handler took or
/// stopped never arrives.
struct WebViewKeyboard;

static KEYBOARD: WebViewKeyboard = WebViewKeyboard;

/// The answer crosses the IPC after the press is over, so the script prevents
/// a chord once its subscription has taken it: from the second press on.
const ON_KEY: &str = "const [skipTyping, clicked, seeded] = data;
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
        const entry = typing(event.target);
        if (skipTyping && entry) return;
        const chord = [event.key, event.ctrlKey, event.shiftKey, event.altKey, event.metaKey];
        // Per target kind: a chord taken outside text entry stays the field's inside it.
        const name = chord.join(' ') + (entry ? ' typing' : '');
        if (taken.has(name)) event.preventDefault();
        dioxus.send([name, ...chord, event.repeat, entry]);
    };
    window.addEventListener('keydown', onKey);";

impl WebViewKeyboard {
    fn listen(
        &self,
        skip_text_entry: bool,
        callback: Box<dyn Fn(KeyChord) -> bool>,
    ) -> Box<dyn KeySubscription> {
        // Per scope: a hotkey re-subscribes on every open and close, and would
        // otherwise start over without its chord each time.
        let scope = Runtime::try_current().and_then(|runtime| runtime.try_current_scope_id());
        let page = page();
        let mut taken = (page.as_ref().zip(scope))
            .and_then(|(page, scope)| page.taken_chords.borrow().get(&scope).cloned())
            .unwrap_or_default();
        let slot = Rc::new(Slot::new());
        let script = eval_with(
            json!([skip_text_entry, CLICKED_INPUT_TYPES, &taken]),
            &format!(
                "{ON_KEY}
                {}
                window.removeEventListener('keydown', onKey);",
                slot.park("take: (name) => taken.add(name)")
            ),
        );
        let task_slot = slot.clone();
        let task = spawn(async move {
            let mut script = script;
            while let Ok((name, key, ctrl, shift, alt, meta, repeat, text_entry)) = script
                .recv::<(String, String, bool, bool, bool, bool, bool, bool)>()
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
                    text_entry,
                }) {
                    continue;
                }
                if skip_text_entry {
                    warn_reserved_chord(&chord_key, modifiers);
                }
                if taken.insert(name.clone()) {
                    if let Some((page, scope)) = page.as_ref().zip(scope) {
                        (page.taken_chords.borrow_mut())
                            .entry(scope)
                            .or_default()
                            .insert(name.clone());
                    }
                    task_slot.call("take", json!(name));
                }
            }
        });
        Box::new(WebViewListener { task, _slot: slot })
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

/// A focus move with no `relatedTarget` left for no element.
const ON_FOCUS: &str =
    "const send = (el) => dioxus.send(el instanceof Element && el !== document.body
        ? [el.tagName, el.getAttribute('type'), el.isContentEditable === true,
           getComputedStyle(el).direction === 'rtl']
        : null);
    document.addEventListener('focusin', (event) => send(event.target));
    document.addEventListener('focusout', (event) => { if (!event.relatedTarget) send(null); });
    send(document.activeElement);
    await new Promise(() => {});";

/// Starts mirroring the page's focus into Rust, once per dom. A key press goes
/// to the focused element, so its handler reads the mirror, as on Blitz.
pub(super) fn watch_focus() {
    focus_page();
}

fn focus_page() -> Option<Rc<PageState>> {
    if !runs_scripts() {
        return None;
    }
    let page = page()?;
    if !page.watching_focus.replace(true) {
        let script = eval(ON_FOCUS);
        let mirror = page.clone();
        spawn_forever(async move {
            let mut script = script;
            while let Ok(next) = script.recv::<Focused>().await {
                mirror.focused.replace(next);
            }
        });
    }
    Some(page)
}

/// Wraps the interpreter's attribute write. Each `input` records the text it sent;
/// a `value` write matching an older record is the render of a stale keystroke and
/// is dropped. Any other value is the app's own and lands. Records expire after 2 s.
const GUARD_TYPED: &str = "const interpreter = window.interpreter;
    if (!interpreter || interpreter.lsxTyped) return;
    const typed = interpreter.lsxTyped = new WeakMap();
    const live = (node) => {
        const now = performance.now();
        return (typed.get(node) ?? []).filter(([, at]) => now - at < 2000);
    };
    window.addEventListener('input', (event) => {
        const node = event.target;
        if (!(node instanceof HTMLInputElement || node instanceof HTMLTextAreaElement)) return;
        typed.set(node, [...live(node), [node.value, performance.now()]]);
    }, { capture: true });
    const write = interpreter.setAttributeInner.bind(interpreter);
    interpreter.setAttributeInner = (node, field, value, ns) => {
        if (field === 'value' && !ns && typed.has(node)) {
            const records = live(node);
            const index = records.findIndex(([text]) => text === value);
            if (index !== -1 && records.splice(0, index + 1) && records.length > 0) {
                typed.set(node, records);
                return;
            }
            typed.delete(node);
        }
        write(node, field, value, ns);
    };";

/// Keeps a controlled field's older renders from overwriting faster typing: an
/// `input` crosses the IPC and its render comes back after the next keystroke (1026).
pub(super) fn guard_typed_values() {
    if !runs_scripts() {
        return;
    }
    if let Some(page) = page()
        && !page.guarding_values.replace(true)
    {
        eval(GUARD_TYPED);
    }
}

/// Queued like [`WebViewDocument::set_root_attribute`], so an edit dioxus sends
/// later still lands over it.
pub(super) fn set_value_by_id(id: &str, value: &str) -> Result<(), PlatformError> {
    if !runs_scripts() {
        return Err(PlatformError::Unsupported);
    }
    eval_with(
        json!([id, value]),
        "const [id, value] = data;
        const el = document.getElementById(id);
        if (el && el.value !== value) el.value = value;",
    );
    Ok(())
}

fn focused() -> Focused {
    focus_page().and_then(|page| page.focused.borrow().clone())
}

pub(super) fn typing_target() -> bool {
    focused()
        .is_some_and(|(tag, kind, editable, _)| editable || takes_typing(&tag, kind.as_deref()))
}

pub(super) fn arrow_target() -> bool {
    focused().is_some_and(|(tag, kind, ..)| takes_arrows(&tag, kind.as_deref()))
}

pub(super) fn rtl_target() -> bool {
    focused().is_some_and(|(.., rtl)| rtl)
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::pin;
    use std::task::{Context, Poll, Waker};

    use dioxus::document::{Eval, EvalError, Evaluator, NoOpDocument};
    use dioxus::prelude::*;
    use dioxus::signals::{AnyStorage, Owner, UnsyncStorage};
    use serde_json::Value;

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
    fn a_page_observes_by_tag_and_a_server_does_not() {
        let observes =
            |dom: &VirtualDom| dom.in_scope(ScopeId::ROOT, crate::platform::observes_by_tag);
        assert!(observes(&page_dom(false)));
        let server =
            VirtualDom::new(app).with_root_context(Rc::new(NoOpDocument) as Rc<dyn Document>);
        assert!(!observes(&server));
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
        assert!(dom.in_scope(ScopeId::ROOT, focus_page).is_none());
        assert!(!dom.in_scope(ScopeId::ROOT, typing_target));
    }

    /// A page whose every script answers `dark` once.
    struct FakePage {
        dark: bool,
        owners: RefCell<Vec<Owner<UnsyncStorage>>>,
    }

    struct Answer(Option<bool>);

    impl Evaluator for Answer {
        fn poll_join(&mut self, _: &mut Context<'_>) -> Poll<Result<Value, EvalError>> {
            Poll::Pending
        }

        fn poll_recv(&mut self, _: &mut Context<'_>) -> Poll<Result<Value, EvalError>> {
            match self.0.take() {
                Some(dark) => Poll::Ready(Ok(Value::Bool(dark))),
                None => Poll::Pending,
            }
        }

        fn send(&self, _: Value) -> Result<(), EvalError> {
            Ok(())
        }
    }

    impl Document for FakePage {
        fn eval(&self, _: String) -> Eval {
            let owner = UnsyncStorage::owner();
            let answer = owner.insert(Box::new(Answer(Some(self.dark))) as Box<dyn Evaluator>);
            self.owners.borrow_mut().push(owner);
            Eval::new(answer)
        }
    }

    fn page_dom(dark: bool) -> VirtualDom {
        let page = FakePage {
            dark,
            owners: RefCell::default(),
        };
        let mut dom = VirtualDom::new(app).with_root_context(Rc::new(page) as Rc<dyn Document>);
        dom.rebuild_in_place();
        dom
    }

    fn settle(dom: &mut VirtualDom) {
        let mut cx = Context::from_waker(Waker::noop());
        for _ in 0..4 {
            let _ = pin!(dom.wait_for_work()).poll(&mut cx);
        }
    }

    /// Desktop provides its document in the window's scope; a `spawn_forever`
    /// task runs at the root and must still reach it (1027).
    #[test]
    fn the_root_reaches_a_document_provided_below_it() {
        #[component]
        fn Window() -> Element {
            use_hook(|| {
                let page = FakePage {
                    dark: true,
                    owners: RefCell::default(),
                };
                provide_context(Rc::new(page) as Rc<dyn Document>);
                assert!(runs_scripts());
            });
            rsx! {}
        }
        fn windowed() -> Element {
            rsx! { Window {} }
        }
        let mut dom = VirtualDom::new(windowed);
        assert!(dom.in_scope(ScopeId::ROOT, page_document).is_none());
        dom.rebuild_in_place();
        assert!(dom.in_scope(ScopeId::ROOT, page_document).is_some());
    }

    /// Liveview sessions and desktop windows share a thread: one's answer must not reach another.
    #[test]
    fn two_pages_on_one_thread_keep_their_own_scheme() {
        let mut dark = page_dom(true);
        let mut light = page_dom(false);
        let heard = Rc::new(Cell::new(0));
        let system =
            |dom: &VirtualDom| dom.in_scope(ScopeId::ROOT, || color_scheme().unwrap().system());
        let _subscription = light.in_scope(ScopeId::ROOT, || {
            let heard = heard.clone();
            color_scheme()
                .unwrap()
                .on_change(Box::new(move |_| heard.set(heard.get() + 1)))
        });
        system(&dark);
        settle(&mut dark);
        settle(&mut light);
        assert_eq!(system(&dark), ColorScheme::Dark);
        assert_eq!(system(&light), ColorScheme::Light);
        assert_eq!(
            heard.get(),
            0,
            "the dark page's answer reached the light one"
        );
    }

    #[test]
    fn base64_decodes_as_btoa_wrote_it() {
        assert_eq!(decode_base64("").unwrap(), b"");
        assert_eq!(decode_base64("YQ==").unwrap(), b"a");
        assert_eq!(decode_base64("YWI=").unwrap(), b"ab");
        assert_eq!(decode_base64("YWJj").unwrap(), b"abc");
        assert_eq!(decode_base64("AP+A/w==").unwrap(), [0, 255, 128, 255]);
        assert_eq!(decode_base64("YW\nJj"), None);
    }

    #[test]
    fn a_held_file_reads_from_memory() {
        let file = held_file("a.txt".into(), "text/plain".into(), 7, b"hello".to_vec());
        assert_eq!(file.name(), "a.txt");
        assert_eq!(file.size(), 5);
        assert_eq!(file.last_modified(), 7);
        assert_eq!(file.content_type().as_deref(), Some("text/plain"));
        let mut cx = Context::from_waker(Waker::noop());
        let Poll::Ready(text) = pin!(file.read_string()).poll(&mut cx) else {
            panic!("a held file's read waits on nothing");
        };
        assert_eq!(text.unwrap(), "hello");
        let untyped = held_file("a".into(), String::new(), 0, Vec::new());
        assert_eq!(untyped.content_type(), None);
    }
}
