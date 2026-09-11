//! Focus moves Blitz makes without a focus event: Tab and Shift+Tab, and
//! libero's own `focus()`/`blur()`. Each arms a check at [`Outlet`]'s next
//! flush, which runs once the move has landed and the document is free.
//!
//! [`Outlet`]: super::Outlet

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use blitz_dom::BaseDocument;
use dioxus::prelude::*;
use dioxus_native_dom::{NodeHandle, NodeId};

use super::{NEXT_CALLBACK, ancestors, anchor, defer, flush_soon};
use crate::platform::{FocusMove, SilentFocusApi, SilentFocusSubscription, focus::OnMove};

type Callback = (u64, Rc<dyn Fn(&dyn FocusMove)>);

thread_local! {
    /// The focus owner when a silent move may have begun, while a check is armed.
    static BEFORE: Cell<Option<Option<NodeId>>> = const { Cell::new(None) };
    static CALLBACKS: RefCell<Vec<Callback>> = const { RefCell::new(Vec::new()) };
    /// Where libero's last `focus()` went, until the flush. See [`clicked`].
    static REQUESTED: Cell<Option<NodeId>> = const { Cell::new(None) };
}

pub(super) fn requested(node_id: NodeId) {
    REQUESTED.set(Some(node_id));
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
pub(super) fn watch(doc: &BaseDocument) {
    if BEFORE.get().is_none() {
        BEFORE.set(Some(doc.get_focussed_node_id()));
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
pub(super) fn check() {
    REQUESTED.set(None);
    let Some(before) = BEFORE.take() else {
        return;
    };
    let Some(now) = anchor()
        .as_ref()
        .and_then(|anchor| Some(anchor.try_doc()?.get_focussed_node_id()))
    else {
        return;
    };
    if now == before {
        return;
    }
    let callbacks: Vec<_> = CALLBACKS.with(|callbacks| {
        callbacks
            .borrow()
            .iter()
            .map(|(_, callback)| callback.clone())
            .collect()
    });
    let moved = Moved { before, now };
    for callback in callbacks {
        callback(&moved);
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
        CALLBACKS.with(|callbacks| callbacks.borrow_mut().push((id, Rc::from(callback))));
        Box::new(Subscription(id))
    }
}

struct Subscription(u64);

impl SilentFocusSubscription for Subscription {}

impl Drop for Subscription {
    fn drop(&mut self) {
        CALLBACKS.with(|callbacks| callbacks.borrow_mut().retain(|(id, _)| *id != self.0));
    }
}
