//! A headless Blitz harness: one libero component in a windowless
//! `DioxusDocument`, driven through Blitz's own event pipeline.
//!
//! ```no_run
//! use dioxus::prelude::*;
//! use e2e::native::mount;
//!
//! fn app() -> Element {
//!     rsx! { button { id: "go", "Go" } }
//! }
//!
//! let mut page = mount(app);
//! page.click("#go");
//! assert!(page.is_focused("#go"));
//! ```

use std::{
    cell::{Cell, RefCell},
    path::Path,
    rc::{Rc, Weak},
    sync::{
        Arc, Once, PoisonError, RwLock, RwLockReadGuard, TryLockError,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use anyrender::{
    Paint, Scene,
    recording::{GlyphRunCommand, RenderCommand},
};
use anyrender_vello_cpu::VelloCpuImageRenderer;
use blitz_dom::{
    BaseDocument, Document, DocumentConfig, Node,
    node::{ImageData, SpecialElementData},
};
use blitz_traits::{
    NodeId,
    events::{
        BlitzKeyEvent, BlitzPointerEvent, BlitzPointerId, BlitzWheelDelta, BlitzWheelEvent,
        KeyState, MouseEventButton, MouseEventButtons, Point, PointerCoords, PointerDetails,
        UiEvent,
    },
    navigation::{NavigationOptions, NavigationProvider},
    net::{Bytes, NetHandler, NetProvider, Request},
    shell::{ShellProvider, Viewport},
};
use dioxus::prelude::*;
use dioxus_native_dom::DioxusDocument;
use libero::LiberoProvider;
use style::properties::{PropertyDeclarationId, PropertyId, ShorthandId};
use warnings::SignalWarnings;

use crate::frames::FrameStats;

mod warnings;

pub use blitz_traits::shell::ColorScheme;
pub use dioxus::prelude::{Code, Key, Location, Modifiers};

/// The viewport every test renders into, CSS pixels at scale 1.
pub const VIEWPORT: (u32, u32) = (1024, 768);

/// [`Page::time_raster_steps`] in ms per step, the warm-up steps dropped.
#[derive(Debug, Clone, Copy)]
pub struct RasterTimes {
    /// The wheel event, its re-render and Blitz's resolve.
    pub update: FrameStats,
    /// Painting and rasterising the viewport on the CPU.
    pub raster: FrameStats,
}

// Bounds `settle`, so a render loop fails the test instead of hanging it.
const MAX_POLLS: usize = 200;

/// How long [`Page::wait_for`] waits for its condition.
pub const WAIT_LIMIT: Duration = Duration::from_secs(5);

// HTML bool attributes, set by presence whatever their value. Not `checked`:
// Blitz reads `checked="false"` as unticked, and `reset` writes it.
const FALSE_FLAGS: &str = "[disabled=false], [hidden=false], [readonly=false], \
    [required=false], [selected=false], [multiple=false], [open=false], \
    [autofocus=false], [inert=false]";

/// Mounts `app` inside a [`LiberoProvider`] and settles the first render.
pub fn mount(app: fn() -> Element) -> Page {
    mount_in(app, ColorScheme::Light)
}

/// [`mount`], with the window theme set to `scheme`.
pub fn mount_in(app: fn() -> Element, scheme: ColorScheme) -> Page {
    let storage = storage_share();
    let warnings = SignalWarnings::watch();
    let vdom = VirtualDom::new_with_props(Root, RootProps { app: App(app) });
    let navigations = Arc::new(Navigations::default());
    let mut doc = DioxusDocument::new(
        vdom,
        DocumentConfig {
            viewport: Some(viewport(scheme)),
            html_parser_provider: Some(Arc::new(blitz_html::HtmlProvider)),
            net_provider: Some(Arc::new(DataUrls)),
            navigation_provider: Some(navigations.clone()),
            ..Default::default()
        },
    );
    let redraws = Arc::new(Redraws::default());
    doc.inner.borrow_mut().set_shell_provider(redraws.clone());
    doc.initial_build();
    let mut page = Page {
        doc,
        time: 0.0,
        warnings,
        redraws,
        navigations,
        last_press: None,
        _storage: storage,
    };
    page.settle();
    page
}

/// Parallel tests share the process: a kept scheme or direction would leak between
/// them, so every read and write fails and choices last the mount, as before 2228.
const NO_STORAGE_DIR: &str = "/dev/null";

static STORAGE: RwLock<()> = RwLock::new(());

static STORAGE_WANTED: AtomicBool = AtomicBool::new(false);

/// How long [`with_storage_dir`] waits for every other test's pages to drop.
const STORAGE_WAIT: Duration = Duration::from_secs(60);

type StorageShare = Rc<RwLockReadGuard<'static, ()>>;

thread_local! {
    static OWNS_STORAGE: Cell<bool> = const { Cell::new(false) };
    /// One share per thread: a thread holding one must never wait for a waiting writer.
    static SHARE: RefCell<Weak<RwLockReadGuard<'static, ()>>> = const { RefCell::new(Weak::new()) };
}

fn no_storage() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| libero::platform::set_storage_dir(NO_STORAGE_DIR));
}

/// The thread's share of [`STORAGE`]; `None` inside [`with_storage_dir`].
fn storage_share() -> Option<StorageShare> {
    no_storage();
    if OWNS_STORAGE.get() {
        return None;
    }
    SHARE.with_borrow_mut(|share| {
        Some(share.upgrade().unwrap_or_else(|| {
            // Let a waiting `with_storage_dir` in, else a stream of new pages starves it.
            while STORAGE_WANTED.load(Ordering::Acquire) {
                thread::sleep(Duration::from_millis(5));
            }
            let fresh = Rc::new(STORAGE.read().unwrap_or_else(PoisonError::into_inner));
            *share = Rc::downgrade(&fresh);
            fresh
        }))
    })
}

