use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use blitz_dom::BaseDocument;
use blitz_traits::{events::UiEvent, shell};
use dioxus::prelude::*;
use dioxus_native_dom::{NodeHandle, NodeId};

use super::INTERACTIVE;
use crate::{
    platform::{
        ColorSchemeApi, ColorSchemeSubscription, Dimensions, DocumentApi, ElementApi, KeyChord,
        KeySubscription, KeyboardApi, PlatformError, Read,
        keyboard::{takes_arrows, takes_typing},
        warn_reserved_chord,
    },
    tokens::{ColorScheme, ColorSchemeSetting},
};

/// Blitz backs a mounted element with a `NodeHandle`, which carries the whole
/// document - so an element can answer for its subtree, and (via [`document`])
/// for the document too.
pub(super) fn element(mounted: &Rc<MountedData>) -> Option<Box<dyn ElementApi>> {
    let handle = mounted.downcast::<NodeHandle>()?.clone();
    remember_document(&handle);
    let node_id = handle.node_id();
    Some(Box::new(BlitzElement {
        anchor: handle,
        node_id,
    }))
}

thread_local! {
    /// A `NodeHandle` is the only way to reach the Blitz document, and
    /// `use_modal` has to ask where focus is without owning an element. So the
    /// first handle we ever see is kept as an anchor: its node may since have
    /// unmounted, but the document it points into outlives it, and only
    /// document-wide calls are made through it.
    ///
    /// [`Outlet`] fills it when `LiberoProvider` mounts. Waiting for the first
    /// element handle to be *called* left it empty until then, so the first
    /// popover was never placed and the first modal returned focus nowhere
    /// (todo 191, seen in a native window).
    static ANCHOR: RefCell<Option<NodeHandle>> = const { RefCell::new(None) };

    /// Commands that found the document borrowed, oldest first. See
    /// [`BlitzElement::command`].
    static DEFERRED: RefCell<Vec<Deferred>> = const { RefCell::new(Vec::new()) };

    /// Bumped to remount [`Outlet`]'s flush element. `None` until it renders.
    static FLUSHES: RefCell<Option<Signal<u64>>> = const { RefCell::new(None) };

    /// A press whose focus move Blitz has not made yet: the focus owner at the
    /// press, and the focusable it hit. See [`Listener`].
    static PRESS: Cell<Option<(Option<NodeId>, NodeId)>> = const { Cell::new(None) };

    /// A press that hit nothing focusable, until its click. See [`refocus_wrapper`].
    static BLANK_PRESS: Cell<bool> = const { Cell::new(false) };

    /// [`Listener`]'s own element.
    static WRAPPER: RefCell<Option<NodeHandle>> = const { RefCell::new(None) };

    /// The node the last press hit, until its click has bubbled. See
    /// [`nested_interactive`].
    static HIT: Cell<Option<NodeId>> = const { Cell::new(None) };

    /// The drag [`follow_pointer`] stands in pointer capture for.
    static FOLLOW: Cell<Option<Follow>> = const { Cell::new(None) };

    /// The scheme Rust was last told, and who to tell when the viewport's
    /// differs. See [`BlitzColorScheme`].
    static SCHEME: Cell<ColorScheme> = const { Cell::new(ColorScheme::Light) };
    static SCHEME_CALLBACKS: RefCell<Vec<SchemeCallback>> = const { RefCell::new(Vec::new()) };
    static NEXT_CALLBACK: Cell<u64> = const { Cell::new(0) };

    /// Every [`BlitzKeyboard`] subscription: id, whether it skips text entry,
    /// callback. Called by [`Listener`]'s `onkeydown`.
    static KEY_CALLBACKS: RefCell<Vec<KeyCallback>> = const { RefCell::new(Vec::new()) };
}

type SchemeCallback = (u64, Rc<dyn Fn(ColorScheme)>);
type KeyCallback = (u64, bool, Rc<dyn Fn(KeyChord) -> bool>);

