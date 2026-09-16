use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

use blitz_dom::{BaseDocument, QualName, local_name, ns};
use blitz_traits::{events::UiEvent, shell};
use dioxus::prelude::*;
use dioxus_native_dom::{NodeHandle, NodeId};

mod focus;

pub(super) use focus::{press_kept_focus, silent_focus};

use super::INTERACTIVE;
use crate::{
    platform::{
        ColorSchemeApi, ColorSchemeSubscription, Dimensions, DocumentApi, ElementApi, KeyChord,
        KeySubscription, KeyboardApi, PlatformError, Read, ScrollApi, ScrollSubscription,
        TimerSubscription,
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

    /// Reads waiting for the next poll's layout. See [`when_laid_out`].
    static LAID_OUT: RefCell<Vec<Deferred>> = const { RefCell::new(Vec::new()) };
    static LAID_OUT_WAIT: RefCell<Option<Box<dyn TimerSubscription>>> = const { RefCell::new(None) };

    /// Bumped to remount [`Outlet`]'s flush element. `None` until it renders.
    static FLUSHES: RefCell<Option<Signal<u64>>> = const { RefCell::new(None) };
    /// The flush element that last mounted. `None` until the first one has.
    static MOUNTED_FLUSH: Cell<Option<u64>> = const { Cell::new(None) };

    /// A press whose focus move Blitz has not made yet: the focus owner at the
    /// press, and the focusable it hit. See [`Listener`].
    static PRESS: Cell<Option<(Option<NodeId>, NodeId)>> = const { Cell::new(None) };

    /// A press that hit nothing focusable, until its click. See [`refocus_wrapper`].
    static BLANK_PRESS: Cell<bool> = const { Cell::new(false) };

    /// What the last press on nothing focusable hit, until a Tab. See [`tab_from_start`].
    static START: Cell<Option<NodeId>> = const { Cell::new(None) };

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
    static SCHEME_CALLBACKS: Callbacks<dyn Fn(ColorScheme)> = const { Callbacks::new() };
    static NEXT_CALLBACK: Cell<u64> = const { Cell::new(0) };
    /// Re-reads the viewport while anyone listens to the scheme.
    static SCHEME_POLL: RefCell<Option<Box<dyn TimerSubscription>>> = const { RefCell::new(None) };

    /// Who to tell that something scrolled. See [`BlitzScroll`].
    static SCROLL_CALLBACKS: Callbacks<dyn Fn()> = const { Callbacks::new() };
    /// Puts [`PortalRoot`] back on the viewport's corner.
    static REALIGN: Cell<Option<Callback<()>>> = const { Cell::new(None) };
    /// Mounted [`PortalEntry`]s drawing something: with none, a scroll has
    /// nothing to realign.
    static PORTAL_ENTRIES: Cell<usize> = const { Cell::new(0) };
    /// A `scroll_into_view` waiting for its target's first layout. See [`show`].
    static SHOW_RETRY: RefCell<Option<Box<dyn TimerSubscription>>> = const { RefCell::new(None) };

    /// Every [`BlitzKeyboard`] subscription: id, whether it skips text entry,
    /// callback. Called by [`Listener`]'s `onkeydown`.
    static KEY_CALLBACKS: RefCell<Vec<KeyCallback>> = const { RefCell::new(Vec::new()) };
}

type KeyCallback = (u64, bool, Rc<dyn Fn(KeyChord) -> bool>);

/// Subscribers, each under the id its subscription drops.
struct Callbacks<F: ?Sized> {
    list: RefCell<Vec<(u64, Rc<F>)>>,
    next: Cell<u64>,
}

impl<F: ?Sized> Callbacks<F> {
    const fn new() -> Self {
        Self {
            list: RefCell::new(Vec::new()),
            next: Cell::new(0),
        }
    }

    fn add(&self, callback: Rc<F>) -> u64 {
        let id = self.next.replace(self.next.get() + 1);
        self.list.borrow_mut().push((id, callback));
        id
    }

    /// Whether any are left.
    fn remove(&self, id: u64) -> bool {
        let mut list = self.list.borrow_mut();
        list.retain(|(other, _)| *other != id);
        !list.is_empty()
    }

    /// A copy, so a callback may subscribe or unsubscribe.
    fn snapshot(&self) -> Vec<Rc<F>> {
        self.list
            .borrow()
            .iter()
            .map(|(_, callback)| callback.clone())
            .collect()
    }
}

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
            onmousedown: |event| focus::mouse_pressed(&event),
            onclick: |_| {
                forget_press();
                focus::clicked();
                if BLANK_PRESS.take() {
                    refocus_wrapper();
                }
            },
            onkeydown: |event| {
                forget_press();
                BLANK_PRESS.set(false);
                focus::forget_kept();
                focus::keyed(&event);
                keyed(&event);
                tab_from_start(&event);
            },
            onpointermove: move |event| followed(&event, false),
            onpointerup: move |event| {
                followed(&event, true);
                focus::released();
            },
            // A wheel bubbles where its scroll does not: see `BlitzScroll`.
            onwheel: |_| notify_scroll(),
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

/// The node a click just activated: the press's target, or for a key the
/// focused node. `None` after a press on nothing focusable.
fn activated(doc: &BaseDocument) -> Option<NodeId> {
    match PRESS.get() {
        Some((_, target)) => Some(target),
        None if BLANK_PRESS.get() => None,
        None => doc.get_focussed_node_id(),
    }
}

/// A submit button, enabled or not.
fn submit_button(doc: &BaseDocument, node_id: NodeId) -> Option<&blitz_dom::node::ElementData> {
    let element = doc.get_node(node_id)?.element_data()?;
    let kind = element
        .attr(local_name!("type"))
        .map(str::to_ascii_lowercase);
    let submits = match &*element.name.local {
        "button" => kind.as_deref().is_none_or(|kind| kind == "submit"),
        "input" => matches!(kind.as_deref(), Some("submit" | "image")),
        _ => false,
    };
    submits.then_some(element)
}

fn is_submitter(doc: &BaseDocument, node_id: NodeId) -> bool {
    submit_button(doc, node_id)
        .is_some_and(|element| element.attr(local_name!("disabled")).is_none())
}

/// The nearest `form` around `node_id`, the one a control belongs to.
fn owning_form(doc: &BaseDocument, node_id: NodeId) -> Option<NodeId> {
    let mut next = doc.get_node(node_id)?.parent;
    while let Some(id) = next {
        let node = doc.get_node(id)?;
        if node
            .element_data()
            .is_some_and(|element| &*element.name.local == "form")
        {
            return Some(id);
        }
        next = node.parent;
    }
    None
}

pub(super) fn activated_submitter(form: &Rc<MountedData>) -> bool {
    let Some(form) = form.downcast::<NodeHandle>() else {
        return false;
    };
    let Some(doc) = form.try_doc() else {
        return false;
    };
    activated(&doc).is_some_and(|target| {
        is_submitter(&doc, target) && owning_form(&doc, target) == Some(form.node_id())
    })
}

/// Input types that block implicit submission when a form has no submit
/// button and more than one of them (HTML's "implicit submission").
const BLOCKING: &[&str] = &[
    "text",
    "search",
    "url",
    "tel",
    "email",
    "password",
    "date",
    "month",
    "week",
    "time",
    "datetime-local",
    "number",
];

fn blocks_implicit_submission(element: &blitz_dom::node::ElementData) -> bool {
    &*element.name.local == "input"
        && BLOCKING.contains(
            &element
                .attr(local_name!("type"))
                .unwrap_or("text")
                .to_ascii_lowercase()
                .as_str(),
        )
}

fn is_checkable(element: &blitz_dom::node::ElementData) -> bool {
    &*element.name.local == "input"
        && matches!(
            element
                .attr(local_name!("type"))
                .unwrap_or("text")
                .to_ascii_lowercase()
                .as_str(),
            "checkbox" | "radio"
        )
}

pub(super) fn implicit_submission(form: &Rc<MountedData>) -> bool {
    let Some(form) = form.downcast::<NodeHandle>() else {
        return false;
    };
    let Some(doc) = form.try_doc() else {
        return false;
    };
    let form_id = form.node_id();
    let Some(focused) = doc.get_focussed_node_id() else {
        return false;
    };
    let focused_element = doc.get_node(focused).and_then(|node| node.element_data());
    let in_field = focused_element.is_some_and(blocks_implicit_submission);
    // A checkbox or radio submits through the default button only, as in
    // Chromium: it never counts as the form's one field.
    let in_checkable = focused_element.is_some_and(is_checkable);
    if !(in_field || in_checkable) || owning_form(&doc, focused) != Some(form_id) {
        return false;
    }
    let owned = |selector: &str| -> Vec<NodeId> {
        doc.query_selector_all_in(form_id, selector)
            .map(|nodes| {
                nodes
                    .into_iter()
                    .filter(|&id| owning_form(&doc, id) == Some(form_id))
                    .collect()
            })
            .unwrap_or_default()
    };
    // The web submits through the first submit button, and not at all when
    // that one is disabled.
    let default_button = owned("button, input")
        .into_iter()
        .find(|&id| submit_button(&doc, id).is_some());
    match default_button {
        Some(button) => is_submitter(&doc, button),
        None if in_checkable => false,
        None => {
            owned("input")
                .into_iter()
                .filter(|&id| {
                    doc.get_node(id)
                        .and_then(|node| node.element_data())
                        .is_some_and(blocks_implicit_submission)
                })
                .count()
                == 1
        }
    }
}

/// A box's `checked` attribute: what dioxus last wrote, its markup default.
fn checked_attr(element: &blitz_dom::node::ElementData) -> bool {
    element
        .attr(local_name!("checked"))
        .is_some_and(|checked| checked != "false")
}

/// The value a `<select>` posts: its selected options, else a single one's
/// first option, as the web picks. Blitz changes none on input.
fn select_values(doc: &BaseDocument, select: NodeId) -> Vec<String> {
    let Some(element) = doc.get_node(select).and_then(|node| node.element_data()) else {
        return Vec::new();
    };
    let multiple = element.attr(local_name!("multiple")).is_some();
    let Ok(options) = doc.query_selector_all_in(select, "option") else {
        return Vec::new();
    };
    let options: Vec<_> = options
        .into_iter()
        .filter_map(|id| {
            let node = doc.get_node(id)?;
            let option = node.element_data()?;
            let value = option
                .attr(local_name!("value"))
                .map(str::to_string)
                .unwrap_or_else(|| node.text_content().trim().to_string());
            let selected = option
                .attr(local_name!("selected"))
                .is_some_and(|selected| selected != "false");
            let disabled = option.attr(local_name!("disabled")).is_some();
            Some((value, selected, disabled))
        })
        .collect();
    let mut chosen = options
        .iter()
        .filter(|(_, selected, disabled)| *selected && !disabled);
    match multiple {
        true => chosen.map(|(value, ..)| value.clone()).collect(),
        false => chosen
            .next()
            .or_else(|| options.iter().find(|(_, _, disabled)| !disabled))
            .map(|(value, ..)| vec![value.clone()])
            .unwrap_or_default(),
    }
}

/// The named, enabled controls of `form`, as a submit would post them: text,
/// ticked boxes and selects. Files are left out.
pub(super) fn form_values(form: &Rc<MountedData>) -> Vec<(String, dioxus::html::FormValue)> {
    let Some(form) = form.downcast::<NodeHandle>() else {
        return Vec::new();
    };
    let Some(doc) = form.try_doc() else {
        return Vec::new();
    };
    let form_id = form.node_id();
    let Ok(controls) =
        doc.query_selector_all_in(form_id, "input[name], textarea[name], select[name]")
    else {
        return Vec::new();
    };
    controls
        .into_iter()
        .filter(|&id| owning_form(&doc, id) == Some(form_id))
        .flat_map(|id| {
            let posted = || -> Option<(String, Vec<String>)> {
                let element = doc.get_node(id)?.element_data()?;
                let name = element.attr(local_name!("name"))?.to_string();
                if element.attr(local_name!("disabled")).is_some() {
                    return None;
                }
                if &*element.name.local == "select" {
                    return Some((name, select_values(&doc, id)));
                }
                let kind = element
                    .attr(local_name!("type"))
                    .unwrap_or("text")
                    .to_ascii_lowercase();
                let value = match kind.as_str() {
                    // Blitz keeps a click's tick apart from the attribute.
                    "checkbox" | "radio" => element
                        .checkbox_input_checked()
                        .unwrap_or_else(|| checked_attr(element))
                        .then(|| {
                            element
                                .attr(local_name!("value"))
                                .unwrap_or("on")
                                .to_string()
                        })?,
                    "submit" | "image" | "button" | "reset" | "file" => return None,
                    _ => match element.text_input_data() {
                        Some(input) => input.editor.raw_text().to_string(),
                        None => element
                            .attr(local_name!("value"))
                            .unwrap_or_default()
                            .to_string(),
                    },
                };
                Some((name, vec![value]))
            };
            let (name, values) = posted().unwrap_or_default();
            values
                .into_iter()
                .map(move |value| (name.clone(), dioxus::html::FormValue::Text(value)))
        })
        .collect()
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

/// Blitz lays `position: fixed` out against the parent box, so the outlet
/// spans the viewport from its corner: a popover lands under its anchor and a
/// modal's `inset: 0` covers the window. `absolute`, not `fixed`: no stacking
/// context, so portaled z-indices still compete with the app's. It takes no
/// hits; [`PortalEntry`] gives them back to what it holds.
const PORTAL_ROOT_STYLE: &str = "position:absolute;width:100vw;height:100vh;pointer-events:none;";

/// The portal outlet, shifted back onto the viewport's corner whenever
/// something scrolls: an `absolute` box moves with a scrolled root, an offset
/// parent and a top margin collapsing through `<main>`.
#[component]
pub(super) fn PortalRoot(children: Element) -> Element {
    let mut shift = use_signal(|| (0.0, 0.0));
    let scrolled = use_signal(|| 0u64);
    let root = use_hook(|| Rc::new(RefCell::new(None::<NodeHandle>)));
    use_hook(|| {
        Rc::new(SCROLL.on_scroll(Box::new(move || {
            // An entry realigns as it mounts or starts to draw; until then a
            // wheel costs no render.
            if PORTAL_ENTRIES.get() == 0 {
                return;
            }
            let mut scrolled = scrolled;
            let next = scrolled.peek().wrapping_add(1);
            scrolled.set(next);
        })))
    });
    let realign = use_callback({
        let root = root.clone();
        move |()| {
            let Some(handle) = root.borrow().clone() else {
                return;
            };
            let Some(doc) = handle.try_doc() else {
                return;
            };
            let Some((x, y, ..)) = client_rect(&doc, handle.node_id()) else {
                return;
            };
            drop(doc);
            if x.abs() >= 0.5 || y.abs() >= 0.5 {
                let (left, top) = *shift.peek();
                shift.set((left - x, top - y));
            }
        }
    });
    use_hook(|| REALIGN.set(Some(realign)));
    use_drop(|| REALIGN.set(None));
    use_effect(move || {
        scrolled();
        realign.call(());
    });
    let (left, top) = shift();
    rsx! {
        div {
            style: "{PORTAL_ROOT_STYLE}left:{left}px;top:{top}px;",
            onmounted: move |event| {
                if let Some(handle) = event.data().downcast::<NodeHandle>() {
                    *root.borrow_mut() = Some(handle.clone());
                }
                // Laid out only after this poll's effects.
                when_laid_out(Box::new(move || realign.call(())));
            },
            {children}
        }
    }
}

/// One portaled entry, taking hits again below [`PORTAL_ROOT_STYLE`]. Counted
/// in [`PORTAL_ENTRIES`] only while it draws: an `idle` one needs no realign.
#[component]
pub(super) fn PortalEntry(children: Element, idle: bool) -> Element {
    let counted = use_hook(|| Rc::new(Cell::new(false)));
    if counted.get() == idle {
        counted.set(!idle);
        let entries = PORTAL_ENTRIES.get();
        PORTAL_ENTRIES.set(if idle { entries - 1 } else { entries + 1 });
    }
    use_drop({
        let counted = counted.clone();
        move || {
            if counted.get() {
                PORTAL_ENTRIES.set(PORTAL_ENTRIES.get() - 1);
            }
        }
    });
    // Waking up, it may find the outlet moved by a scroll it skipped.
    let mounted = use_hook(|| Rc::new(Cell::new(false)));
    use_effect(use_reactive!(|(idle,)| {
        if mounted.replace(true) && !idle {
            realign_portals();
        }
    }));
    rsx! {
        div {
            display: "contents",
            pointer_events: "auto",
            // The app's layout may have moved the outlet since the last scroll.
            onmounted: |_| realign_portals(),
            {children}
        }
    }
}

fn realign_portals() {
    if let Some(realign) = REALIGN.get() {
        realign.call(());
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
    let blank = matches!(press, Some(None));
    BLANK_PRESS.set(blank);
    START.set(HIT.get().filter(|_| blank));
    PRESS.set(press.flatten());
}

/// The web starts Tab and Shift+Tab from a clicked node that takes no focus;
/// Blitz from the focus owner, the wrapper. So focus goes to that node first,
/// and Blitz's move after this dispatch walks on from it.
fn tab_from_start(event: &Event<KeyboardData>) {
    if event.key() != Key::Tab {
        return;
    }
    let Some(start) = START.take() else {
        return;
    };
    if !event.default_action_enabled() {
        return;
    }
    let (Some(anchor), Some(wrapper)) = (
        anchor(),
        WRAPPER.with(|wrapper| wrapper.borrow().as_ref().map(NodeHandle::node_id)),
    ) else {
        return;
    };
    let Some(doc) = anchor.try_doc() else {
        return;
    };
    // Focus moved since the press: the web starts from the new owner too.
    let unmoved = doc
        .get_focussed_node_id()
        .is_none_or(|focused| focused == wrapper || focused == doc.root_element().id);
    let present = doc
        .get_node(start)
        .is_some_and(|node| node.flags.is_in_document());
    drop(doc);
    if unmoved && present {
        anchor.doc_mut().set_focus_to(start);
    }
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

type Deferred = Box<dyn FnOnce()>;

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
        MOUNTED_FLUSH.set(None);
        flushes
    });
    use_drop(|| FLUSHES.with(|slot| *slot.borrow_mut() = None));

    rsx! {
        for flush in [flushes()] {
            div {
                key: "{flush}",
                display: "none",
                onmounted: move |event| {
                    MOUNTED_FLUSH.set(Some(flush));
                    if let Some(handle) = event.data().downcast::<NodeHandle>() {
                        remember_document(handle);
                    }
                    run_deferred();
                    focus::check();
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
    let anchor = anchor.clone();
    later(move || run(&mut anchor.doc_mut()));
}

/// Makes [`Outlet`] flush at the end of this poll, sharing a pending flush.
fn flush_soon() {
    later(|| {});
}

/// Runs `run` now if the document is free, else at [`Outlet`]'s next flush,
/// where it is: reads made there answer.
pub(super) fn when_free(run: Box<dyn FnOnce()>) {
    if anchor().is_none_or(|anchor| anchor.try_doc().is_some()) {
        run();
        return;
    }
    later(run);
}

/// Runs `run` where the document is free and laid out: effects run before the
/// shell lays out what this poll mounted, so a timer waits for the next poll.
pub(super) fn when_laid_out(run: Box<dyn FnOnce()>) {
    let first = LAID_OUT.with(|waiting| {
        let mut waiting = waiting.borrow_mut();
        waiting.push(run);
        waiting.len() == 1
    });
    if !first {
        return;
    }
    let wait = super::thread::timer().map(|timer| {
        timer.after(
            Duration::ZERO,
            Box::new(|| {
                let waiting = LAID_OUT.with(|waiting| std::mem::take(&mut *waiting.borrow_mut()));
                for run in waiting {
                    when_free(run);
                }
            }),
        )
    });
    LAID_OUT_WAIT.with(|slot| drop(slot.replace(wait)));
}

fn later(run: impl FnOnce() + 'static) {
    DEFERRED.with(|queue| queue.borrow_mut().push(Box::new(run)));
    // One remount per flush: a flush element replaced before its mount event
    // ran panics in dioxus-native (a `Modal` backdrop click; todo 664: a timer
    // task re-keying it again in the poll that rendered it).
    FLUSHES.with(|slot| {
        if let Some(mut flushes) = *slot.borrow()
            && MOUNTED_FLUSH.get() == Some(*flushes.peek())
        {
            let next = flushes.peek().wrapping_add(1);
            flushes.set(next);
        }
    });
}

fn run_deferred() {
    let deferred = DEFERRED.with(|queue| std::mem::take(&mut *queue.borrow_mut()));
    for run in deferred {
        run();
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
/// Blitz sends no event on a live theme change, so it is read at [`Outlet`]'s
/// first mount and then every [`SCHEME_INTERVAL`] while anyone subscribes.
/// A timer's task finds the document free, unlike a handler's.
struct BlitzColorScheme;

static COLOR_SCHEME: BlitzColorScheme = BlitzColorScheme;

/// How late a live theme switch reaches Rust at most; the CSS follows at once.
const SCHEME_INTERVAL: Duration = Duration::from_millis(500);

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
    for callback in SCHEME_CALLBACKS.with(Callbacks::snapshot) {
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
        let id = SCHEME_CALLBACKS.with(|callbacks| callbacks.add(Rc::from(callback)));
        SCHEME_POLL.with(|poll| {
            let mut poll = poll.borrow_mut();
            if poll.is_none() {
                *poll = super::thread::timer()
                    .map(|timer| timer.every(SCHEME_INTERVAL, Box::new(check_scheme)));
            }
        });
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
        if !SCHEME_CALLBACKS.with(|callbacks| callbacks.remove(self.0)) {
            // Taken out before it drops, so no borrow is held across its drop.
            let poll = SCHEME_POLL.with(|poll| poll.borrow_mut().take());
            drop(poll);
        }
    }
}

pub(super) fn scroll() -> Option<&'static dyn ScrollApi> {
    Some(&SCROLL)
}

/// Blitz dispatches `scroll` only to the node that scrolled, and it does not
/// bubble. What moves it does: a wheel reaches [`Listener`], and libero's own
/// `scroll_to`/`scroll_into_view` report themselves. Blitz scrolls nothing on
/// a key or a mouse drag (measured on the harness).
struct BlitzScroll;

static SCROLL: BlitzScroll = BlitzScroll;

fn notify_scroll() {
    for callback in SCROLL_CALLBACKS.with(Callbacks::snapshot) {
        callback();
    }
}

impl ScrollApi for BlitzScroll {
    fn on_scroll(&self, callback: Box<dyn Fn()>) -> Box<dyn ScrollSubscription> {
        let id = SCROLL_CALLBACKS.with(|callbacks| callbacks.add(Rc::from(callback)));
        Box::new(BlitzScrollSubscription(id))
    }
}

struct BlitzScrollSubscription(u64);

impl ScrollSubscription for BlitzScrollSubscription {}

impl Drop for BlitzScrollSubscription {
    fn drop(&mut self) {
        SCROLL_CALLBACKS.with(|callbacks| callbacks.remove(self.0));
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

    /// On the `<html>` `root_element()` reaches; the mutator's write restyles
    /// it (todo 480). The theme is a colour change, so baked boxes rebuild with it.
    fn set_root_attribute(&self, name: &str, value: Option<&str>) -> bool {
        let Some(anchor) = anchor() else {
            return false;
        };
        let name = QualName::new(None, ns!(), name.into());
        let value = value.map(str::to_string);
        run_or_defer(&anchor, move |doc| {
            let root = doc.root_element().id;
            let mut mutator = doc.mutate();
            match value {
                Some(value) => mutator.set_attribute(root, name, &value),
                None => mutator.clear_attribute(root, name),
            }
            drop(mutator);
            rebuild_baked_boxes(doc);
        });
        true
    }

    fn colors_changed(&self) {
        if let Some(anchor) = anchor() {
            run_or_defer(&anchor, rebuild_baked_boxes);
        }
    }
}

/// Runs `run` now if the document is free, else at [`Outlet`]'s next flush.
fn run_or_defer(anchor: &NodeHandle, run: impl FnOnce(&mut BaseDocument) + 'static) {
    if anchor.try_doc().is_some() {
        run(&mut anchor.doc_mut());
    } else {
        defer(anchor, run);
    }
}

/// Blitz bakes colours into boxes it builds and keeps them: an inline `<svg>`'s
/// `currentColor` (todo 478), and the style of an anonymous block, the text of
/// a flex or grid container (todo 621). Rewriting an attribute rebuilds them.
fn rebuild_baked_boxes(doc: &mut BaseDocument) {
    let Ok(elements) = doc.query_selector_all("*") else {
        return;
    };
    let attrs: Vec<_> = elements
        .into_iter()
        .filter_map(|id| {
            let node = doc.get_node(id)?;
            let element = node.element_data()?;
            let anonymous = node
                .layout_children
                .borrow()
                .as_ref()
                .is_some_and(|boxes| boxes.iter().any(|id| !node.children.contains(id)));
            if !anonymous && &*element.name.local != "svg" {
                return None;
            }
            Some((id, element.attrs().first()?.clone()))
        })
        .collect();
    let mut mutator = doc.mutate();
    for (id, attr) in attrs {
        mutator.set_attribute(id, attr.name, &attr.value);
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
        run_or_defer(&self.anchor, run);
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
            focus::watch(doc);
            focus::requested(node_id);
            doc.set_focus_to(node_id);
        });
        Ok(())
    }

    fn blur(&self) -> Result<(), PlatformError> {
        let node_id = self.node_id;
        self.command(move |doc| {
            if doc.get_focussed_node_id() == Some(node_id) {
                focus::watch(doc);
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
            let (x, y, _, _) = client_rect(doc, node_id)?;
            Some((x, y))
        })
    }

    /// Taffy's `scroll_width` is the overflow, the furthest offset, not the
    /// content's size: the box's own size is added back (todo 652).
    fn scroll_size(&self) -> Read<Dimensions> {
        self.read(|doc, node_id| {
            let layout = doc.get_node(node_id)?.final_layout();
            Some(Dimensions {
                width: (layout.size.width + layout.scroll_width()) as f64,
                height: (layout.size.height + layout.scroll_height()) as f64,
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

    /// Stylo's resolved value, so a length arrives in `px` as on the web.
    fn computed_px(&self, property: &str) -> Read<Option<f64>> {
        self.read(|doc, node_id| {
            doc.get_node(node_id)?;
            let value = doc.resolved_style_value(node_id, property);
            Some(
                value
                    .strip_suffix("px")
                    .and_then(|px| px.trim().parse().ok()),
            )
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
            // A wheel's sign: a positive delta scrolls towards the start.
            doc.scroll_node_by(node_id, offset.x - x, offset.y - y, |_| {});
            notify_scroll();
        });
        Ok(())
    }

    /// The web's walk over Blitz's layout; `smooth` is ignored, as Blitz only
    /// jumps.
    fn scroll_into_view(&self, _smooth: bool) -> Result<(), PlatformError> {
        show(self.anchor.clone(), self.node_id, LAYOUT_TRIES);
        Ok(())
    }

    /// Blitz has no `FileList`, and nothing native posts a form anyway.
    fn set_files(&self, _files: &[dioxus::html::FileData]) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    /// Blitz implements no form reset, so each text control and box is written
    /// back to its markup default: what dioxus last wrote, the component's own
    /// value for a controlled one. A `<select>` never changes on input here.
    fn reset(&self) -> Result<(), PlatformError> {
        let form = self.node_id;
        self.command(move |doc| {
            let Ok(controls) = doc.query_selector_all_in(form, "input, textarea") else {
                return;
            };
            let defaults: Vec<_> = controls
                .into_iter()
                .filter_map(|id| {
                    let node = doc.get_node(id)?;
                    let element = node.element_data()?;
                    if element.checkbox_input_checked().is_some() {
                        let checked = checked_attr(element);
                        return Some((id, local_name!("checked"), checked.to_string()));
                    }
                    element.text_input_data()?;
                    let default = match &*element.name.local {
                        "textarea" => node.text_content(),
                        _ => element
                            .attr(local_name!("value"))
                            .unwrap_or_default()
                            .to_string(),
                    };
                    Some((id, local_name!("value"), default))
                })
                .collect();
            let mut mutator = doc.mutate();
            for (id, name, default) in defaults {
                mutator.set_attribute(id, QualName::new(None, ns!(), name), &default);
            }
        });
        Ok(())
    }

    /// Through the `value` attribute, as [`reset`](Self::reset) writes it:
    /// Blitz's text input follows it.
    fn set_value(&self, value: &str) -> Result<(), PlatformError> {
        let node_id = self.node_id;
        let value = value.to_string();
        self.command(move |doc| {
            let text = doc
                .get_node(node_id)
                .and_then(|node| node.element_data()?.text_input_data())
                .is_some();
            if text {
                let name = QualName::new(None, ns!(), local_name!("value"));
                doc.mutate().set_attribute(node_id, name, &value);
            }
        });
        Ok(())
    }

    fn attribute(&self, name: &str) -> Result<Option<String>, PlatformError> {
        let doc = self.anchor.try_doc().ok_or(PlatformError::Unsupported)?;
        let element = doc
            .get_node(self.node_id)
            .and_then(|node| node.element_data())
            .ok_or(PlatformError::NotFound)?;
        Ok(element
            .attrs()
            .iter()
            .find(|attr| attr.name.local.as_ref() == name)
            .map(|attr| attr.value.clone()))
    }

    /// A pre-order walk down to this node: everything it visits first comes
    /// before it, ancestors included, and its own subtree comes after.
    fn previous_focusable(
        &self,
        selector: &str,
    ) -> Result<Option<Box<dyn ElementApi>>, PlatformError> {
        let doc = self.anchor.try_doc().ok_or(PlatformError::Unsupported)?;
        let matches = doc
            .query_selector_all(selector)
            .map_err(|_| PlatformError::NotFound)?;
        let mut before = std::collections::HashSet::new();
        let mut stack = vec![doc.root_element().id];
        let mut found = false;
        while let Some(id) = stack.pop() {
            if id == self.node_id {
                found = true;
                break;
            }
            before.insert(id);
            if let Some(node) = doc.get_node(id) {
                stack.extend(node.children.iter().rev());
            }
        }
        drop(doc);
        if !found {
            return Err(PlatformError::NotFound);
        }
        let node_id = matches.into_iter().rev().find(|id| before.contains(id));
        Ok(node_id.map(|node_id| self.at(node_id)))
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

/// An effect runs before the shell lays out what it rendered, so a node
/// mounted this frame has no box yet: retried after a frame, a few times.
const LAYOUT_TRIES: u8 = 3;
const LAYOUT_WAIT: Duration = Duration::from_millis(20);

fn show(anchor: NodeHandle, node_id: NodeId, tries: u8) {
    let element = BlitzElement {
        anchor: anchor.clone(),
        node_id,
    };
    element.command(move |doc| {
        let unlaid = doc.get_node(node_id).is_some_and(|node| {
            let size = node.final_layout().size;
            size.width == 0.0 && size.height == 0.0
        });
        if unlaid && tries > 0 {
            let retry = super::thread::timer().map(|timer| {
                timer.after(
                    LAYOUT_WAIT,
                    Box::new(move || show(anchor, node_id, tries - 1)),
                )
            });
            // The latest call wins, as a second scroll would override the first.
            SHOW_RETRY.with(|slot| drop(slot.replace(retry)));
        } else if let Some((scroller, delta)) = into_view(doc, node_id) {
            doc.scroll_node_by(scroller, 0.0, -delta, |_| {});
            notify_scroll();
        }
    });
}

/// `x, y, width, height` of a node's border box in the viewport.
/// `get_client_bounding_rect` also subtracts the node's own scroll offset,
/// which moves its contents, not its box: added back here.
fn client_rect(doc: &BaseDocument, node_id: NodeId) -> Option<(f64, f64, f64, f64)> {
    let rect = doc.get_client_bounding_rect(node_id)?;
    let own = doc.get_node(node_id)?.scroll_offset();
    Some((rect.x + own.x, rect.y + own.y, rect.width, rect.height))
}

/// `WebElement::scroll_into_view`'s walk: the nearest ancestor that overflows
/// with `overflow-y: auto | scroll`, and how far it scrolls. No
/// `scroll-margin`: servo's stylo does not parse it.
fn into_view(doc: &BaseDocument, node_id: NodeId) -> Option<(NodeId, f64)> {
    let (_, y, width, height) = client_rect(doc, node_id)?;
    if width == 0.0 && height == 0.0 {
        return None;
    }
    let mut ancestor = doc.get_node(node_id)?.parent;
    let scroller = loop {
        let node = doc.get_node(ancestor?)?;
        if node.is_element()
            && node.final_layout().scroll_height() > 0.0
            && matches!(
                doc.resolved_style_value(node.id, "overflow-y").as_str(),
                "auto" | "scroll"
            )
        {
            break node;
        }
        ancestor = node.parent;
    };
    let layout = scroller.final_layout();
    let view_top = client_rect(doc, scroller.id)?.1 + f64::from(layout.border.top);
    let client_height = layout.size.height
        - layout.border.top
        - layout.border.bottom
        - layout.scrollbar_size.height;
    let delta =
        super::nearest_scroll(y, y + height, view_top, view_top + f64::from(client_height))?;
    Some((scroller.id, delta))
}