/// Runs `test` with local storage kept in `dir` while no other test's page lives,
/// so no stray scheme lands there. Call it before mounting anything on the thread.
pub fn with_storage_dir<T>(dir: &Path, test: impl FnOnce() -> T) -> T {
    struct Restore;
    impl Drop for Restore {
        fn drop(&mut self) {
            OWNS_STORAGE.set(false);
            libero::platform::set_storage_dir(NO_STORAGE_DIR);
        }
    }
    no_storage();
    // Bounded: a page leaked by another test must fail this one, not hang the run.
    let started = Instant::now();
    STORAGE_WANTED.store(true, Ordering::Release);
    let _alone = loop {
        match STORAGE.try_write() {
            Ok(alone) => break alone,
            Err(TryLockError::Poisoned(poisoned)) => break poisoned.into_inner(),
            Err(TryLockError::WouldBlock) if started.elapsed() > STORAGE_WAIT => {
                STORAGE_WANTED.store(false, Ordering::Release);
                panic!("another test's page kept the storage lock for {STORAGE_WAIT:?}")
            }
            Err(TryLockError::WouldBlock) => thread::sleep(Duration::from_millis(5)),
        }
    };
    STORAGE_WANTED.store(false, Ordering::Release);
    libero::platform::set_storage_dir(dir);
    OWNS_STORAGE.set(true);
    let _restore = Restore;
    test()
}

/// The shell: counts the redraws the document asks for, and holds the
/// clipboard a paste reads ([`Page::set_clipboard`]).
#[derive(Default)]
struct Redraws(AtomicUsize, std::sync::Mutex<String>);

impl ShellProvider for Redraws {
    fn request_redraw(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }

    fn get_clipboard_text(&self) -> Result<String, blitz_traits::shell::ClipboardError> {
        Ok(self.1.lock().unwrap().clone())
    }

    fn set_clipboard_text(&self, text: String) -> Result<(), blitz_traits::shell::ClipboardError> {
        *self.1.lock().unwrap() = text;
        Ok(())
    }
}

/// The shell's link handler, recording what it would open instead of opening it.
#[derive(Default)]
struct Navigations(std::sync::Mutex<Vec<String>>);

impl NavigationProvider for Navigations {
    fn navigate_to(&self, options: NavigationOptions) {
        self.0.lock().unwrap().push(options.url.to_string());
    }
}

/// The network: only `data:` URLs, answered at once, so a test's pictures load
/// (`data:image/svg+xml,<svg ...>`, percent-escapes allowed; base64 as libero's
/// `bytes_data_url` writes it).
struct DataUrls;

impl NetProvider for DataUrls {
    fn fetch(&self, _doc_id: usize, request: Request, handler: Box<dyn NetHandler>) {
        let url = request.url.as_str();
        let Some((meta, payload)) = url
            .strip_prefix("data:")
            .and_then(|rest| rest.split_once(','))
        else {
            return;
        };
        let bytes = match meta.ends_with(";base64") {
            true => base64_decode(payload),
            false => percent_decode(payload),
        };
        handler.bytes(url.to_string(), Bytes::from(bytes));
    }
}

fn base64_decode(text: &str) -> Vec<u8> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let sextets: Vec<u32> = text
        .bytes()
        .filter_map(|byte| ALPHABET.iter().position(|&a| a == byte))
        .map(|at| at as u32)
        .collect();
    let mut out = Vec::with_capacity(sextets.len() * 3 / 4);
    for group in sextets.chunks(4) {
        let bits = group
            .iter()
            .enumerate()
            .fold(0, |bits, (i, sextet)| bits | sextet << (18 - 6 * i));
        // Two sextets make one byte, three two, four three.
        out.extend_from_slice(&bits.to_be_bytes()[1..group.len()]);
    }
    out
}

fn percent_decode(text: &str) -> Vec<u8> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let escaped = (bytes[i] == b'%')
            .then(|| text.get(i + 1..i + 3))
            .flatten()
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match escaped {
            Some(byte) => {
                out.push(byte);
                i += 3;
            }
            None => {
                out.push(bytes[i]);
                i += 1;
            }
        }
    }
    out
}

/// A painted colour in `computed`'s notation, so the two compare.
fn css_color(color: peniko::Color) -> String {
    let [r, g, b, a] = color.to_rgba8().to_u8_array();
    if a == 255 {
        format!("rgb({r}, {g}, {b})")
    } else {
        format!("rgba({r}, {g}, {b}, {})", a as f32 / 255.0)
    }
}

/// Blitz's client rect leaves out `transform`; the web's does not. libero's
/// `client_rect` in `platform/backend/blitz.rs` does the same walk.
/// libero's Blitz `client_rect`: transforms followed, a table row its cells' union.
fn client_rect(doc: &BaseDocument, id: NodeId) -> Option<(f64, f64, f64, f64)> {
    if let Some(rect) = boxless_rect(doc, id).or_else(|| transformed_rect(doc, id)) {
        return Some(rect);
    }
    let rect = doc.get_client_bounding_rect(id)?;
    let own = doc.get_node(id)?.scroll_offset();
    Some((rect.x + own.x, rect.y + own.y, rect.width, rect.height))
}

fn computed_value(doc: &BaseDocument, id: NodeId, property: &str) -> String {
    let Ok(property) = PropertyId::parse_enabled_for_all_content(property) else {
        return String::new();
    };
    let Some(style) = doc.get_node(id).and_then(|node| node.primary_styles()) else {
        return String::new();
    };
    let shorthand = match property.as_shorthand() {
        Ok(shorthand) => shorthand,
        Err(id) => return style.computed_value_to_string(id),
    };
    // A shorthand as `getComputedStyle` reads it: one value when its longhands agree (2866).
    let mut values: Vec<String> = shorthand
        .longhands()
        .map(|longhand| style.computed_value_to_string(PropertyDeclarationId::Longhand(longhand)))
        .collect();
    values.dedup();
    match (shorthand, values.as_slice()) {
        (ShorthandId::WhiteSpace, [wrap, collapse]) => {
            white_space(collapse, wrap).map_or_else(|| values.join(" "), str::to_owned)
        }
        _ => values.join(" "),
    }
}

/// `white-space` from its `white-space-collapse` and `text-wrap-mode`.
fn white_space(collapse: &str, wrap: &str) -> Option<&'static str> {
    Some(match (collapse, wrap) {
        ("collapse", "wrap") => "normal",
        ("collapse", "nowrap") => "nowrap",
        ("preserve", "nowrap") => "pre",
        ("preserve", "wrap") => "pre-wrap",
        ("preserve-breaks", "wrap") => "pre-line",
        ("break-spaces", "wrap") => "break-spaces",
        _ => return None,
    })
}