/// Wraps the app natively, so a press's target is known until its click has
/// bubbled, and a key press reaches [`BlitzKeyboard`]. Blitz runs a click's
/// handlers before it moves focus (web: after), so `remember_active()` in one
/// would name the element focused before.
#[component]
pub(super) fn Listener(children: Element) -> Element {
    rsx! {
        div {
            display: "contents",
            tabindex: "-1",
            onmounted: |event| {
                if let Some(handle) = event.data().downcast::<NodeHandle>() {
                    WRAPPER.with(|wrapper| *wrapper.borrow_mut() = Some(handle.clone()));
                }
            },
            // Bubble phase: this dioxus has no capture listeners.
            onpointerdown: move |event| pressed(&event),
            onclick: |_| {
                forget_press();
                if BLANK_PRESS.take() {
                    refocus_wrapper();
                }
            },
            onkeydown: |event| {
                forget_press();
                BLANK_PRESS.set(false);
                keyed(&event);
            },
            onpointermove: move |event| followed(&event, false),
            onpointerup: move |event| followed(&event, true),
            {children}
        }
    }
}

/// Blitz focuses `<html>` after a click on nothing focusable, above this
/// wrapper, where no key press would reach [`BlitzKeyboard`]. Taken back once
/// that move is made, at the end of the poll; the web's equivalent is `<body>`.
fn refocus_wrapper() {
    let Some(wrapper) = WRAPPER.with(|wrapper| wrapper.borrow().clone()) else {
        return;
    };
    let node_id = wrapper.node_id();
    defer(&wrapper, move |doc| {
        let root = doc.root_element().id;
        if doc
            .get_focussed_node_id()
            .is_none_or(|focused| focused == root)
        {
            doc.set_focus_to(node_id);
        }
    });
}

/// Hands a press that bubbled out of the app to every key subscription.
fn keyed(event: &Event<KeyboardData>) {
    // Bubble phase runs last: a press a handler below took is not the document's.
    if !event.default_action_enabled() || event.is_composing() {
        return;
    }
    let typing = typing_target();
    let callbacks: Vec<_> = KEY_CALLBACKS.with(|callbacks| {
        callbacks
            .borrow()
            .iter()
            .filter(|(_, skip_text_entry, _)| !(typing && *skip_text_entry))
            .map(|(_, skip_text_entry, callback)| (*skip_text_entry, callback.clone()))
            .collect()
    });
    let (key, modifiers) = (event.key(), event.modifiers());
    let mut taken = false;
    for (skip_text_entry, callback) in callbacks {
        let handled = callback(KeyChord {
            key: key.clone(),
            modifiers,
            repeat: event.is_auto_repeating(),
        });
        if handled && skip_text_entry {
            warn_reserved_chord(&key, modifiers);
        }
        taken |= handled;
    }
    if taken {
        event.prevent_default();
    }
}

/// The focused element's upper-case tag, `type` attribute, and whether it
/// sits in a `contenteditable`. Blitz sends a key press to the focused node.
fn focused_element() -> Option<(String, Option<String>, bool)> {
    let anchor = anchor()?;
    let doc = anchor.try_doc()?;
    let node = doc.get_node(doc.get_focussed_node_id()?)?;
    let element = node.element_data()?;
    let attr = |element: &blitz_dom::node::ElementData, name: &str| {
        element
            .attrs
            .iter()
            .find(|attr| *attr.name.local == *name)
            .map(|attr| attr.value.clone())
    };
    let tag = element.name.local.to_string().to_ascii_uppercase();
    let kind = attr(element, "type");
    // The nearest `contenteditable` decides, as it inherits on the web.
    let mut editable = false;
    let mut next = Some(node);
    while let Some(node) = next {
        if let Some(value) = node
            .element_data()
            .and_then(|data| attr(data, "contenteditable"))
        {
            editable = !value.eq_ignore_ascii_case("false");
            break;
        }
        next = node.parent.and_then(|parent| doc.get_node(parent));
    }
    Some((tag, kind, editable))
}

pub(super) fn typing_target() -> bool {
    focused_element()
        .is_some_and(|(tag, kind, editable)| editable || takes_typing(&tag, kind.as_deref()))
}

pub(super) fn arrow_target() -> bool {
    focused_element().is_some_and(|(tag, kind, _)| takes_arrows(&tag, kind.as_deref()))
}

pub(super) fn keyboard() -> Option<&'static dyn KeyboardApi> {
    Some(&KEYBOARD)
}

