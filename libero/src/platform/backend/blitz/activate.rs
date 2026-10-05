//! Activation crates.io Blitz leaves out: a click made from code, a key that
//! clicks, focus on a clicked element, and a focus ring for keys only.

use std::{
    any::Any,
    borrow::Cow,
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
    time::{Duration, Instant},
};

use blitz_dom::{
    BaseDocument, DocGuard, DocGuardMut, Document, EventDriver, EventHandler, Node, QualName,
    local_name, ns,
};
use blitz_traits::events::{DomEvent, DomEventData, EventState};
use dioxus::{
    core::{ElementId, Runtime},
    html::PlatformEventData,
    prelude::*,
};
use dioxus_native_dom::{NodeHandle, NodeId, synthetic_click_event};

use super::{anchor, defer, focus, later};
use crate::platform::keyboard::takes_typing;

thread_local! {
    /// The node a Space press armed, until its release. See [`key_up`].
    static SPACE: Cell<Option<NodeId>> = const { Cell::new(None) };
    /// Whether the last input was a key: `:focus-visible` then matches.
    static KEYBOARD: Cell<bool> = const { Cell::new(false) };
    /// The nodes [`sync_marks`] last marked, per attribute.
    static MARKED: Cell<[(&'static str, Vec<NodeId>); 2]> =
        const { Cell::new([(WITHIN, Vec::new()), (VISIBLE, Vec::new())]) };
    /// The last press, where and when, and Blitz's click count for it.
    static PRESSES: Cell<Option<(Instant, f64, f64, u32)>> = const { Cell::new(None) };
    /// Whether [`takes_over`] fired the last press's `dblclick`.
    static DOUBLED: Cell<bool> = const { Cell::new(false) };
    /// Whether the last press's `pointerdown` was cancelled. See [`press_cancelled`].
    static CANCELLED: Cell<bool> = const { Cell::new(false) };
    /// The last press's data, counted once however many wrappers it bubbles through.
    static PRESS_DATA: RefCell<Option<Weak<PlatformEventData>>> = const { RefCell::new(None) };
}

/// Clicks `node_id` at the end of this poll through Blitz's event driver:
/// handlers, then the default action. A disabled element takes none.
pub(super) fn click(anchor: &NodeHandle, node_id: NodeId) {
    let anchor = anchor.clone();
    later(move || {
        let Some(runtime) = Runtime::try_current() else {
            return;
        };
        let data = {
            let doc = anchor.doc();
            let Some(node) = doc
                .get_node(node_id)
                .filter(|node| node.flags.is_in_document())
            else {
                return;
            };
            if node
                .element_data()
                .is_some_and(|element| element.attr(local_name!("disabled")).is_some())
            {
                return;
            }
            node.synthetic_click_event_data(Modifiers::empty())
        };
        let mut doc = Shared(anchor.clone());
        EventDriver::new(&mut doc, Forward(runtime))
            .handle_dom_event(DomEvent::new(node_id, DomEventData::Click(data)));
    });
}

/// The shell's view of the document: borrowed per call, so a handler finds it free.
struct Shared(NodeHandle);

impl Document for Shared {
    fn inner(&self) -> DocGuard<'_> {
        DocGuard::RefCell(self.0.doc())
    }

    fn inner_mut(&mut self) -> DocGuardMut<'_> {
        DocGuardMut::RefCell(self.0.doc_mut())
    }
}

/// Whether nothing holds focus: Blitz then answers `<html>`.
fn cleared(doc: &BaseDocument) -> bool {
    doc.get_focussed_node_id() == doc.try_root_element().map(|root| root.id)
}

/// dioxus-native's own handler, for clicks only: the one event data it lets
/// others build.
struct Forward(Rc<Runtime>);