/// A table row or row group, which lays out no box in Blitz: its children's union.
fn boxless_rect(doc: &BaseDocument, id: NodeId) -> Option<(f64, f64, f64, f64)> {
    let node = doc.get_node(id)?;
    let element = node.element_data()?;
    let size = node.final_layout().size;
    if !matches!(&*element.name.local, "tr" | "tbody" | "thead" | "tfoot")
        || size.width != 0.0
        || size.height != 0.0
    {
        return None;
    }
    let (left, top, right, bottom) = node
        .children
        .iter()
        .filter(|&&child| doc.get_node(child).is_some_and(|node| node.is_element()))
        .filter_map(|&child| client_rect(doc, child))
        .filter(|&(_, _, width, height)| width > 0.0 || height > 0.0)
        .fold(
            (
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
            ),
            |(left, top, right, bottom), (x, y, width, height)| {
                (
                    left.min(x),
                    top.min(y),
                    right.max(x + width),
                    bottom.max(y + height),
                )
            },
        );
    (right >= left).then_some((left, top, right - left, bottom - top))
}

fn transformed_rect(doc: &BaseDocument, node_id: NodeId) -> Option<(f64, f64, f64, f64)> {
    let node = doc.get_node(node_id)?;
    let size = node.unrounded_layout().size;
    let (width, height) = (f64::from(size.width), f64::from(size.height));
    let mut corners = [(0.0, 0.0), (width, 0.0), (0.0, height), (width, height)];
    let scale = doc.viewport().scale_f64();
    let mut transformed = false;
    let mut current = Some(node);
    while let Some(node) = current {
        let boxed = matches!(
            node.data,
            blitz_dom::NodeData::Element(_) | blitz_dom::NodeData::AnonymousBlock(_)
        );
        if let Some(transform) = boxed.then(|| *node.transform()).flatten() {
            let [a, b, c, d, e, f] = transform.as_coeffs();
            for (x, y) in &mut corners {
                (*x, *y) = (a * *x + c * *y + e / scale, b * *x + d * *y + f / scale);
            }
            transformed = true;
        }
        let location = node.final_layout().location;
        let parent = node.layout_parent.get().and_then(|id| doc.get_node(id));
        let scroll = parent.map(|parent| *parent.scroll_offset());
        for (x, y) in &mut corners {
            *x += f64::from(location.x) - scroll.map_or(0.0, |s| s.x);
            *y += f64::from(location.y) - scroll.map_or(0.0, |s| s.y);
        }
        current = parent;
    }
    if !transformed {
        return None;
    }
    let scroll = doc.viewport_scroll();
    let xs = corners.map(|(x, _)| x - scroll.x);
    let ys = corners.map(|(_, y)| y - scroll.y);
    let min_x = xs.iter().copied().fold(f64::INFINITY, f64::min);
    let min_y = ys.iter().copied().fold(f64::INFINITY, f64::min);
    let max_x = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let max_y = ys.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    Some((min_x, min_y, max_x - min_x, max_y - min_y))
}

/// crates.io Blitz stops marking ancestors at a stale dirty bit, freezing a tick's transition
/// (Blitz #789, todo 880). libero heals at a flush, read or pointer move; a tick has none.
fn heal_dirty_bits(doc: &BaseDocument) {
    let mut dirty = Vec::new();
    doc.visit(|id, node| {
        if node.has_dirty_descendants() {
            dirty.push(id);
        }
    });
    for id in dirty {
        let mut parent = doc.get_node(id).and_then(|node| node.parent);
        while let Some(node) = parent.and_then(|id| doc.get_node(id)) {
            node.set_dirty_descendants();
            parent = node.parent;
        }
    }
}

fn viewport(scheme: ColorScheme) -> Viewport {
    Viewport::new(VIEWPORT.0, VIEWPORT.1, 1.0, scheme)
}

// A newtype, because `fn` pointers compare by an address codegen may merge.
#[derive(Clone, Copy)]
struct App(fn() -> Element);

impl PartialEq for App {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::fn_addr_eq(self.0, other.0)
    }
}

#[component]
fn Root(app: App) -> Element {
    rsx! {
        LiberoProvider { Mounted { app } }
    }
}

// Its own scope, so `app`'s hooks see the provider's context.
#[component]
fn Mounted(app: App) -> Element {
    (app.0)()
}

/// A mounted document. Every action settles before it returns.
pub struct Page {
    pub doc: DioxusDocument,
    time: f64,
    warnings: SignalWarnings,
    redraws: Arc<Redraws>,
    navigations: Arc<Navigations>,
    /// When and where the last drag pressed, see [`Page::press_apart`].
    last_press: Option<(Instant, f32, f32)>,
    /// Shared while the page lives; `None` inside [`with_storage_dir`], which holds it alone.
    _storage: Option<StorageShare>,
}

impl Page {
    /// The URLs the document asked the shell to open, in order. dioxus-native
    /// hands http, https and mailto ones to the browser.
    pub fn navigations(&self) -> Vec<String> {
        self.navigations.0.lock().unwrap().clone()
    }

    /// How many redraws the document has asked the shell for: at rest, none.
    pub fn redraws(&self) -> usize {
        self.redraws.0.load(Ordering::Relaxed)
    }

    /// The text a text control's editor holds: Blitz keeps typing there, off `value`.
    pub fn editor_text(&self, selector: &str) -> String {
        let doc = self.doc.inner.borrow();
        let node = doc.get_node(self.node(selector)).expect("a text control");
        let element = node.element_data().expect("an element");
        let input = element.text_input_data().expect("a text control");
        input.editor.raw_text().to_string()
    }

    /// What the next Ctrl+V pastes.
    pub fn set_clipboard(&self, text: &str) {
        *self.redraws.1.lock().unwrap() = text.to_string();
    }

