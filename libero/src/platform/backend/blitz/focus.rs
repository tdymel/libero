//! Focus moves Blitz makes without a focus event (Tab, libero's `focus()` and
//! `blur()`), each checked at [`Outlet`]'s next flush.
//!
//! [`Outlet`]: super::Outlet

use std::{
    cell::Cell,
    rc::{Rc, Weak},
};

use blitz_dom::BaseDocument;
use dioxus::prelude::*;
use dioxus_native_dom::{NodeHandle, NodeId};

use super::{
    BlitzElement, Callbacks, Doc, ancestors, anchor, baked, defer, doc, flush_soon, reveal,
};
use crate::platform::{
    ElementApi, FocusMove, SilentFocusApi, SilentFocusSubscription, focus::OnMove,
};

type OnMoves = Callbacks<dyn Fn(&dyn FocusMove)>;

/// One document's silent-move watch.
pub(super) struct Watch {
    /// The focus owner when a silent move may have begun, while a check is armed.
    before: Cell<Option<Option<NodeId>>>,
    /// The armed move scrolls its target into view, as the web's Tab and
    /// `focus()` do; a press's focus does not.
    reveal: Cell<bool>,
    callbacks: OnMoves,
}

impl Default for Watch {
    fn default() -> Self {
        Self {
            before: Cell::new(None),
            reveal: Cell::new(false),
            callbacks: Callbacks::new(),
        }
    }
}

thread_local! {
    /// Where libero's last `focus()` went, until the flush. See [`clicked`].
    static REQUESTED: Cell<Option<NodeId>> = const { Cell::new(None) };
    /// Where focus goes back to after a press that cancelled its `mousedown`,
    /// until the press has ended. See [`mouse_pressed`].
    static KEPT: Cell<Option<NodeId>> = const { Cell::new(None) };
}

/// A `mousedown` reaching [`Listener`](super::Listener). Cancelled, the web
/// leaves focus where it is; Blitz moves it on the release all the same.
pub(super) fn mouse_pressed(event: &Event<MouseData>) {
    KEPT.set(None);
    if event.default_action_enabled() {
        return;
    }
    let Some(doc) = doc() else {
        return;
    };
    let wrapper = doc.wrapper_id();
    // `<html>` or the wrapper: nothing of the app's held focus to keep.
    let owner = doc.anchor().as_ref().and_then(|anchor| {
        let doc = anchor.try_doc()?;
        let owner = doc.get_focussed_node_id()?;
        (owner != doc.root_element().id && Some(owner) != wrapper).then_some(owner)
    });
    KEPT.set(owner);
}

/// The press is released and Blitz moves focus next: back it goes, or to where
/// libero moved it during the press.
pub(super) fn released() {
    let (Some(owner), Some(anchor)) = (KEPT.get(), anchor()) else {
        return;
    };
    defer(&anchor, move |doc| {
        KEPT.set(None);
        let present = doc
            .get_node(owner)
            .is_some_and(|node| node.flags.is_in_document());
        if present && doc.get_focussed_node_id() != Some(owner) {
            doc.set_focus_to(owner);
        }
    });
}

/// Whether a blur now is Blitz's move for a press that cancelled its
/// `mousedown`, which [`released`] undoes.
pub(in crate::platform::backend) fn press_kept_focus() -> bool {
    KEPT.get().is_some()
}

/// A key ends any press.
pub(super) fn forget_kept() {
    KEPT.set(None);
}

/// During a kept press, the release now puts focus back on `node_id`: a drag
/// sends no click for [`clicked`] to follow.
pub(super) fn requested(node_id: NodeId) {
    REQUESTED.set(Some(node_id));
    if KEPT.get().is_some() {
        KEPT.set(Some(node_id));
    }
}

/// A click has bubbled out, and Blitz focuses its target next (the web: before
/// the handlers). A `focus()` one of them made is made again after that.
pub(super) fn clicked() {
    let (Some(node_id), Some(anchor)) = (REQUESTED.take(), anchor()) else {
        return;
    };
    defer(&anchor, move |doc| {
        if doc
            .get_node(node_id)
            .is_some_and(|node| node.flags.is_in_document())
        {
            doc.set_focus_to(node_id);
        }
    });
}

/// Arms a check against where focus is now. The first arm since the last check
/// keeps its owner, so two moves in one poll compare against the start.
pub(super) fn watch(document: &BaseDocument) {
    let Some(doc) = doc() else {
        return;
    };
    if doc.focus.before.get().is_none() {
        doc.focus.before.set(Some(document.get_focussed_node_id()));
    }
    flush_soon();
}

