//! Blitz lays `position: sticky` out as `relative` (stylo_taffy). Every box
//! whose computed `position` is sticky is moved by a `transform` instead, kept
//! in its containing block, at each flush and after each scroll. Only a `top`
//! edge sticks.

use std::cell::Cell;

use blitz_dom::{BaseDocument, QualName, ns};
use dioxus_native_dom::NodeId;
use style::computed_values::position::T as Position;

use super::{anchor, resolved_style_value, run_or_defer, when_laid_out};

/// The shift last written, in px, so an unchanged one writes nothing.
const SHIFT_ATTR: &str = "data-lsx-sticky-shift";

thread_local! {
    static ARMED: Cell<bool> = const { Cell::new(false) };
}

/// Something scrolled: [`sync`] once the scroll is in the tree.
pub(super) fn sync_soon() {
    if ARMED.replace(true) {
        return;
    }
    when_laid_out(Box::new(|| {
        ARMED.set(false);
        if let Some(anchor) = anchor() {
            run_or_defer(&anchor, sync);
        }
    }));
}

/// Moves each sticky box to where its scroller's top edge holds it.
pub(super) fn sync(doc: &mut BaseDocument) {
    let written = |node: &blitz_dom::Node| {
        node.element_data()?
            .attrs
            .iter()
            .find(|attr| &*attr.name.local == SHIFT_ATTR)
            .and_then(|attr| attr.value.parse::<f32>().ok())
    };
    // A box no longer sticky (a breakpoint changed) is moved back too.
    let mut boxes = Vec::new();
    doc.visit(|id, node| {
        let sticky = node
            .primary_styles()
            .is_some_and(|styles| styles.clone_position() == Position::Sticky);
        if sticky || written(node).is_some_and(|shift| shift != 0.0) {
            boxes.push((id, sticky));
        }
    });
    let changes: Vec<(NodeId, f32)> = boxes
        .into_iter()
        .filter_map(|(id, sticky)| {
            let shift = match sticky {
                true => shift(doc, id)?,
                false => 0.0,
            };
            let written = written(doc.get_node(id)?).unwrap_or(0.0);
            // A re-render may have replaced the inline style under the mark.
            let lost = written != 0.0 && resolved_style_value(doc, id, "transform") == "none";
            (shift != written || lost).then_some((id, shift))
        })
        .collect();
    if changes.is_empty() {
        return;
    }
    let name = QualName::new(None, ns!(), SHIFT_ATTR.into());
    let mut mutator = doc.mutate();
    for (id, shift) in changes {
        mutator.set_attribute(id, name.clone(), &shift.to_string());
        match shift {
            0.0 => mutator.remove_style_property(id, "transform"),
            _ => mutator.set_style_property(id, "transform", &format!("translateY({shift}px)")),
        }
    }
}

/// How far below its place the box has to move, rounded to a pixel: the
/// scroller's top edge plus `top`, held to the parent's content box.
fn shift(doc: &BaseDocument, id: NodeId) -> Option<f32> {
    let node = doc.get_node(id)?;
    let top = px(&resolved_style_value(doc, id, "top"))?;
    let parent = doc.get_node(node.layout_parent.get()?)?;
    // Blitz applied `top` as a relative offset.
    let place = node.final_layout().location.y - top;
    let (mut y, mut edge) = (place, None);
    let mut current = Some(parent);
    while let Some(ancestor) = current {
        if scrolls(doc, ancestor.id) {
            let layout = ancestor.final_layout();
            edge = Some(ancestor.scroll_offset().y as f32 + layout.border.top);
            break;
        }
        y += ancestor.final_layout().location.y;
        current = ancestor.layout_parent.get().and_then(|id| doc.get_node(id));
    }
    let edge = edge.unwrap_or(doc.viewport_scroll().y as f32) + top;
    let layout = parent.final_layout();
    // A scroller holds the box over its whole content, not just its window.
    let height = match scrolls(doc, parent.id) {
        true => layout.size.height + layout.scroll_height(),
        false => layout.size.height,
    };
    let room = height
        - layout.padding.bottom
        - layout.border.bottom
        - (place + node.final_layout().size.height);
    Some((edge - y).clamp(0.0, room.max(0.0)).round() - top)
}

fn scrolls(doc: &BaseDocument, id: NodeId) -> bool {
    matches!(
        resolved_style_value(doc, id, "overflow-y").as_str(),
        "auto" | "scroll" | "hidden"
    )
}

fn px(value: &str) -> Option<f32> {
    value.strip_suffix("px")?.parse().ok()
}