    /// Polls the vdom until it has no work left, then restyles and lays out.
    pub fn settle(&mut self) {
        let until = Instant::now() + WAIT_LIMIT;
        let mut polls = 0;
        loop {
            let busy = self.doc.poll(None);
            self.doc.inner.borrow_mut().resolve(self.time);
            if busy {
                polls += 1;
                assert!(
                    polls < MAX_POLLS,
                    "the vdom was still busy after {MAX_POLLS} polls"
                );
                continue;
            }
            // A shell's next frame runs libero's `when_laid_out` work, whose wake a
            // loaded machine delays (2596): poll until it ran.
            let awaits = self.doc.vdom.in_runtime(libero::platform::awaits_layout);
            if awaits && Instant::now() < until {
                thread::sleep(Duration::from_millis(1));
                continue;
            }
            self.assert_no_false_flags();
            self.assert_no_signal_warnings();
            return;
        }
    }

    /// dioxus-native writes a `false` bool attribute as `disabled="false"`,
    /// which Blitz reads as set; libero's flush drops it (todo 943).
    fn assert_no_false_flags(&self) {
        if let Some(id) = self.query(FALSE_FLAGS) {
            panic!(
                "a bool attribute written as \"false\" on {} in\n{}",
                self.describe(id),
                self.tree()
            );
        }
    }

    /// A `dioxus_signals` warning names a value read outside its owner's
    /// scope, which may be dropped under the reader.
    fn assert_no_signal_warnings(&self) {
        let seen = self.warnings.take();
        if !seen.is_empty() {
            panic!("dioxus_signals warned:\n{}", seen.join("\n"));
        }
    }

    /// Advances the animation clock, so a CSS transition reaches its end.
    pub fn advance(&mut self, seconds: f64) {
        self.time += seconds;
        heal_dirty_bits(&self.doc.inner.borrow());
        self.settle();
    }

    /// Sleeps in real time, then settles: libero's timers are OS threads.
    pub fn wait(&mut self, duration: Duration) {
        thread::sleep(duration);
        self.settle();
    }

    /// Whether `done` held within [`WAIT_LIMIT`], waiting in short settled real-time steps:
    /// a timer's work lands late on a loaded machine.
    pub fn wait_for(&mut self, mut done: impl FnMut(&Page) -> bool) -> bool {
        let until = Instant::now() + WAIT_LIMIT;
        while !done(self) {
            if Instant::now() >= until {
                return false;
            }
            self.wait(Duration::from_millis(5));
        }
        true
    }

    /// Sends one raw event through Blitz's pipeline and settles.
    pub fn dispatch(&mut self, event: UiEvent) {
        self.doc.handle_ui_event(event);
        self.settle();
    }

    /// Switches the window theme, as the shell does on a theme-change event.
    pub fn set_color_scheme(&mut self, scheme: ColorScheme) {
        self.doc.inner.borrow_mut().set_viewport(viewport(scheme));
        self.settle();
    }

    /// Zooms the page, as the shell's Ctrl+= does: CSS pixels grow, so the
    /// layout viewport shrinks to [`VIEWPORT`] / `zoom`.
    pub fn set_zoom(&mut self, zoom: f32) {
        {
            let mut doc = self.doc.inner.borrow_mut();
            let mut port = doc.viewport().clone();
            port.set_zoom(zoom);
            doc.set_viewport(port);
        }
        self.settle();
    }

    /// The window's size, [`VIEWPORT`] until a [`resize`](Self::resize).
    pub fn window_size(&self) -> (u32, u32) {
        self.doc.inner.borrow().viewport().window_size
    }

    /// Resizes the window, as the shell does on a resize event. `painted_*`
    /// still rasterise [`VIEWPORT`].
    pub fn resize(&mut self, width: u32, height: u32) {
        {
            let mut doc = self.doc.inner.borrow_mut();
            let mut viewport = doc.viewport().clone();
            viewport.window_size = (width, height);
            doc.set_viewport(viewport);
        }
        self.settle();
    }

    /// Key down and up on whatever holds focus.
    pub fn press(&mut self, key: Key) {
        self.press_with(key, Modifiers::empty());
    }

    pub fn press_with(&mut self, key: Key, modifiers: Modifiers) {
        self.dispatch(UiEvent::KeyDown(key_event(
            key.clone(),
            KeyState::Pressed,
            modifiers,
        )));
        self.dispatch(UiEvent::KeyUp(key_event(
            key,
            KeyState::Released,
            modifiers,
        )));
    }

    pub fn tab(&mut self) {
        self.press(Key::Tab);
    }

    pub fn shift_tab(&mut self) {
        self.press_with(Key::Tab, Modifiers::SHIFT);
    }

    /// Pointer down and up at the centre of the first match.
    pub fn click(&mut self, selector: &str) {
        let (x, y) = self.centre(selector);
        self.click_at(x, y);
    }

    /// Two clicks at the first match's centre, apart from the last double: Blitz counts
    /// on presses within 500 ms, and a third double in a row fired no `dblclick` (todo 2507).
    pub fn double_click(&mut self, selector: &str) {
        let (x, y) = self.centre(selector);
        self.press_apart(x, y);
        self.dispatch(UiEvent::PointerUp(self.pointer(x, y, false)));
        self.click_at(x, y);
        self.last_press = Some((Instant::now(), x, y));
    }

    /// Pointer down and up at a viewport point, e.g. on a backdrop.
    pub fn click_at(&mut self, x: f32, y: f32) {
        self.dispatch(UiEvent::PointerDown(self.pointer(x, y, true)));
        self.dispatch(UiEvent::PointerUp(self.pointer(x, y, false)));
    }

    /// [`click`](Self::click), but the vdom runs dry before the next layout, as
    /// a window's shell may poll several times between two frames.
    pub fn click_before_layout(&mut self, selector: &str) {
        let (x, y) = self.centre(selector);
        self.dispatch(UiEvent::PointerDown(self.pointer(x, y, true)));
        self.dispatch_before_layout(UiEvent::PointerUp(self.pointer(x, y, false)));
    }

    /// [`press`](Self::press), polled dry before the next layout.
    pub fn press_before_layout(&mut self, key: Key) {
        let modifiers = Modifiers::empty();
        self.doc.handle_ui_event(UiEvent::KeyDown(key_event(
            key.clone(),
            KeyState::Pressed,
            modifiers,
        )));
        self.dispatch_before_layout(UiEvent::KeyUp(key_event(
            key,
            KeyState::Released,
            modifiers,
        )));
    }