/// [`watch`] for a move the web scrolls into view: Tab and `focus()`.
pub(super) fn watch_revealing(document: &BaseDocument) {
    watch(document);
    if let Some(doc) = doc() {
        doc.focus.reveal.set(true);
    }
}

/// A Tab reaching [`Listener`](super::Listener): Blitz moves focus after the
/// dispatch, unless a handler prevented it.
pub(super) fn keyed(event: &Event<KeyboardData>) {
    if event.key() != Key::Tab || !event.default_action_enabled() {
        return;
    }
    if let Some(doc) = anchor().as_ref().and_then(|anchor| anchor.try_doc()) {
        watch_revealing(&doc);
    }
}

/// Tells every subscriber when focus is no longer where it was armed.
pub(super) fn check(doc: &Doc) {
    REQUESTED.set(None);
    let revealing = doc.focus.reveal.replace(false);
    let Some(before) = doc.focus.before.take() else {
        return;
    };
    let Some(anchor) = doc.anchor() else {
        return;
    };
    let Some(now) = anchor.try_doc().map(|doc| doc.get_focussed_node_id()) else {
        return;
    };
    if now == before {
        return;
    }
    baked::check_soon();
    // Before the subscribers, which measure where it lands. `Outlet`'s mount
    // runs outside the document's borrow.
    if let Some(now) = now.filter(|&id| revealing && Some(id) != doc.wrapper_id()) {
        let mut document = anchor.doc_mut();
        if now != document.root_element().id {
            reveal(&mut document, now);
        }
    }
    let moved = Moved {
        before,
        now,
        wrapper: doc.wrapper_id(),
        anchor,
    };
    doc.focus.callbacks.each(|callback| callback(&moved));
}

struct Moved {
    before: Option<NodeId>,
    now: Option<NodeId>,
    wrapper: Option<NodeId>,
    anchor: NodeHandle,
}

/// A removed node has left the slab, so focus on it reads as outside.
fn holds(element: &Rc<MountedData>, focus: Option<NodeId>) -> bool {
    let (Some(focus), Some(handle)) = (focus, element.downcast::<NodeHandle>()) else {
        return false;
    };
    let target = handle.node_id();
    handle
        .try_doc()
        .is_some_and(|doc| ancestors(&doc, focus).any(|id| id == target))
}

impl FocusMove for Moved {
    fn was_in(&self, element: &Rc<MountedData>) -> bool {
        holds(element, self.before)
    }

    fn is_in(&self, element: &Rc<MountedData>) -> bool {
        holds(element, self.now)
    }

    /// As the web's `relatedTarget`: `<html>`, the wrapper or a removed node
    /// read as `<body>`.
    fn entered_from(&self, boundary: &str) -> Option<Option<Box<dyn ElementApi>>> {
        let doc = self.anchor.try_doc()?;
        let from = self.before.filter(|&id| {
            id != doc.root_element().id
                && Some(id) != self.wrapper
                && doc
                    .get_node(id)
                    .is_some_and(|node| node.flags.is_in_document())
        });
        let Some(from) = from else {
            return Some(None);
        };
        let hosts = doc.query_selector_all(boundary).ok()?;
        if ancestors(&doc, from).any(|id| hosts.contains(&id)) {
            return None;
        }
        drop(doc);
        Some(Some(Box::new(BlitzElement {
            anchor: self.anchor.clone(),
            node_id: from,
        })))
    }
}

pub(in crate::platform::backend) fn silent_focus() -> Option<&'static dyn SilentFocusApi> {
    Some(&SILENT_FOCUS)
}

struct BlitzSilentFocus;

static SILENT_FOCUS: BlitzSilentFocus = BlitzSilentFocus;

impl SilentFocusApi for BlitzSilentFocus {
    fn on_move(&self, callback: OnMove) -> Box<dyn SilentFocusSubscription> {
        let doc = doc();
        let id = doc
            .as_ref()
            .map(|doc| doc.focus.callbacks.add(Rc::from(callback)));
        Box::new(Subscription(id, doc.as_ref().map(Rc::downgrade)))
    }
}

struct Subscription(Option<u64>, Option<Weak<Doc>>);

impl SilentFocusSubscription for Subscription {}

impl Drop for Subscription {
    fn drop(&mut self) {
        if let (Some(id), Some(doc)) = (self.0, self.1.as_ref().and_then(Weak::upgrade)) {
            doc.focus.callbacks.remove(id);
        }
    }
}