impl EventHandler for Forward {
    fn handle_event(
        &mut self,
        chain: &[NodeId],
        event: &mut DomEvent,
        doc: &mut dyn Document,
        state: &mut EventState,
    ) {
        if !matches!(event.data, DomEventData::Click(_)) {
            return;
        }
        let (data, id) = {
            let doc = doc.inner();
            let Some(node) = doc.get_node(event.target) else {
                return;
            };
            let id = chain
                .iter()
                .find_map(|&id| doc.get_node(id).and_then(dioxus_id));
            (synthetic_click_event(node, Modifiers::empty()), id)
        };
        let Some(id) = id else {
            return;
        };
        let data: Rc<dyn Any> = Rc::new(PlatformEventData::new(data));
        let click = Event::new(data, event.bubbles);
        self.0.handle_event("click", click.clone(), id);
        // Blitz's default action would only clear the focus the handlers left.
        if !click.default_action_enabled() || !blitz_acts(&doc.inner(), event.target) {
            state.prevent_default();
        }
        if !click.propagates() {
            state.stop_propagation();
        }
    }
}

/// Whether Blitz's click default action does more than clear focus: its walk
/// up from `target` in `handle_click`.
fn blitz_acts(doc: &BaseDocument, target: NodeId) -> bool {
    let mut next = Some(target);
    while let Some(node) = next.and_then(|id| doc.get_node(id)) {
        next = node.parent;
        let Some(element) = node.element_data() else {
            continue;
        };
        let kind = element.attr(local_name!("type"));
        let acts = element.attr(local_name!("disabled")).is_some()
            || element.text_input_data().is_some()
            || match &*element.name.local {
                "input" => matches!(kind, Some("checkbox" | "radio")),
                "summary" => node
                    .parent
                    .and_then(|id| doc.get_node(id))
                    .is_some_and(|parent| {
                        parent
                            .data
                            .is_element_with_tag_name(&local_name!("details"))
                    }),
                "label" => doc.label_bound_input_element(node.id).is_some(),
                "a" => element.attr(local_name!("href")).is_some(),
                _ => false,
            };
        if acts {
            return true;
        }
    }
    false
}

/// The vdom's id for a node, which dioxus-native writes as an attribute.
fn dioxus_id(node: &Node) -> Option<ElementId> {
    node.element_data()?
        .attrs
        .iter()
        .find(|attr| &*attr.name.local == "data-dioxus-id")?
        .value
        .parse()
        .ok()
        .map(ElementId::from_raw)
}

/// Which key clicks the focused element on the web: Enter on its press, Space
/// on its release.
fn activation(doc: &BaseDocument, node_id: NodeId) -> (bool, bool) {
    let Some(element) = doc.get_node(node_id).and_then(Node::element_data) else {
        return (false, false);
    };
    let kind = element
        .attr(local_name!("type"))
        .unwrap_or_default()
        .to_ascii_lowercase();
    match &*element.name.local {
        "button" | "summary" => (true, true),
        "input" => match kind.as_str() {
            "button" | "submit" | "reset" | "image" => (true, true),
            "checkbox" | "radio" => (false, true),
            _ => (false, false),
        },
        "a" => (element.attr(local_name!("href")).is_some(), false),
        _ => (false, false),
    }
}

fn is_space(event: &Event<KeyboardData>) -> bool {
    event.code() == Code::Space || event.key() == Key::Character(" ".into())
}

/// A key press bubbled out of the app unprevented: Enter clicks now, Space arms.
pub(super) fn key_down(event: &Event<KeyboardData>) {
    KEYBOARD.set(true);
    if is_space(event) {
        SPACE.set(None);
    }
    let modified = event
        .modifiers()
        .intersects(Modifiers::CONTROL | Modifiers::ALT | Modifiers::META | Modifiers::SUPER);
    if !event.default_action_enabled() || event.is_composing() || modified {
        return;
    }
    let Some(anchor) = anchor() else {
        return;
    };
    let Some((focused, (enter, space))) = anchor.try_doc().and_then(|doc| {
        let focused = doc.get_focussed_node_id()?;
        Some((focused, activation(&doc, focused)))
    }) else {
        return;
    };
    if enter && event.key() == Key::Enter {
        click(&anchor, focused);
    } else if space && is_space(event) {
        SPACE.set(Some(focused));
    }
}