    /// [`dispatch`](Self::dispatch), polled dry before the next layout.
    pub fn dispatch_before_layout(&mut self, event: UiEvent) {
        self.doc.handle_ui_event(event);
        for _ in 0..MAX_POLLS {
            if !self.doc.poll(None) {
                break;
            }
        }
        self.settle();
    }

    /// Presses at the first match's centre, moves by `(dx, dy)` in eight steps,
    /// releases there. Waits out a double press of the drag before it.
    pub fn drag(&mut self, selector: &str, dx: f32, dy: f32) {
        const STEPS: u8 = 8;
        let (x, y) = self.centre(selector);
        self.press_apart(x, y);
        for step in 1..=STEPS {
            let t = f32::from(step) / f32::from(STEPS);
            self.dispatch(UiEvent::PointerMove(self.pointer(
                x + dx * t,
                y + dy * t,
                true,
            )));
        }
        self.dispatch(UiEvent::PointerUp(self.pointer(x + dx, y + dy, false)));
    }

    /// The steps of a hand-driven drag, each settled: press, move with the
    /// button held, release.
    pub fn press_at(&mut self, x: f32, y: f32) {
        self.press_apart(x, y);
    }

    /// Presses at `(x, y)` once that is no double press of the last drag's, so
    /// two drags on one handle (`Splitter` collapses on a double) need no sleep.
    /// `click` skips it: two clicks are how a test asks for a double.
    fn press_apart(&mut self, x: f32, y: f32) {
        let gap = double_press_gap(self.last_press, Instant::now(), x, y);
        if !gap.is_zero() {
            self.wait(gap);
        }
        self.dispatch(UiEvent::PointerDown(self.pointer(x, y, true)));
        // Stamped after the handler ran, so a slow dispatch cannot shrink the gap.
        self.last_press = Some((Instant::now(), x, y));
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.dispatch(UiEvent::PointerMove(self.pointer(x, y, true)));
    }

    pub fn release_at(&mut self, x: f32, y: f32) {
        self.dispatch(UiEvent::PointerUp(self.pointer(x, y, false)));
    }

    /// A finger lands on the first match's centre; [`touch_up`](Self::touch_up) lifts it.
    pub fn touch_down(&mut self, selector: &str) -> (f32, f32) {
        let (x, y) = self.centre(selector);
        self.dispatch(UiEvent::PointerDown(self.finger(x, y, true)));
        (x, y)
    }

    /// Lifts the finger at a viewport point.
    pub fn touch_up(&mut self, x: f32, y: f32) {
        self.dispatch(UiEvent::PointerUp(self.finger(x, y, false)));
    }

    /// A finger lands on a viewport point, moves by `(dx, dy)` in eight steps, lifts there.
    pub fn swipe_from(&mut self, x: f32, y: f32, dx: f32, dy: f32) {
        const STEPS: u8 = 8;
        self.dispatch(UiEvent::PointerDown(self.finger(x, y, true)));
        for step in 1..=STEPS {
            let t = f32::from(step) / f32::from(STEPS);
            self.dispatch(UiEvent::PointerMove(self.finger(
                x + dx * t,
                y + dy * t,
                true,
            )));
        }
        self.touch_up(x + dx, y + dy);
    }

    /// Moves the pointer, no button held, to the first match's centre.
    pub fn hover(&mut self, selector: &str) {
        let (x, y) = self.centre(selector);
        self.hover_at(x, y);
    }

    /// Moves the pointer, no button held, to a viewport point.
    pub fn hover_at(&mut self, x: f32, y: f32) {
        self.dispatch(UiEvent::PointerMove(self.pointer(x, y, false)));
    }

    /// Turns the wheel over the first match's centre, positive `dy` scrolling down. Blitz scrolls
    /// what is hovered: `hover` it first, once (a second move dropped the next wheel).
    pub fn wheel(&mut self, selector: &str, dy: f64) {
        let (x, y) = self.centre(selector);
        self.wheel_at(x, y, dy);
    }

    /// [`wheel`](Self::wheel) at a viewport point, whatever is there now.
    pub fn wheel_at(&mut self, x: f32, y: f32, dy: f64) {
        let coords = self.pointer(x, y, false).coords;
        self.dispatch(UiEvent::Wheel(BlitzWheelEvent {
            // Blitz's sign is a finger's: a negative delta scrolls down.
            delta: BlitzWheelDelta::Pixels(0.0, -dy),
            coords,
            buttons: MouseEventButtons::empty(),
            mods: Modifiers::empty(),
            element: Point { x: 0.0, y: 0.0 },
        }));
    }

    /// [`wheel`](Self::wheel) sideways: a positive `dx` scrolls right.
    pub fn wheel_x(&mut self, selector: &str, dx: f64) {
        let (x, y) = self.centre(selector);
        let coords = self.pointer(x, y, false).coords;
        self.dispatch(UiEvent::Wheel(BlitzWheelEvent {
            delta: BlitzWheelDelta::Pixels(-dx, 0.0),
            coords,
            buttons: MouseEventButtons::empty(),
            mods: Modifiers::empty(),
            element: Point { x: 0.0, y: 0.0 },
        }));
    }

    /// A node's vertical scroll offset.
    pub fn scroll_top(&self, selector: &str) -> f64 {
        let id = self.node(selector);
        self.doc
            .inner
            .borrow()
            .get_node(id)
            .map_or(0.0, |node| node.scroll_offset().y)
    }

    /// A node's `(left, top)` scroll offset.
    pub fn scroll_pos(&self, selector: &str) -> (f64, f64) {
        let id = self.node(selector);
        let doc = self.doc.inner.borrow();
        doc.get_node(id).map_or((0.0, 0.0), |node| {
            let offset = node.scroll_offset();
            (offset.x, offset.y)
        })
    }

    /// A node's `scrollHeight`.
    pub fn scroll_height(&self, selector: &str) -> f64 {
        let id = self.node(selector);
        let doc = self.doc.inner.borrow();
        doc.get_node(id)
            .map_or(0.0, |node| f64::from(node.scroll_height()))
    }

