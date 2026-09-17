//! Focus moves Blitz makes without a focus event: Tab and Shift+Tab, and
//! libero's own `focus()`/`blur()`. Each arms a check at [`Outlet`]'s next
//! flush, which runs once the move has landed and the document is free.
//!
//! [`Outlet`]: super::Outlet

use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

use blitz_dom::BaseDocument;
use dioxus::core::Runtime;
use dioxus::prelude::*;
use dioxus_native_dom::{NodeHandle, NodeId};

use super::{Doc, NEXT_CALLBACK, ancestors, anchor, defer, doc, flush_soon};
use crate::platform::{FocusMove, SilentFocusApi, SilentFocusSubscription, focus::OnMove};

/// Id, the subscriber's scope, callback. Every subscription sits in a hook, so
/// the scope outlives it.
type Callback = (u64, Option<ScopeId>, Rc<dyn Fn(&dyn FocusMove)>);

/// One document's silent-move watch.
#[derive(Default)]
pub(super) struct Watch {
    /// The focus owner when a silent move may have begun, while a check is armed.
    before: Cell<Option<Option<NodeId>>>,
    callbacks: RefCell<Vec<Callback>>,
}

thread_local! {
    /// Where libero's last `focus()` went, until the flush. See [`clicked`].
    static REQUESTED: Cell<Option<NodeId>> = const { Cell::new(None) };
    /// The focus owner at a press that cancelled its `mousedown`, until the
    /// press has ended, and whether it gets focus back. See [`mouse_pressed`].
    static KEPT: Cell<Option<(NodeId, bool)>> = const { Cell::new(None) };
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
        (owner != doc.root_element().id && Some(owner) != wrapper).then_some((owner, true))
    });
    KEPT.set(owner);
}

/// The press is released and Blitz moves focus next: back it goes, unless
/// libero moved it itself during the press.
pub(super) fn released() {
    let (Some((owner, restore)), Some(anchor)) = (KEPT.get(), anchor()) else {
        return;
    };
    defer(&anchor, move |doc| {
        KEPT.set(None);
        let present = doc
            .get_node(owner)
            .is_some_and(|node| node.flags.is_in_document());
        if restore && present && doc.get_focussed_node_id() != Some(owner) {
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

pub(super) fn requested(node_id: NodeId) {
    REQUESTED.set(Some(node_id));
    if let Some((owner, _)) = KEPT.get() {
        KEPT.set(Some((owner, false)));
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

/// A Tab reaching [`Listener`](super::Listener): Blitz moves focus after the
/// dispatch, unless a handler prevented it.
pub(super) fn keyed(event: &Event<KeyboardData>) {
    if event.key() != Key::Tab || !event.default_action_enabled() {
        return;
    }
    if let Some(doc) = anchor().as_ref().and_then(|anchor| anchor.try_doc()) {
        watch(&doc);
    }
}

/// Tells every subscriber when focus is no longer where it was armed.
pub(super) fn check(doc: &Doc) {
    REQUESTED.set(None);
    let Some(before) = doc.focus.before.take() else {
        return;
    };
    let Some(now) = doc
        .anchor()
        .as_ref()
        .and_then(|anchor| Some(anchor.try_doc()?.get_focussed_node_id()))
    else {
        return;
    };
    if now == before {
        return;
    }
    let callbacks: Vec<_> = doc
        .focus
        .callbacks
        .borrow()
        .iter()
        .map(|(_, scope, callback)| (*scope, callback.clone()))
        .collect();
    let moved = Moved { before, now };
    let runtime = Runtime::try_current();
    // In the subscriber's scope, not `Outlet`'s: what it reads is owned there.
    for (scope, callback) in callbacks {
        match (&runtime, scope) {
            (Some(runtime), Some(scope)) => runtime.in_scope(scope, || callback(&moved)),
            _ => callback(&moved),
        }
    }
}

struct Moved {
    before: Option<NodeId>,
    now: Option<NodeId>,
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
}

pub(in crate::platform::backend) fn silent_focus() -> Option<&'static dyn SilentFocusApi> {
    Some(&SILENT_FOCUS)
}

struct BlitzSilentFocus;

static SILENT_FOCUS: BlitzSilentFocus = BlitzSilentFocus;

impl SilentFocusApi for BlitzSilentFocus {
    fn on_move(&self, callback: OnMove) -> Box<dyn SilentFocusSubscription> {
        let id = NEXT_CALLBACK.replace(NEXT_CALLBACK.get() + 1);
        let doc = doc();
        if let Some(doc) = &doc {
            let scope = Runtime::try_current().and_then(|runtime| runtime.try_current_scope_id());
            doc.focus
                .callbacks
                .borrow_mut()
                .push((id, scope, Rc::from(callback)));
        }
        Box::new(Subscription(id, doc.as_ref().map(Rc::downgrade)))
    }
}

struct Subscription(u64, Option<Weak<Doc>>);

impl SilentFocusSubscription for Subscription {}

impl Drop for Subscription {
    fn drop(&mut self) {
        if let Some(doc) = self.1.as_ref().and_then(Weak::upgrade) {
            doc.focus
                .callbacks
                .borrow_mut()
                .retain(|(id, ..)| *id != self.0);
        }
    }
}