/// Key presses as they bubble out of the app to [`Listener`]. Unlike the web's
/// capture listener, a handler that stops a press first hides it, and one
/// while focus sits outside the wrapper never arrives.
struct BlitzKeyboard;

static KEYBOARD: BlitzKeyboard = BlitzKeyboard;

impl BlitzKeyboard {
    fn listen(
        &self,
        skip_text_entry: bool,
        callback: Box<dyn Fn(KeyChord) -> bool>,
    ) -> Box<dyn KeySubscription> {
        let id = NEXT_CALLBACK.replace(NEXT_CALLBACK.get() + 1);
        KEY_CALLBACKS.with(|callbacks| {
            callbacks
                .borrow_mut()
                .push((id, skip_text_entry, Rc::from(callback)))
        });
        Box::new(BlitzKeySubscription(id))
    }
}

impl KeyboardApi for BlitzKeyboard {
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

struct BlitzKeySubscription(u64);

impl KeySubscription for BlitzKeySubscription {}

impl Drop for BlitzKeySubscription {
    fn drop(&mut self) {
        KEY_CALLBACKS.with(|callbacks| callbacks.borrow_mut().retain(|(id, _, _)| *id != self.0));
    }
}

fn pressed(event: &Event<PointerData>) {
    let point = event.client_coordinates();
    let wrapper = WRAPPER.with(|wrapper| wrapper.borrow().as_ref().map(NodeHandle::node_id));
    HIT.set(None);
    let press = anchor().and_then(|anchor| {
        let doc = anchor.try_doc()?;
        let hit = doc.hit(point.x as f32, point.y as f32)?.node_id;
        HIT.set(Some(hit));
        // The wrapper is no press target: Blitz sends such a press's focus to `<html>`.
        let target = focusable_ancestor(&doc, hit).filter(|&target| Some(target) != wrapper);
        Some(target.map(|target| (doc.get_focussed_node_id(), target)))
    });
    BLANK_PRESS.set(matches!(press, Some(None)));
    PRESS.set(press.flatten());
}

fn forget_press() {
    PRESS.set(None);
    HIT.set(None);
}

/// The hit node and its ancestors, nearest first.
fn ancestors(doc: &BaseDocument, node_id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
    std::iter::successors(Some(node_id), |&id| doc.get_node(id)?.parent)
}

/// [`HIT`], walked up to the nearest `boundary`: nested when an interactive
/// element sits on the way, as the web's `closest` pair answers.
pub(super) fn nested_interactive(boundary: &str) -> bool {
    let nested = || -> Option<bool> {
        let hit = HIT.get()?;
        let anchor = anchor()?;
        let doc = anchor.try_doc()?;
        let boundaries = doc.query_selector_all(boundary).ok()?;
        let path: Vec<NodeId> = ancestors(&doc, hit).collect();
        let within = path.iter().position(|id| boundaries.contains(id))?;
        let interactive = doc.query_selector_all_in(path[within], INTERACTIVE).ok()?;
        Some(path[..within].iter().any(|id| interactive.contains(id)))
    };
    nested().unwrap_or(false)
}

/// A drag's moves and release, for the pointer it pressed with.
#[derive(Clone, Copy)]
struct Follow {
    pointer_id: i32,
    capture: NodeId,
    onmove: Callback<Event<PointerData>>,
    onup: Callback<Event<PointerData>>,
}

pub(super) fn follow_pointer(
    event: &Event<PointerData>,
    capture: &Rc<MountedData>,
    onmove: Callback<Event<PointerData>>,
    onup: Callback<Event<PointerData>>,
) {
    let Some(handle) = capture.downcast::<NodeHandle>() else {
        return;
    };
    FOLLOW.set(Some(Follow {
        pointer_id: event.pointer_id(),
        capture: handle.node_id(),
        onmove,
        onup,
    }));
}

/// Hands the followed drag what bubbled here from outside its element; what
/// lands inside reached the element's own handlers already. Bubble phase, so
/// an outside `stop_propagation` hides a move, and a release off the app
/// (on `<html>`, outside this wrapper) is never seen.
fn followed(event: &Event<PointerData>, up: bool) {
    let Some(follow) = FOLLOW.get().filter(|f| f.pointer_id == event.pointer_id()) else {
        return;
    };
    if up {
        FOLLOW.set(None);
    }
    let point = event.client_coordinates();
    let outside = anchor().and_then(|anchor| {
        let doc = anchor.try_doc()?;
        // Gone with its element, and so are the handlers.
        if !doc
            .get_node(follow.capture)
            .is_some_and(|node| node.flags.is_in_document())
        {
            return None;
        }
        let inside = doc
            .hit(point.x as f32, point.y as f32)
            .is_some_and(|hit| ancestors(&doc, hit.node_id).any(|id| id == follow.capture));
        Some(!inside)
    });
    match outside {
        Some(true) if up => follow.onup.call(event.clone()),
        Some(true) => follow.onmove.call(event.clone()),
        Some(false) => {}
        None => FOLLOW.set(None),
    }
}

fn focusable_ancestor(doc: &BaseDocument, mut node_id: NodeId) -> Option<NodeId> {
    loop {
        let node = doc.get_node(node_id)?;
        if node.is_focussable() {
            return Some(node_id);
        }
        node_id = node.parent?;
    }
}

type Deferred = (NodeHandle, Box<dyn FnOnce(&mut BaseDocument)>);

/// Mounted once by `LiberoProvider`. Its first mount is the document anchor;
/// every later one runs the deferred commands.
///
/// A mount is the one place dioxus-native calls back into user code with the
/// document free while no event is being dispatched:
/// `DioxusDocument::poll` fires `onmounted` after it drops the borrow it held
/// across `render_immediate`. So deferring bumps `FLUSHES`, the keyed element
/// below is replaced, and its `onmounted` runs the queue at the end of the
/// same poll (seen in a native window: a modal's focus return, deferred and
/// then run, before the next event).
#[component]
pub(super) fn Outlet() -> Element {
    let flushes = use_hook(|| {
        let flushes = Signal::new(0u64);
        FLUSHES.with(|slot| *slot.borrow_mut() = Some(flushes));
        flushes
    });
    use_drop(|| FLUSHES.with(|slot| *slot.borrow_mut() = None));

    rsx! {
        for flush in [flushes()] {
            div {
                key: "{flush}",
                display: "none",
                onmounted: move |event| {
                    if let Some(handle) = event.data().downcast::<NodeHandle>() {
                        remember_document(handle);
                    }
                    run_deferred();
                    // Once, at the provider's mount: see `BlitzColorScheme`.
                    if flush == 0 {
                        check_scheme();
                    }
                },
            }
        }
    }
}

/// Runs `run` at [`Outlet`]'s next flush, at the end of this poll.
fn defer(anchor: &NodeHandle, run: impl FnOnce(&mut BaseDocument) + 'static) {
    DEFERRED.with(|queue| queue.borrow_mut().push((anchor.clone(), Box::new(run))));
    FLUSHES.with(|slot| {
        if let Some(mut flushes) = *slot.borrow() {
            let next = flushes.peek().wrapping_add(1);
            flushes.set(next);
        }
    });
}

fn run_deferred() {
    let deferred = DEFERRED.with(|queue| std::mem::take(&mut *queue.borrow_mut()));
    for (anchor, command) in deferred {
        command(&mut anchor.doc_mut());
    }
}

fn remember_document(handle: &NodeHandle) {
    ANCHOR.with(|anchor| {
        let mut anchor = anchor.borrow_mut();
        if anchor.is_none() {
            *anchor = Some(handle.clone());
        }
    });
}

pub(super) fn document() -> Option<&'static dyn DocumentApi> {
    anchor().map(|_| &DOCUMENT as &'static dyn DocumentApi)
}

/// The anchor as of right now. Held in a `thread_local` rather than in
/// [`BlitzDocument`], so the document can be a `&'static` like every other
/// capability - and so a handle taken before the first frame is not stale.
fn anchor() -> Option<NodeHandle> {
    ANCHOR.with(|anchor| anchor.borrow().clone())
}

pub(super) fn color_scheme() -> Option<&'static dyn ColorSchemeApi> {
    Some(&COLOR_SCHEME)
}