    /// Scrolls the first match to `(left, top)` by wheel, over its centre, as `scrollTo` lands.
    pub fn scroll_to(&mut self, selector: &str, left: f64, top: f64) {
        let (x, y) = self.centre(selector);
        let (now_left, now_top) = self.scroll_pos(selector);
        self.hover_at(x, y);
        if left != now_left {
            self.wheel_x(selector, left - now_left);
        }
        if top != now_top {
            self.wheel_at(x, y, top - now_top);
        }
    }

    /// Every match's `getBoundingClientRect()`, as [`rect`](Self::rect) reads one.
    pub fn rects(&self, selector: &str) -> Vec<(f64, f64, f64, f64)> {
        let doc = self.doc.inner.borrow();
        self.query_all(selector)
            .into_iter()
            .filter_map(|id| client_rect(&doc, id))
            .collect()
    }

    /// Sets one inline style property of the first match, as `style.setProperty` does.
    pub fn set_style(&mut self, selector: &str, name: &str, value: &str) {
        let id = self.node(selector);
        self.doc
            .inner
            .borrow_mut()
            .set_style_property(id, name, value);
        self.settle();
    }

    /// Focuses the first match directly, the way `element.focus()` does, scrolling it into view.
    /// Blitz fires no focus or blur event for it.
    pub fn focus(&mut self, selector: &str) {
        let id = self.node(selector);
        {
            let mut doc = self.doc.inner.borrow_mut();
            doc.set_focus_to(id);
            // libero's Tab-move walk: scroll-padding and scroll-margin honoured (2863).
            self.doc
                .vdom
                .in_runtime(|| libero::platform::reveal_in(&mut doc, id));
        }
        self.settle();
    }

    pub fn query(&self, selector: &str) -> Option<NodeId> {
        self.doc
            .inner
            .borrow()
            .query_selector(selector)
            .unwrap_or_else(|_| panic!("Blitz cannot parse the selector {selector:?}"))
    }

    pub fn query_all(&self, selector: &str) -> Vec<NodeId> {
        self.doc
            .inner
            .borrow()
            .query_selector_all(selector)
            .unwrap_or_else(|_| panic!("Blitz cannot parse the selector {selector:?}"))
            .to_vec()
    }

    /// The text of every laid-out run inside the matches that broke onto more
    /// than one line: the painted text, not the box (todo 627).
    pub fn wrapped_text(&self, selector: &str) -> Vec<String> {
        let doc = self.doc.inner.borrow();
        let mut stack = self.query_all(selector);
        let mut wrapped = Vec::new();
        while let Some(id) = stack.pop() {
            let node = doc.get_node(id).expect("a matched node");
            let inline = node
                .data
                .downcast_element()
                .and_then(|data| data.inline_layout_data.as_ref());
            if let Some(inline) = inline.filter(|inline| inline.layout.len() > 1) {
                let lines = inline.layout.len();
                wrapped.push(format!("{:?} in {lines} lines", inline.text));
            }
            // Layout children too: a flex item's text sits in an anonymous block.
            let layout = node.layout_children.borrow();
            let children = layout.as_deref().unwrap_or(&node.children);
            stack.extend(children.iter().copied());
        }
        wrapped
    }

    /// The first match. Panics with the tree when there is none.
    pub fn node(&self, selector: &str) -> NodeId {
        self.query(selector)
            .unwrap_or_else(|| panic!("nothing matches {selector:?} in\n{}", self.tree()))
    }

    pub fn exists(&self, selector: &str) -> bool {
        self.query(selector).is_some()
    }

    /// A click on nothing focusable leaves Blitz's focus on `<html>`, not `None`.
    pub fn focused(&self) -> Option<NodeId> {
        self.doc.inner.borrow().get_focussed_node_id()
    }

    /// What holds focus, for assertion messages.
    pub fn focus_owner(&self) -> String {
        self.focused()
            .map(|id| self.describe(id))
            .unwrap_or_else(|| "nothing".into())
    }

    pub fn is_focused(&self, selector: &str) -> bool {
        self.focused()
            .is_some_and(|id| self.query_all(selector).contains(&id))
    }

    /// An attribute of the first match.
    pub fn attr(&self, selector: &str, name: &str) -> Option<String> {
        self.attr_of(self.node(selector), name)
    }

    pub fn attr_of(&self, id: NodeId, name: &str) -> Option<String> {
        let doc = self.doc.inner.borrow();
        let node = doc.get_node(id)?;
        node.attrs()?
            .iter()
            .find(|attr| *attr.name.local == *name)
            .map(|attr| attr.value.clone())
    }

    pub fn text(&self, selector: &str) -> String {
        let id = self.node(selector);
        self.doc
            .inner
            .borrow()
            .get_node(id)
            .map(Node::text_content)
            .unwrap_or_default()
    }

    /// The text Blitz lays out in the first inline context at or under the match:
    /// what the reader sees, after Blitz's whitespace handling.
    pub fn laid_out_text(&self, selector: &str) -> String {
        let doc = self.doc.inner.borrow();
        let mut queue = std::collections::VecDeque::from([self.node(selector)]);
        while let Some(id) = queue.pop_front() {
            let node = doc.get_node(id).expect("a laid out node");
            if let Some(inline) = node
                .data
                .downcast_element()
                .and_then(|data| data.inline_layout_data.as_ref())
            {
                return inline.text.clone();
            }
            let layout = node.layout_children.borrow();
            queue.extend(layout.as_deref().unwrap_or(&node.children).iter().copied());
        }
        panic!("no inline layout at or under {selector:?}")
    }

    /// `getComputedStyle(match).getPropertyValue(property)`: the computed
    /// value, or the used one for layout-dependent properties.
    pub fn computed(&self, selector: &str, property: &str) -> String {
        self.computed_of(self.node(selector), property)
    }

    pub fn computed_of(&self, id: NodeId, property: &str) -> String {
        computed_value(&self.doc.inner.borrow(), id, property)
    }

    /// The stroke Blitz will paint the first match's (an `svg`) first stroked
    /// path with, as `rgb(..)`. Not `computed`: Blitz bakes it at box build.
    pub fn painted_stroke(&self, selector: &str) -> String {
        self.painted_stroke_of(self.node(selector))
    }

