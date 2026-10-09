//! Presses and Tab stops the web refuses and Blitz does not (734): a disabled
//! control takes no click, an `inert` subtree neither a click nor focus.

use std::cell::Cell;

use blitz_dom::{BaseDocument, local_name};
use dioxus::prelude::*;
use dioxus_native_dom::NodeId;

use super::{
    BLANK_PRESS, activate, ancestors, anchor, defer, doc, focus, focusable_ancestor, forget_press,
    is_rendered,
};

thread_local! {
    /// The last press's page point. See [`release`].
    static PRESSED_AT: Cell<Option<(f64, f64)>> = const { Cell::new(None) };
}

pub(super) fn pressed(event: &Event<PointerData>) {
    let point = event.page_coordinates();
    PRESSED_AT.set(Some((point.x, point.y)));
}

pub(super) fn is_inert(doc: &BaseDocument, node_id: NodeId) -> bool {
    ancestors(doc, node_id).any(|id| {
        doc.get_node(id)
            .and_then(|node| node.element_data())
            .is_some_and(|element| element.attr(local_name!("inert")).is_some())
    })
}

fn refuses_press(doc: &BaseDocument, node_id: NodeId) -> bool {
    doc.get_node(node_id)
        .and_then(|node| node.element_data())
        .is_some_and(|element| {
            let control = [
                local_name!("button"),
                local_name!("input"),
                local_name!("select"),
                local_name!("textarea"),
            ]
            .contains(&element.name.local);
            element.attr(local_name!("inert")).is_some()
                || (control && element.attr(local_name!("disabled")).is_some())
        })
}

/// The outermost node from `node_id` up that refuses a press.
fn refusing(doc: &BaseDocument, node_id: NodeId) -> Option<NodeId> {
    ancestors(doc, node_id)
        .filter(|&id| refuses_press(doc, id))
        .last()
}

/// Where a press on `node_id` counts from: outside a disabled control or an
/// `inert` subtree holding it.
pub(super) fn press_origin(doc: &BaseDocument, node_id: NodeId) -> Option<NodeId> {
    match refusing(doc, node_id) {
        Some(outer) => doc.get_node(outer)?.parent,
        None => Some(node_id),
    }
}

/// Prevents a release on a disabled control or under `inert`, within 2px of the
/// press only: a prevented release would leave Blitz's drag running.
pub(super) fn release(event: &Event<PointerData>) {
    let point = event.page_coordinates();
    let still = PRESSED_AT
        .take()
        .is_some_and(|(x, y)| (point.x - x).abs() <= 2.0 && (point.y - y).abs() <= 2.0);
    let Some(anchor) = anchor().filter(|_| still) else {
        return;
    };
    let wrapper = doc().and_then(|doc| doc.wrapper_id());
    let (before, target) = {
        let Some(doc) = anchor.try_doc() else {
            return;
        };
        let Some(hit) = doc.hit(point.x as f32, point.y as f32) else {
            return;
        };
        if refusing(&doc, hit.node_id).is_none() {
            return;
        }
        let target =
            focusable_ancestor(&doc, hit.node_id).filter(|&target| Some(target) != wrapper);
        (doc.get_focussed_node_id(), target)
    };
    event.prevent_default();
    forget_press();
    BLANK_PRESS.set(false);
    // A press that cancelled its `mousedown` keeps focus where it is.
    if focus::press_kept_focus() {
        return;
    }
    match (target, wrapper) {
        (Some(target), _) => activate::focus_pressed(before, target),
        // The web's press on nothing focusable: focus leaves for `<body>`.
        (None, Some(wrapper)) => defer(&anchor, move |doc| {
            if doc.get_focussed_node_id() == before && before != Some(wrapper) {
                focus::watch(doc);
                doc.set_focus_to(wrapper);
            }
        }),
        (None, None) => {}
    }
}

/// Tab or Shift+Tab whose next stop Blitz would find under `inert` or not rendered
/// (`visibility: hidden`, `display: none`): prevented, and focus goes to the next one outside it.
pub(super) fn tab(event: &Event<KeyboardData>) {
    if event.key() != Key::Tab || !event.default_action_enabled() {
        return;
    }
    let Some(anchor) = anchor() else {
        return;
    };
    let back = event.modifiers().shift();
    let next = {
        let Some(doc) = anchor.try_doc() else {
            return;
        };
        let Some(start) = doc.get_focussed_node_id().and_then(|id| doc.get_node(id)) else {
            return;
        };
        let step = |filter: &dyn Fn(NodeId) -> bool| {
            let filter = |node: &blitz_dom::Node| node.is_focussable() && filter(node.id);
            if back {
                doc.prev_node(start, filter)
            } else {
                doc.next_node(start, filter)
            }
        };
        let refused = |id| is_inert(&doc, id) || !is_rendered(&doc, id);
        let blitz = step(&|_| true);
        if !blitz.is_some_and(refused) {
            return;
        }
        step(&|id| !refused(id))
    };
    event.prevent_default();
    let Some(next) = next else {
        return;
    };
    let mut doc = anchor.doc_mut();
    focus::watch(&doc);
    doc.set_focus_to(next);
}