/// Space released where it was pressed, unprevented: a click.
pub(super) fn key_up(event: &Event<KeyboardData>) {
    if !is_space(event) {
        return;
    }
    let (Some(armed), Some(anchor)) = (SPACE.take(), anchor()) else {
        return;
    };
    let unmoved = anchor
        .try_doc()
        .is_some_and(|doc| doc.get_focussed_node_id() == Some(armed));
    if event.default_action_enabled() && unmoved {
        click(&anchor, armed);
    }
}

/// A press: from here the focus ring stays off, as the web's `:focus-visible`.
/// Counted as Blitz counts clicks, within 500ms and 2px of the last, and when
/// cancelled too, as on the web: Blitz then skips its count.
/// `raw` is the press's own data: a nested provider's wrapper hears it again (todo 2246).
pub(super) fn pointer_down(event: &Event<PointerData>, raw: &Rc<PlatformEventData>) {
    KEYBOARD.set(false);
    SPACE.set(None);
    CANCELLED.set(!event.default_action_enabled());
    let seen = PRESS_DATA
        .replace(Some(Rc::downgrade(raw)))
        .is_some_and(|last| last.ptr_eq(&Rc::downgrade(raw)));
    if seen {
        return;
    }
    let at = event.client_coordinates();
    let now = Instant::now();
    DOUBLED.set(false);
    let count = match PRESSES.get() {
        Some((last, x, y, count))
            if now - last < Duration::from_millis(500)
                && (at.x - x).abs() <= 2.0
                && (at.y - y).abs() <= 2.0 =>
        {
            count + 1
        }
        _ => 1,
    };
    PRESSES.set(Some((now, at.x, at.y, count)));
}

/// Whether the last press was cancelled: Blitz's click default would then count
/// its click from an older press, and the web moves no focus for it.
pub(super) fn press_cancelled() -> bool {
    CANCELLED.get()
}

/// Prevents a bubbled click whose Blitz default would only clear focus, so
/// focus moves to the target as on the web. `raw` is the click's own data.
pub(super) fn takes_over(
    event: &Event<MouseData>,
    raw: &Event<PlatformEventData>,
    hit: Option<NodeId>,
) -> bool {
    let prevented = !event.default_action_enabled();
    let takes = !prevented
        && !anchor()
            .zip(hit)
            .is_none_or(|(anchor, hit)| anchor.try_doc().is_none_or(|doc| blitz_acts(&doc, hit)));
    if takes {
        event.prevent_default();
    } // Blitz fires `dblclick` from its click default, which no longer runs (todo 1413).
    // Once per press: a nested provider's wrapper hears the click too.
    if (prevented || takes)
        && PRESSES.get().is_some_and(|press| press.3 == 2)
        && !DOUBLED.replace(true)
        && let Some(hit) = hit
    {
        double(raw.data.clone(), hit);
    }
    takes
}

/// Fires the `dblclick` Blitz's click default would have, with the click's data, at the flush.
fn double(data: Rc<PlatformEventData>, hit: NodeId) {
    let Some(anchor) = anchor() else {
        return;
    };
    later(move || {
        let Some(runtime) = Runtime::try_current() else {
            return;
        };
        let id = {
            let doc = anchor.doc();
            doc.get_node(hit)
                .filter(|node| node.flags.is_in_document())
                .and_then(|_| {
                    std::iter::successors(Some(hit), |&id| doc.get_node(id)?.parent)
                        .find_map(|id| doc.get_node(id).and_then(dioxus_id))
                })
        };
        if let Some(id) = id {
            runtime.handle_event("dblclick", Event::new(data as Rc<dyn Any>, true), id);
        }
    });
}

/// Focus moves to a pressed `target` at the flush, unless something moved it
/// since the press (`before`).
pub(super) fn focus_pressed(before: Option<NodeId>, target: NodeId) {
    let Some(anchor) = anchor() else {
        return;
    };
    defer(&anchor, move |doc| {
        let present = doc
            .get_node(target)
            .is_some_and(|node| node.flags.is_in_document());
        let focused = doc.get_focussed_node_id();
        if present && (focused == before || cleared(doc)) && focused != Some(target) {
            focus::watch(doc);
            doc.set_focus_to(target);
            // The press cancelled its `mousedown`: the release keeps this focus.
            focus::requested(target);
        }
    });
}