    /// [`painted_stroke`](Self::painted_stroke) of a node.
    pub fn painted_stroke_of(&self, id: NodeId) -> String {
        let selector = self.describe(id);
        let doc = self.doc.inner.borrow();
        let element = doc
            .get_node(id)
            .and_then(|node| node.element_data())
            .unwrap();
        let SpecialElementData::Image(image) = &element.special_data else {
            panic!("{selector:?} was not built as an image");
        };
        let ImageData::Svg(svg) = &**image else {
            panic!("{selector:?} was not built as an svg");
        };
        let tree = format!("{:?}", svg.tree.root());
        let start = tree.find("Color { red: ").expect("no stroke colour") + "Color { red: ".len();
        let rest = &tree[start..];
        let channels: Vec<&str> = rest[..rest.find(" }").unwrap()]
            .split(", ")
            .map(|channel| channel.rsplit(": ").next().unwrap())
            .collect();
        format!("rgb({})", channels.join(", "))
    }

    /// The transform Blitz paints the first match with, as affine coefficients.
    /// Not `computed`: Blitz caches it at layout.
    pub fn painted_transform(&self, selector: &str) -> Option<[f64; 6]> {
        let id = self.node(selector);
        let doc = self.doc.inner.borrow();
        doc.get_node(id)?
            .transform()
            .map(|affine| affine.as_coeffs())
    }

    /// The scene Blitz would draw now, recorded rather than rasterised.
    fn scene(&self) -> Scene {
        let mut scene = Scene::new();
        let mut doc = self.doc.inner.borrow_mut();
        blitz_paint::paint_scene(&mut scene, &mut doc, 1.0, VIEWPORT.0, VIEWPORT.1, 0, 0);
        scene
    }

    /// The `rgb(..)` of the pixel at `(x, y)` as the CPU renderer rasterises it,
    /// where a recorded command may still draw nothing (a zero-blur shadow).
    pub fn painted_pixel(&self, x: u32, y: u32) -> String {
        let [r, g, b, a] = self.painted_pixels(&[(x, y)])[0];
        css_color(peniko::Color::from_rgba8(r, g, b, a))
    }

    /// [`painted_pixel`](Self::painted_pixel) for many points, rasterised once.
    pub fn painted_pixels(&self, points: &[(u32, u32)]) -> Vec<[u8; 4]> {
        let (width, height) = VIEWPORT;
        let mut doc = self.doc.inner.borrow_mut();
        let buffer = anyrender::render_to_buffer::<VelloCpuImageRenderer, _>(
            |scene| blitz_paint::paint_scene(scene, &mut doc, 1.0, width, height, 0, 0),
            width,
            height,
        );
        points
            .iter()
            .map(|&(x, y)| {
                let at = ((y * width + x) * 4) as usize;
                [0, 1, 2, 3].map(|i| buffer[at + i])
            })
            .collect()
    }

    /// Times `steps` wheel steps of `dy` px over `selector`, each one's event and
    /// re-render, then its CPU raster of the viewport (1086). Report only.
    pub fn time_raster_steps(&mut self, selector: &str, steps: usize, dy: f64) -> RasterTimes {
        let (x, y) = self.centre(selector);
        let ms = |started: Instant| started.elapsed().as_secs_f64() * 1000.0;
        let (mut update, mut raster) = (Vec::with_capacity(steps), Vec::with_capacity(steps));
        for _ in 0..steps {
            let started = Instant::now();
            self.wheel_at(x, y, dy);
            update.push(ms(started));
            let started = Instant::now();
            self.painted_pixels(&[(0, 0)]);
            raster.push(ms(started));
        }
        RasterTimes {
            update: FrameStats::from_deltas(&update, 0),
            raster: FrameStats::from_deltas(&raster, 0),
        }
    }

    /// The whole viewport rasterised as a binary PPM, for looking at a failure.
    pub fn save_ppm(&self, path: &str) {
        let (width, height) = VIEWPORT;
        let mut doc = self.doc.inner.borrow_mut();
        let buffer = anyrender::render_to_buffer::<VelloCpuImageRenderer, _>(
            |scene| blitz_paint::paint_scene(scene, &mut doc, 1.0, width, height, 0, 0),
            width,
            height,
        );
        let mut out = format!("P6\n{width} {height}\n255\n").into_bytes();
        out.extend(buffer.chunks(4).flat_map(|px| [px[0], px[1], px[2]]));
        std::fs::write(path, out).expect("writable path");
    }