/// The viewport's scheme, which the shell keeps in step with the window theme.
/// Read once, at [`Outlet`]'s first mount; `on_change` fires only for that
/// correction, as Blitz sends no event on a live theme change.
struct BlitzColorScheme;

static COLOR_SCHEME: BlitzColorScheme = BlitzColorScheme;

fn viewport_scheme() -> Option<ColorScheme> {
    let anchor = anchor()?;
    let doc = anchor.try_doc()?;
    Some(match doc.viewport().color_scheme {
        shell::ColorScheme::Dark => ColorScheme::Dark,
        shell::ColorScheme::Light => ColorScheme::Light,
    })
}

fn check_scheme() {
    let Some(scheme) = viewport_scheme() else {
        return;
    };
    if SCHEME.replace(scheme) == scheme {
        return;
    }
    let callbacks: Vec<_> = SCHEME_CALLBACKS.with(|callbacks| {
        callbacks
            .borrow()
            .iter()
            .map(|(_, callback)| callback.clone())
            .collect()
    });
    for callback in callbacks {
        callback(scheme);
    }
}

impl ColorSchemeApi for BlitzColorScheme {
    /// Before the first frame there is no document to ask, so this answers the
    /// last scheme known, and the first [`Outlet`] mount corrects it.
    fn system(&self) -> ColorScheme {
        if let Some(scheme) = viewport_scheme() {
            SCHEME.set(scheme);
        }
        SCHEME.get()
    }

