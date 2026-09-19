//! Tab after the focus owner is removed (todo 948): the web starts from where
//! it was, Blitz from the document start.

use std::cell::RefCell;

use blitz_dom::BaseDocument;
use dioxus::prelude::*;
use dioxus_native_dom::NodeId;

use super::{Doc, ancestors, anchor, doc, flush_soon, focus, refused};

thread_local! {
    /// The last known focus owner, and the document-order predecessor of it and
    /// of each ancestor, nearest first. See [`tab`].
    static OWNER: RefCell<Option<(NodeId, Vec<NodeId>)>> = const { RefCell::new(None) };
}

fn present(doc: &BaseDocument, node_id: NodeId) -> bool {
    doc.get_node(node_id)
        .is_some_and(|node| node.flags.is_in_document())
}

/// The node just before `node_id` in document order: its previous sibling's
/// last descendant, else its parent.
fn predecessor(doc: &BaseDocument, node_id: NodeId) -> Option<NodeId> {
    let parent = doc.get_node(doc.get_node(node_id)?.parent?)?;
    let at = parent.children.iter().position(|&id| id == node_id)?;
    let Some(mut last) = at.checked_sub(1).map(|at| parent.children[at]) else {
        return Some(parent.id);
    };
    while let Some(&child) = doc.get_node(last).and_then(|node| node.children.last()) {
        last = child;
    }
    Some(last)
}

/// Remembers `owner` as the focus owner, or forgets it (`None`).
pub(super) fn record(doc: &BaseDocument, owner: Option<NodeId>) {
    let entry = owner
        .filter(|&id| id != doc.root_element().id)
        .map(|owner| {
            let marks = ancestors(doc, owner)
                .filter_map(|id| predecessor(doc, id))
                .collect();
            (owner, marks)
        });
    // This poll's render may remove it: see [`check`].
    if entry.is_some() {
        flush_soon();
    }
    OWNER.set(entry);
}

/// A key reaching the wrapper: a Tab from a removed owner starts where it was,
/// then the owner now is remembered for the next key.
pub(super) fn keyed(event: &Event<KeyboardData>) {
    let Some(anchor) = anchor() else {
        return;
    };
    if event.key() == Key::Tab && event.default_action_enabled() {
        let next = {
            let Some(doc) = anchor.try_doc() else {
                return;
            };
            next_stop(&doc, event.modifiers().shift())
        };
        if let Some(next) = next {
            event.prevent_default();
            let mut doc = anchor.doc_mut();
            focus::watch(&doc);
            doc.set_focus_to(next);
            OWNER.set(None);
            return;
        }
    }
    let Some(doc) = anchor.try_doc() else {
        return;
    };
    let wrapper = self::doc().and_then(|doc| doc.wrapper_id());
    let owner = doc
        .get_focussed_node_id()
        .filter(|&id| Some(id) != wrapper && present(&doc, id));
    if owner.is_some() {
        record(&doc, owner);
    }
}

/// At [`Outlet`](super::Outlet)'s flush: the remembered owner was removed with
/// focus on it. Focus goes to the wrapper, as the web's to `<body>`, so the next
/// key reaches [`keyed`]; Blitz would send it to `<html>`.
pub(super) fn check(doc: &Doc) {
    let (Some(anchor), Some(wrapper)) = (doc.anchor(), doc.wrapper_id()) else {
        return;
    };
    let lost = {
        let Some(document) = anchor.try_doc() else {
            return;
        };
        let gone = OWNER.with_borrow(|owner| {
            owner
                .as_ref()
                .is_some_and(|(owner, _)| !present(&document, *owner))
        });
        // Blitz moves focus off a removed node to `<html>`.
        let root = document.root_element().id;
        gone && document
            .get_focussed_node_id()
            .is_none_or(|id| id == root || !present(&document, id))
    };
    if lost {
        let mut document = anchor.doc_mut();
        focus::watch(&document);
        document.set_focus_to(wrapper);
    }
}

/// Where a Tab (`back`: Shift+Tab) goes when the remembered owner is gone and
/// focus with it: on from the nearest mark still present.
fn next_stop(doc: &BaseDocument, back: bool) -> Option<NodeId> {
    let wrapper = self::doc().and_then(|doc| doc.wrapper_id());
    let focused = doc.get_focussed_node_id();
    let lost = focused
        .is_none_or(|id| id == doc.root_element().id || Some(id) == wrapper || !present(doc, id));
    if !lost {
        return None;
    }
    let mark = OWNER.with_borrow(|owner| {
        let (owner, marks) = owner.as_ref()?;
        if present(doc, *owner) {
            return None;
        }
        marks.iter().copied().find(|&id| present(doc, id))
    })?;
    let stop = |node: &blitz_dom::Node| node.is_focussable() && !refused::is_inert(doc, node.id);
    let start = doc.get_node(mark)?;
    if back {
        if stop(start) {
            return Some(mark);
        }
        doc.prev_node(start, stop)
    } else {
        doc.next_node(start, stop)
    }
}