/// Focuses a clicked `target` as the web does, where Blitz cleared focus or,
/// for a link, left it unmoved since the press (`before`).
pub(super) fn clicked(before: Option<NodeId>, target: NodeId) {
    let Some(anchor) = anchor() else {
        return;
    };
    defer(&anchor, move |doc| {
        let Some(node) = doc.get_node(target) else {
            return;
        };
        let link = node.element_data().is_some_and(|element| {
            *element.name.local == *"a" && element.attr(local_name!("href")).is_some()
        });
        let unmoved = link && doc.get_focussed_node_id() == before;
        if node.flags.is_in_document() && (cleared(doc) || unmoved) {
            focus::watch(doc);
            doc.set_focus_to(target);
        }
    });
}

const VISIBLE: &str = "data-lsx-focus-visible";
const WITHIN: &str = "data-lsx-focus-within";

/// `:focus-visible` and `:focus-within` as attributes: Blitz's stylo matches
/// neither, and [`sync_marks`] keeps these.
pub(in crate::platform::backend) fn focus_selectors(css: &str) -> Cow<'_, str> {
    if !css.contains(":focus-") {
        return Cow::Borrowed(css);
    }
    Cow::Owned(
        css.replace(":focus-visible", &format!("[{VISIBLE}]"))
            .replace(":focus-within", &format!("[{WITHIN}]")),
    )
}

/// Marks the focused element and its ancestors, and rings the focused one as
/// the web's `:focus-visible` does: a text field always, anything else after a key.
pub(super) fn sync_marks(doc: &mut BaseDocument) {
    // Nothing held: `<html>`, or the wrapper standing in for it.
    let nothing = [
        doc.try_root_element().map(|root| root.id),
        super::doc().and_then(|doc| doc.wrapper_id()),
    ];
    let focused = doc
        .get_focussed_node_id()
        .filter(|&id| !nothing.contains(&Some(id)));
    let within: Vec<NodeId> = focused
        .map(|id| {
            std::iter::successors(Some(id), |&id| doc.get_node(id)?.parent)
                .filter(|&id| doc.get_node(id).is_some_and(Node::is_element))
                .collect()
        })
        .unwrap_or_default();
    let visible = focused.filter(|&id| {
        KEYBOARD.get()
            || doc
                .get_node(id)
                .and_then(Node::element_data)
                .is_some_and(|element| {
                    let tag = element.name.local.to_ascii_uppercase();
                    takes_typing(&tag, element.attr(local_name!("type")))
                })
    });
    let marks = [(WITHIN, within), (VISIBLE, visible.into_iter().collect())];
    let old = MARKED.take();
    // Read back rather than trusted: a removed node's id may be reused.
    let has = |doc: &BaseDocument, id: NodeId, name: &str| {
        doc.get_node(id)
            .and_then(Node::element_data)
            .is_some_and(|element| element.attrs.iter().any(|attr| &*attr.name.local == name))
    };
    let mut changes = Vec::new();
    for ((name, want), (_, had)) in marks.iter().zip(old.iter()) {
        changes.extend(
            had.iter()
                .filter(|id| !want.contains(id) && has(doc, **id, name))
                .map(|&id| (id, *name, false)),
        );
        changes.extend(
            want.iter()
                .filter(|&&id| !has(doc, id, name))
                .map(|&id| (id, *name, true)),
        );
    }
    MARKED.set(marks.map(|(name, ids)| (name, ids)));
    if changes.is_empty() {
        return;
    }
    let mut mutator = doc.mutate();
    for (id, name, set) in changes {
        let name = QualName::new(None, ns!(), name.into());
        if set {
            mutator.set_attribute(id, name, "");
        } else {
            mutator.clear_attribute(id, name);
        }
    }
}