    fn on_change(&self, callback: Box<dyn Fn(ColorScheme)>) -> Box<dyn ColorSchemeSubscription> {
        let id = NEXT_CALLBACK.replace(NEXT_CALLBACK.get() + 1);
        SCHEME_CALLBACKS.with(|callbacks| callbacks.borrow_mut().push((id, Rc::from(callback))));
        Box::new(BlitzColorSchemeSubscription(id))
    }

    /// Blitz has no storage, so an override lives for the session.
    fn stored(&self) -> Option<ColorSchemeSetting> {
        None
    }

    fn store(&self, _setting: ColorSchemeSetting) {}
}

struct BlitzColorSchemeSubscription(u64);

impl ColorSchemeSubscription for BlitzColorSchemeSubscription {}

impl Drop for BlitzColorSchemeSubscription {
    fn drop(&mut self) {
        SCHEME_CALLBACKS.with(|callbacks| callbacks.borrow_mut().retain(|(id, _)| *id != self.0));
    }
}

struct BlitzDocument;

static DOCUMENT: BlitzDocument = BlitzDocument;

impl DocumentApi for BlitzDocument {
    fn viewport(&self) -> Read<Dimensions> {
        let anchor = anchor();
        let answer = match anchor.as_ref().and_then(|anchor| anchor.try_doc()) {
            Some(doc) => {
                // `window_size` is physical pixels; layout, and so every
                // rect `client_offset` answers, is in CSS pixels.
                let scale = doc.viewport().scale_f64();
                let (width, height) = doc.viewport().window_size;
                Ok(Dimensions {
                    width: width as f64 / scale,
                    height: height as f64 / scale,
                })
            }
            None => Err(PlatformError::Unsupported),
        };
        Box::pin(std::future::ready(answer))
    }

    fn active_element(&self) -> Option<Box<dyn ElementApi>> {
        let anchor = anchor()?;
        let doc = anchor.try_doc()?;
        let focused = doc.get_focussed_node_id();
        // Focus unmoved since a press means Blitz has yet to move it there.
        let pressed = PRESS.get().filter(|&(before, target)| {
            before == focused
                && doc
                    .get_node(target)
                    .is_some_and(|node| node.flags.is_in_document())
        });
        let node_id = match pressed {
            Some((_, target)) => target,
            None => focused?,
        };
        drop(doc);
        Some(Box::new(BlitzElement { anchor, node_id }))
    }

    /// Not natively, for now - so the theme switch rebuilds its sheet here
    /// instead. **Conservative, not impossible**, and the comment that said
    /// the root is unreachable was wrong: dioxus-native does build a real
    /// `html`/`head`/`body`/`main` tree and `BaseDocument::root_element()`
    /// reaches the `<html>` that `:root` matches. What is unverified is
    /// whether mutating an attribute there marks the node dirty for a
    /// restyle, so todo 69 phase 4 should try it before keeping this
    /// fallback. `@media (prefers-color-scheme: dark)` *is* honoured
    /// natively either way - stylo evaluates it off the window theme - so
    /// only an explicit override needs this. See
    /// [[codebase/blitz-platform-gaps]].
    fn set_root_attribute(&self, _name: &str, _value: Option<&str>) -> bool {
        false
    }
}

