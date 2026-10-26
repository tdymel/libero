//! A WebView (wry: desktop, Android): Rust holds no DOM handle, so the page's
//! script runs over `eval`; elements stay on the [`mounted`](super::mounted) floor.
//!
//! A server build shares this cfg, so every accessor first asks [`runs_scripts`].

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::core::{Runtime, ScopeId, Task};
use dioxus::document::{Document, Eval, NoOpDocument};
use serde_json::{Value, json};

use crate::platform::{Dimensions, DocumentApi, ElementApi, PlatformError, Read};
use crate::tokens::{AccessibilityPreferences, ColorScheme};

mod appearance;
#[cfg(target_os = "android")]
mod back;
mod capture;
mod clipboard;
mod direction;
mod file_dialog;
mod focus;
mod geolocation;
mod guards;
mod keyboard;
mod media;
mod media_query;
mod observe;
mod permission;
mod press;
#[cfg(any(target_os = "android", feature = "charts"))]
mod raster;
#[cfg(all(
    not(target_os = "android"),
    not(feature = "native"),
    not(feature = "desktop")
))]
mod save_file;
mod scroll;
mod selected;
#[cfg(not(target_os = "android"))]
mod share;
#[cfg(not(target_os = "android"))]
mod system_notification;
#[cfg(test)]
mod tests;
mod waveform;

pub(super) use appearance::{a11y_media, color_scheme, prefers_reduced_motion};
#[cfg(target_os = "android")]
pub(super) use back::{back_entries, watch_back};
pub(crate) use capture::capture;
pub(crate) use clipboard::clipboard;
pub(super) use direction::{
    alias_rtl_tag, forget_rtl_tag, is_rtl_by_tag, read_rtl_tag, watch_direction,
};
pub(crate) use file_dialog::{file_dialog, image_crop};
#[cfg(test)]
use focus::focus_page;
pub(super) use focus::{
    arrow_target, caret_edges, click_selector, focus_among, focus_closest_of_focused,
    focus_first_of, focus_kept, focus_lands_in, focus_selector, focus_tab_from, focus_tab_within,
    focused_attribute, focused_tag, keep_focused, rtl_target, typing_target, watch_focus,
};
pub(crate) use geolocation::geolocation;
pub(super) use guards::{
    follow_pointer, guard_defaults, guard_typed_values, on_pointer_release, reset_selector,
    set_value_by_id,
};
pub(super) use keyboard::{claim_escape, keyboard};
pub(crate) use media::{fullscreen, media};
pub(super) use media_query::media_query;
pub(super) use observe::{
    command_by_tag, computed_px, has_match, observe_content, observe_resize, on_intersection,
};
pub(crate) use permission::permission;
pub(super) use press::press;
#[cfg(target_os = "android")]
pub(crate) use raster::square_png;
#[cfg(feature = "charts")]
pub(crate) use raster::svg_png;
#[cfg(all(
    not(target_os = "android"),
    not(feature = "native"),
    not(feature = "desktop")
))]
pub(crate) use save_file::save_file;
pub(super) use scroll::{
    hold_edge_pan, on_element_scroll, on_viewport_resize, scroll, visible_band,
};
pub(crate) use selected::mounted_by_selector;
pub(super) use selected::selected_natural_size;
#[cfg(not(target_os = "android"))]
pub(crate) use share::share;
#[cfg(not(target_os = "android"))]
pub(crate) use system_notification::system_notification;
#[cfg(all(target_os = "linux", feature = "desktop"))]
pub(crate) use system_notification::wry_page;
pub(crate) use waveform::waveform;

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
    if runs {
        eval(GUARD_LISTENERS);
    }
    runs
}

/// dioxus c607e4e `interpreter/src/ts/core.ts:161` drops `local[id]` on a first removal; a second
/// threw and the page stopped applying edits (2702). Drop when upstream fixes it.
const GUARD_LISTENERS: &str = "const interpreter = window.interpreter;
    if (!interpreter || interpreter.lsxListeners) return;
    interpreter.lsxListeners = true;
    const remove = interpreter.removeNonBubblingListener.bind(interpreter);
    interpreter.removeNonBubblingListener = (element, name) => {
        const id = element.getAttribute('data-dioxus-id');
        if (interpreter.local[id]) return remove(element, name);
        element.removeEventListener(name, interpreter.handler);
    };";

pub(super) fn document() -> Option<&'static dyn DocumentApi> {
    runs_scripts().then_some(&DOCUMENT as &'static dyn DocumentApi)
}

struct WebViewDocument;

static DOCUMENT: WebViewDocument = WebViewDocument;

impl DocumentApi for WebViewDocument {
    /// No element handle can name the answer (todo 5).
    fn active_element(&self) -> Option<Box<dyn ElementApi>> {
        None
    }

    /// The height stops at the visual viewport's bottom, as on the web.
    fn viewport(&self) -> Read<Dimensions> {
        let read = eval(
            "const visual = window.visualViewport;
            const bottom = visual ? visual.offsetTop + visual.height : Infinity;
            return [window.innerWidth, Math.min(window.innerHeight, bottom)];",
        );
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
    let script = format!(
        "if (typeof dioxus !== 'undefined') {{
            (window.__liberoHeld ??= new Set()).add(dioxus);
            setTimeout(() => __liberoHeld.delete(dioxus), {CHANNEL_HOLD_MS});
        }}
        {script}"
    );
    match page_document() {
        Some(document) => document.eval(script),
        None => NoOpDocument.eval(script),
    }
}

/// dioxus-desktop drops an eval's Rust half once the page GCs its `dioxus` channel,
/// result read or not (`Finished`): held past the read, see `codebase/platform/platform-api`.
const CHANNEL_HOLD_MS: u32 = 5000;

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
            document.eval(self.call_script(member, &argument));
        }
    }

    /// The argument is bound as `data`, never spliced into the call.
    fn call_script(&self, member: &str, argument: &Value) -> String {
        format!(
            "const data = {argument};\nwindow.__lsxSlots?.[{}]?.{member}(data);",
            self.token
        )
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

/// Answers once the page finished its current task: an eval only runs after it, and an
/// event that task fired (a `focusin` after a `focusout`) reached Rust first (todo 2664).
pub(crate) async fn next_task() {
    if runs_scripts() {
        let _ = eval("return 0;").join::<u8>().await;
    }
}

/// Answers `true` once the page's web fonts have loaded.
pub(crate) async fn fonts_ready() -> bool {
    runs_scripts()
        && eval("await document.fonts.ready; return 0;")
            .join::<u8>()
            .await
            .is_ok()
}

/// A listening script's task and slot: dropping it stops both.
struct WebViewListener {
    task: Task,
    _slot: Rc<Slot>,
}

impl Drop for WebViewListener {
    fn drop(&mut self) {
        self.task.cancel();
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
    /// Whether [`guard_defaults`] ran on this page.
    guarding_defaults: Cell<bool>,
    /// Each tagged element's computed `direction` as RTL, mirrored by [`watch_direction`].
    rtl: RefCell<HashMap<u64, bool>>,
    /// A following handle's tag to the tag of the element it follows.
    rtl_aliases: RefCell<HashMap<u64, u64>>,
    watching_direction: Cell<bool>,
}

/// The focused element as the page last reported it: tag, `type`, editable, RTL,
/// and an input's caret at its start and its end.
type Focused = Option<(String, Option<String>, bool, bool, Option<(bool, bool)>)>;

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
