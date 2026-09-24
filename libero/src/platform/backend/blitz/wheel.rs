//! Wheel latching, which Blitz lacks: a gesture stays on its first scroller
//! until the pointer moves or the wheel pauses, not a nested one sliding under.

use std::{cell::Cell, time::Instant};

use blitz_dom::BaseDocument;
use dioxus::{html::geometry::WheelDelta, prelude::*};
use dioxus_native_dom::NodeId;

use super::{ancestors, doc, resolved_style_value};

/// How long a pause ends a gesture (Chromium and Firefox: a few hundred ms).
const PAUSE_MS: u128 = 400;

/// Blitz's line height for a `Lines` delta.
const LINE: f64 = 20.0;

/// `None` for the viewport.
type Scroller = Option<NodeId>;

#[derive(Clone, Copy)]
struct Latch {
    scroller: Scroller,
    at: Instant,
    x: f64,
    y: f64,
}

/// One document's latch: a thread-wide one carried a gesture into the next
/// document wheeled at the same point (todo 1164).
#[derive(Default)]
pub(super) struct Watch {
    latch: Cell<Option<Latch>>,
}

/// A wheel reaching the wrapper, before Blitz's default scroll.
pub(super) fn wheeled(event: &Event<WheelData>) {
    let Some(state) = doc() else {
        return;
    };
    let latch = &state.wheel.latch;
    let point = event.client_coordinates();
    // Blitz hands on a finger's sign; flipped to the web's, positive scrolls down.
    let (dx, dy) = match event.delta() {
        WheelDelta::Pixels(delta) => (-delta.x, -delta.y),
        WheelDelta::Lines(delta) => (-delta.x * LINE, -delta.y * LINE),
        WheelDelta::Pages(_) => {
            latch.set(None);
            return;
        }
    };
    if !event.default_action_enabled() || (dx == 0.0 && dy == 0.0) {
        return;
    }
    let Some(anchor) = state.anchor() else {
        return;
    };
    let now = Instant::now();
    let latched = latch.get().filter(|latch| {
        latch.x == point.x
            && latch.y == point.y
            && now.duration_since(latch.at).as_millis() < PAUSE_MS
    });
    let (start, locked) = {
        let Some(doc) = anchor.try_doc() else {
            return;
        };
        (
            chain_start(&doc, doc.get_hover_node_id(), dx, dy),
            viewport_locked(&doc, dy != 0.0),
        )
    };
    let scroller = match latched {
        Some(latch) if latch.scroller != start && present(&anchor, latch.scroller) => {
            event.prevent_default();
            let mut doc = anchor.doc_mut();
            match latch.scroller {
                Some(node) => doc.scroll_node_by(node, -dx, -dy, |_| {}),
                None if !locked => doc.scroll_viewport_by(-dx, -dy),
                None => {}
            }
            drop(doc);
            latch.scroller
        }
        _ => {
            if start.is_none() && locked {
                event.prevent_default();
            }
            start
        }
    };
    latch.set(Some(Latch {
        scroller,
        at: now,
        x: point.x,
        y: point.y,
    }));
}

fn present(anchor: &dioxus_native_dom::NodeHandle, scroller: Scroller) -> bool {
    let Some(node) = scroller else {
        return true;
    };
    anchor.try_doc().is_some_and(|doc| {
        doc.get_node(node)
            .is_some_and(|node| node.flags.is_in_document())
    })
}

/// Where Blitz's scroll chain starts for this delta: the nearest box from
/// `hover` up that the user can scroll that way, else the viewport.
fn chain_start(doc: &BaseDocument, hover: Option<NodeId>, dx: f64, dy: f64) -> Scroller {
    let hover = hover?;
    ancestors(doc, hover).find(|&id| can_scroll(doc, id, dx, dy))
}

/// The viewport's propagated `overflow` is `hidden` or `clip` (a `Modal`'s
/// scroll lock), which Blitz ignores.
fn viewport_locked(doc: &BaseDocument, vertical: bool) -> bool {
    let axis = if vertical { "overflow-y" } else { "overflow-x" };
    let root = doc.root_element();
    let mut used = resolved_style_value(doc, root.id, axis);
    if used == "visible" {
        let body = root.children.iter().copied().find(|&id| {
            doc.get_node(id)
                .and_then(|node| node.element_data())
                .is_some_and(|element| &*element.name.local == "body")
        });
        if let Some(body) = body {
            used = resolved_style_value(doc, body, axis);
        }
    }
    matches!(used.as_str(), "hidden" | "clip")
}

fn can_scroll(doc: &BaseDocument, id: NodeId, dx: f64, dy: f64) -> bool {
    let Some(node) = doc.get_node(id).filter(|node| node.is_element()) else {
        return false;
    };
    let user = |axis: &str| {
        matches!(
            resolved_style_value(doc, id, axis).as_str(),
            "auto" | "scroll"
        )
    };
    let layout = node.final_layout();
    let offset = node.scroll_offset();
    let moves = |delta: f64, offset: f64, max: f32| {
        (delta > 0.0 && offset < f64::from(max)) || (delta < 0.0 && offset > 0.0)
    };
    (user("overflow-x") && moves(dx, offset.x, layout.scroll_width()))
        || (user("overflow-y") && moves(dy, offset.y, layout.scroll_height()))
}