    /// The colour Blitz paints the first match's text with, as `rgb(..)`. Not
    /// `computed`: an anonymous block's text keeps the style of its box build.
    pub fn painted_text(&self, selector: &str) -> String {
        let (left, top, width, height) = self.rect(selector);
        let rect = kurbo::Rect::new(left, top, left + width, top + height);
        self.scene()
            .commands
            .iter()
            .find_map(|command| match command {
                RenderCommand::GlyphRun(GlyphRunCommand {
                    transform,
                    brush: Paint::Solid(color),
                    glyphs,
                    ..
                }) if glyphs.iter().any(|glyph| {
                    // Just above the baseline, inside the line box.
                    let at = kurbo::Point::new(glyph.x as f64, glyph.y as f64 - 1.0);
                    rect.contains(*transform * at)
                }) =>
                {
                    Some(css_color(*color))
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("no text painted in {selector:?}"))
    }

    /// `<tag #id [attr=value ...]>` of a node, for assertion messages.
    pub fn describe(&self, id: NodeId) -> String {
        let doc = self.doc.inner.borrow();
        doc.get_node(id)
            .map(describe)
            .unwrap_or_else(|| format!("#{id} (gone)"))
    }

    /// The first match's role and name in Blitz's accessibility tree, the role as
    /// accesskit's `Debug` name. Blitz names a node only by its own text children.
    pub fn accessible(&self, selector: &str) -> (String, String) {
        let id = self.node(selector).as_u64();
        let tree = self.doc.inner.borrow().build_accessibility_tree();
        let find = |id: u64| {
            tree.nodes
                .iter()
                .find(|(at, _)| at.0 == id)
                .map(|(_, node)| node)
        };
        let node = find(id).unwrap_or_else(|| panic!("{selector:?} is not in the AX tree"));
        let name = node
            .labelled_by()
            .iter()
            .filter_map(|by| find(by.0).and_then(|text| text.value()))
            .collect();
        (format!("{:?}", node.role()), name)
    }

    /// The element tree, one per line, for a failing assertion's message.
    pub fn tree(&self) -> String {
        let doc = self.doc.inner.borrow();
        let mut out = String::new();
        write_tree(&mut out, &doc, doc.root_node(), 0);
        out
    }

    /// The first match's `getBoundingClientRect()`: `(x, y, width, height)`, adding back
    /// the node's own scroll offset Blitz subtracts, as libero's `client_rect` does (todo 885).
    pub fn rect(&self, selector: &str) -> (f64, f64, f64, f64) {
        let id = self.node(selector);
        let doc = self.doc.inner.borrow();
        client_rect(&doc, id).unwrap_or_else(|| panic!("{selector:?} has no layout box"))
    }

    /// Whether a pointer at the first match's centre hits it or a descendant.
    pub fn hits(&self, selector: &str) -> bool {
        let (x, y) = self.centre(selector);
        self.hits_at(selector, x, y)
    }

    /// Whether a pointer at `(x, y)` hits the first match or a descendant.
    pub fn hits_at(&self, selector: &str, x: f32, y: f32) -> bool {
        let target = self.node(selector);
        let (left, top) = self.viewport_scroll();
        let doc = self.doc.inner.borrow();
        let mut hit = doc.hit(x + left, y + top).map(|hit| hit.node_id);
        while let Some(id) = hit {
            if id == target {
                return true;
            }
            hit = doc.get_node(id).and_then(|node| node.parent);
        }
        false
    }

    fn centre(&self, selector: &str) -> (f32, f32) {
        let (x, y, width, height) = self.rect(selector);
        ((x + width / 2.0) as f32, (y + height / 2.0) as f32)
    }

    /// How far the window itself has scrolled.
    pub fn viewport_scroll(&self) -> (f32, f32) {
        let scroll = self.doc.inner.borrow().viewport_scroll();
        (scroll.x as f32, scroll.y as f32)
    }

    /// A pointer at a viewport point. Blitz hit-tests the page point, so the
    /// window's scroll is added, as the shell does (todo 655).
    fn pointer(&self, x: f32, y: f32, down: bool) -> BlitzPointerEvent {
        let (left, top) = self.viewport_scroll();
        let mut event = pointer(x, y, down);
        event.coords.page_x += left;
        event.coords.page_y += top;
        event
    }

    /// [`pointer`](Self::pointer) as the first finger.
    fn finger(&self, x: f32, y: f32, down: bool) -> BlitzPointerEvent {
        BlitzPointerEvent {
            id: BlitzPointerId::Finger(1),
            ..self.pointer(x, y, down)
        }
    }
}

fn describe(node: &Node) -> String {
    let Some(element) = node.element_data() else {
        return format!("{:?}", node.text_content());
    };
    let attrs: String = element
        .attrs
        .iter()
        .filter(|attr| {
            let name = &*attr.name.local;
            // Listeners show up as attributes valued `<rust func>`.
            !matches!(name, "style" | "class" | "data-dioxus-id") && attr.value != "<rust func>"
        })
        .map(|attr| format!(" {}={:?}", attr.name.local, attr.value))
        .collect();
    format!("<{}{attrs}>", element.name.local)
}

fn write_tree(out: &mut String, doc: &BaseDocument, node: &Node, depth: usize) {
    let skip = node
        .element_data()
        .is_some_and(|element| matches!(&*element.name.local, "style" | "head"));
    if skip {
        return;
    }
    if node.element_data().is_some() {
        out.push_str(&"  ".repeat(depth));
        out.push_str(&describe(node));
        out.push('\n');
    }
    for child in &node.children {
        if let Some(child) = doc.get_node(*child) {
            write_tree(out, doc, child, depth + 1);
        }
    }
}

fn key_event(key: Key, state: KeyState, modifiers: Modifiers) -> BlitzKeyEvent {
    let text = match (&key, state) {
        (Key::Character(text), KeyState::Pressed) => Some(text.as_str().into()),
        _ => None,
    };
    BlitzKeyEvent {
        key,
        code: Code::Unidentified,
        modifiers,
        location: Location::Standard,
        is_auto_repeating: false,
        is_composing: false,
        state,
        text,
    }
}

/// How long a press at `(x, y)` waits to be no double press of `last`: libero's
/// `DoublePress` counts one within 500 ms and 2 px.
fn double_press_gap(last: Option<(Instant, f32, f32)>, now: Instant, x: f32, y: f32) -> Duration {
    match last {
        Some((at, last_x, last_y)) if (x - last_x).abs() <= 2.0 && (y - last_y).abs() <= 2.0 => {
            Duration::from_millis(500).saturating_sub(now.saturating_duration_since(at))
        }
        _ => Duration::ZERO,
    }
}

fn pointer(x: f32, y: f32, down: bool) -> BlitzPointerEvent {
    BlitzPointerEvent {
        id: BlitzPointerId::Mouse,
        is_primary: true,
        coords: PointerCoords {
            page_x: x,
            page_y: y,
            screen_x: x,
            screen_y: y,
            client_x: x,
            client_y: y,
        },
        button: MouseEventButton::Main,
        buttons: if down {
            MouseEventButtons::Primary
        } else {
            MouseEventButtons::empty()
        },
        mods: Modifiers::empty(),
        details: PointerDetails::default(),
        element: Point { x: 0.0, y: 0.0 },
        active_pointers: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_press_at_the_same_point_waits_out_the_500_ms() {
        let then = Instant::now();
        let last = Some((then, 10.0, 20.0));

        assert_eq!(
            double_press_gap(last, then + Duration::from_millis(120), 11.0, 21.5),
            Duration::from_millis(380)
        );
        assert_eq!(
            double_press_gap(last, then + Duration::from_millis(500), 10.0, 20.0),
            Duration::ZERO
        );
    }

    #[test]
    fn another_point_or_no_earlier_press_waits_for_nothing() {
        let then = Instant::now();
        let soon = then + Duration::from_millis(10);

        assert_eq!(
            double_press_gap(Some((then, 10.0, 20.0)), soon, 13.0, 20.0),
            Duration::ZERO
        );
        assert_eq!(double_press_gap(None, soon, 10.0, 20.0), Duration::ZERO);
    }
}