struct BlitzElement {
    /// Any handle into the same document; `node_id` is what this addresses.
    /// Nodes found by a query have no `NodeHandle` of their own - its fields
    /// are private to dioxus - so one is carried along to reach the document.
    anchor: NodeHandle,
    node_id: NodeId,
}

impl BlitzElement {
    /// **A read answers when it is called, not when it is awaited.** Blitz
    /// hands its event driver the document as a mutable borrow and holds it
    /// across the whole dispatch - which includes dioxus draining every task
    /// spawned from a handler, so a read *created* inside a `spawn` finds the
    /// document locked and no amount of yielding escapes that window (measured:
    /// 2000 self-wakes, still borrowed). During the handler itself it is free.
    ///
    /// So callers must build the future where the handler is and await it
    /// wherever; the returned `Read` is already resolved here, exactly as on
    /// the web.
    fn read<T: 'static>(&self, from: impl FnOnce(&BaseDocument, NodeId) -> Option<T>) -> Read<T> {
        let Some(doc) = self.anchor.try_doc() else {
            return Box::pin(std::future::ready(Err(PlatformError::Unsupported)));
        };
        let answer = from(&doc, self.node_id).ok_or(PlatformError::NotFound);
        Box::pin(std::future::ready(answer))
    }

    /// **A command runs now if it can, and at the end of this poll if not.**
    /// Every dioxus task - anything `spawn`ed, and every timer callback - is
    /// polled inside `render_immediate`, which `DioxusDocument::poll` calls
    /// while it holds the document mutably (`dioxus_document.rs:233-236` in
    /// native-dom). A command from there cannot take the document, and
    /// `doc_mut()` panicked: `FocusReturn::restore` closing any modal did
    /// exactly that (todo 189, seen in a native window). An event handler finds
    /// it free.
    ///
    /// `try_doc` is how to ask - there is no `try_doc_mut` - and it fails only
    /// while the document is borrowed mutably, which is the case that matters:
    /// nothing in libero holds a shared borrow across a command.
    fn command(&self, run: impl FnOnce(&mut BaseDocument) + 'static) {
        if self.anchor.try_doc().is_some() {
            run(&mut self.anchor.doc_mut());
            return;
        }
        defer(&self.anchor, run);
    }

    fn at(&self, node_id: NodeId) -> Box<dyn ElementApi> {
        Box::new(BlitzElement {
            anchor: self.anchor.clone(),
            node_id,
        })
    }
}

impl ElementApi for BlitzElement {
    fn focus(&self) -> Result<(), PlatformError> {
        let node_id = self.node_id;
        self.command(move |doc| {
            doc.set_focus_to(node_id);
        });
        Ok(())
    }

    fn blur(&self) -> Result<(), PlatformError> {
        let node_id = self.node_id;
        self.command(move |doc| {
            if doc.get_focussed_node_id() == Some(node_id) {
                doc.clear_focus();
            }
        });
        Ok(())
    }

    /// Queued rather than dispatched: everything that reaches a dioxus
    /// `onclick` belongs to the `Document` wrapping this one, which only the
    /// shell holds - and we are usually inside a handler with the document
    /// mid-dispatch anyway. The shell runs it on its next turn, through the
    /// whole pipeline. Like every command here, "queued" is the answer.
    fn click(&self) -> Result<(), PlatformError> {
        let node_id = self.node_id;
        self.command(move |doc| doc.queue_ui_event(UiEvent::Activate(node_id)));
        Ok(())
    }

    fn is_focused(&self) -> bool {
        self.anchor
            .try_doc()
            .is_some_and(|doc| doc.get_focussed_node_id() == Some(self.node_id))
    }

