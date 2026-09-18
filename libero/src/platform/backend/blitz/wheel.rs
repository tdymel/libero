//! Wheel latching, which Blitz lacks: each tick scrolls from the hovered node,
//! so a nested scroller sliding under a still pointer takes the rest of the
//! turn. Browsers keep a gesture on the scroller it began with until the
//! pointer moves or the wheel pauses; here a later tick that Blitz would start
//! elsewhere is cancelled and scrolls the latched one instead.

use std::{cell::Cell, time::Instant};

use blitz_dom::BaseDocument;
use dioxus::{html::geometry::WheelDelta, prelude::*};
use dioxus_native_dom::NodeId;

use super::{ancestors, anchor, resolved_style_value};

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

thread_local! {
    static LATCH: Cell<Option<Latch>> = const { Cell::new(None) };
}

/// A wheel reaching the wrapper, before Blitz's default scroll.
pub(super) fn wheeled(event: &Event<WheelData>) {
    let point = event.client_coordinates();
    // Blitz hands on a finger's sign; flipped to the web's, positive scrolls down.
    let (dx, dy) = match event.delta() {
        WheelDelta::Pixels(delta) => (-delta.x, -delta.y),
        WheelDelta::Lines(delta) => (-delta.x * LINE, -delta.y * LINE),
        WheelDelta::Pages(_) => {
            LATCH.set(None);
            return;
        }
    };
    if !event.default_action_enabled() || (dx == 0.0 && dy == 0.0) {
        return;
    }
    let Some(anchor) = anchor() else {
        return;
    };
    let now = Instant::now();
    let latched = LATCH.get().filter(|latch| {
        latch.x == point.x
            && latch.y == point.y
            && now.duration_since(latch.at).as_millis() < PAUSE_MS
    });
    let start = {
        let Some(doc) = anchor.try_doc() else {
            return;
        };
        chain_start(&doc, doc.get_hover_node_id(), dx, dy)
    };
    let scroller = match latched {
        Some(latch) if latch.scroller != start && present(&anchor, latch.scroller) => {
            event.prevent_default();
            let mut doc = anchor.doc_mut();
            match latch.scroller {
                Some(node) => doc.scroll_node_by(node, -dx, -dy, |_| {}),
                None => doc.scroll_viewport_by(-dx, -dy),
            }
            drop(doc);
            latch.scroller
        }
        _ => start,
    };
    LATCH.set(Some(Latch {
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