    /// Blitz tracks this itself: `NodeFlags::IS_IN_DOCUMENT` is set across a
    /// whole subtree when it is inserted and cleared across it again when it
    /// is removed (`blitz-dom`'s `Mutator::process_added_subtree` /
    /// `process_removed_subtree`). So this is a flag read, not a walk up the
    /// parent chain, and it is exactly the DOM's `isConnected`.
    ///
    /// A removed node is also dropped from the slab, so a missing node is gone
    /// too - both answers are covered.
    ///
    /// No document in reach answers `true`, the trait's rule for a renderer
    /// that cannot tell. That case is real here: `try_doc` fails while dioxus
    /// is draining tasks, and an optimistic answer keeps focus return doing
    /// what it did before the predicate existed.
    fn is_connected(&self) -> bool {
        let Some(doc) = self.anchor.try_doc() else {
            return true;
        };
        doc.get_node(self.node_id)
            .is_some_and(|node| node.flags.is_in_document())
    }

    fn dimensions(&self) -> Read<Dimensions> {
        self.read(|doc, node_id| {
            let rect = doc.get_client_bounding_rect(node_id)?;
            Some(Dimensions {
                width: rect.width,
                height: rect.height,
            })
        })
    }

    fn client_offset(&self) -> Read<(f64, f64)> {
        self.read(|doc, node_id| {
            let rect = doc.get_client_bounding_rect(node_id)?;
            Some((rect.x, rect.y))
        })
    }

    fn scroll_size(&self) -> Read<Dimensions> {
        self.read(|doc, node_id| {
            let layout = doc.get_node(node_id)?.final_layout();
            Some(Dimensions {
                width: layout.scroll_width() as f64,
                height: layout.scroll_height() as f64,
            })
        })
    }

    fn scroll_offset(&self) -> Read<(f64, f64)> {
        self.read(|doc, node_id| {
            let offset = doc.get_node(node_id)?.scroll_offset();
            Some((offset.x, offset.y))
        })
    }

    /// Only a raster picture carries its size; a decoded SVG answers
    /// `NotFound`, as does an image still loading.
    fn natural_size(&self) -> Read<Dimensions> {
        self.read(|doc, node_id| {
            let image = doc.get_node(node_id)?.element_data()?.raster_image_data()?;
            Some(Dimensions {
                width: image.width as f64,
                height: image.height as f64,
            })
        })
    }

    /// Blitz only scrolls by a delta, so this reads the current offset first -
    /// when the command runs, which may be after it was asked for.
    fn scroll_to(&self, x: f64, y: f64) -> Result<(), PlatformError> {
        if self
            .anchor
            .try_doc()
            .is_some_and(|doc| doc.get_node(self.node_id).is_none())
        {
            return Err(PlatformError::NotFound);
        }
        let node_id = self.node_id;
        self.command(move |doc| {
            let Some(offset) = doc.get_node(node_id).map(|node| *node.scroll_offset()) else {
                return;
            };
            doc.scroll_node_by(node_id, x - offset.x, y - offset.y, |_| {});
        });
        Ok(())
    }

    /// Not written: finding the scrolling ancestor needs Blitz's overflow
    /// styles, and nothing native has asked for it.
    fn scroll_into_view(&self, _smooth: bool) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    /// Blitz has no `FileList`, and nothing native posts a form anyway.
    fn set_files(&self, _files: &[dioxus::html::FileData]) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    /// Blitz implements no form reset.
    fn reset(&self) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    /// Blitz fires no submit event from code.
    fn request_submit(&self) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    /// Blitz has no pointer capture. `use_drag` keeps tracking outside its
    /// element through [`follow_pointer`] instead.
    fn set_pointer_capture(&self, _pointer_id: i32) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    fn query_selector(&self, selector: &str) -> Result<Box<dyn ElementApi>, PlatformError> {
        let doc = self.anchor.try_doc().ok_or(PlatformError::Unsupported)?;
        let node_id = doc
            .query_selector_in(self.node_id, selector)
            .map_err(|_| PlatformError::NotFound)?
            .ok_or(PlatformError::NotFound)?;
        drop(doc);
        Ok(self.at(node_id))
    }

    fn query_selector_all(
        &self,
        selector: &str,
    ) -> Result<Vec<Box<dyn ElementApi>>, PlatformError> {
        let doc = self.anchor.try_doc().ok_or(PlatformError::Unsupported)?;
        let nodes = doc
            .query_selector_all_in(self.node_id, selector)
            .map_err(|_| PlatformError::NotFound)?;
        drop(doc);
        Ok(nodes.into_iter().map(|node_id| self.at(node_id)).collect())
    }
}
