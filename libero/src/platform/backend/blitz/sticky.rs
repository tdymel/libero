//! `position: sticky`, which Blitz lays out in flow, done by `transform` at each
//! flush and scroll, and a z-indexed `fixed` box, which Blitz paints and hit-tests
//! scrolled. The window size is polled in `resize.rs`: Blitz reports no resize.

use std::cell::Cell;

use blitz_dom::{BaseDocument, QualName, ns};
use dioxus_native_dom::NodeId;
use style::computed_values::position::T as Position;
use style::values::computed::{Length, LengthPercentage, position::Inset};
use style::values::generics::position::{GenericInset, GenericZIndex};

use super::{anchor, resize, resolved_style_value, run_or_defer, when_laid_out};

/// The shift last written, `"x y"` in px, so an unchanged one writes nothing.
pub(super) const SHIFT_ATTR: &str = "data-lsx-sticky-shift";

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

/// Moves each sticky box to where its scroller's edges hold it, and each
/// z-indexed `fixed` box back by the page's scroll.
pub(super) fn sync(doc: &mut BaseDocument) {
    // A box no longer placed (a breakpoint changed) is moved back too.
    let root = doc.root_element().id;
    let mut boxes = Vec::new();
    doc.visit(|id, node| {
        let position = node.primary_styles().and_then(|styles| {
            match styles.clone_position() {
                Position::Sticky => Some(Position::Sticky),
                // Blitz hoists a non-zero `z-index` and cancels no scroll for it.
                Position::Fixed
                    if node.containing_block() == Some(root)
                        && matches!(styles.clone_z_index(), GenericZIndex::Integer(z) if z != 0) =>
                {
                    Some(Position::Fixed)
                }
                _ => None,
            }
        });
        let moved = written(node).is_some_and(|shift| shift != [0.0; 2]);
        if position.is_some() || moved {
            boxes.push((id, position));
        }
    });
    resize::watch_placed(
        doc,
        boxes
            .iter()
            .any(|&(_, position)| position == Some(Position::Sticky)),
    );
    let scroll = doc.viewport_scroll();
    let mut changes = Vec::new();
    for (id, position) in boxes {
        let shift = match position {
            Some(Position::Sticky) => shift(doc, id),
            Some(_) => pinned(doc, id, [scroll.x as f32, scroll.y as f32]),
            None => Some([0.0; 2]),
        };
        let (Some(shift), Some(node)) = (shift, doc.get_node(id)) else {
            continue;
        };
        let written = written(node).unwrap_or([0.0; 2]);
        // A re-render may have replaced the inline style under the mark.
        let lost = written != [0.0; 2] && resolved_style_value(doc, id, "transform") == "none";
        if shift != written || lost {
            changes.push((id, shift));
        }
    }
    if changes.is_empty() {
        return;
    }
    let name = QualName::new(None, ns!(), SHIFT_ATTR.into());
    let mut mutator = doc.mutate();
    for (id, [x, y]) in changes {
        mutator.set_attribute(id, name.clone(), &format!("{x} {y}"));
        match [x, y] == [0.0; 2] {
            true => mutator.remove_style_property(id, "transform"),
            false => {
                mutator.set_style_property(id, "transform", &format!("translate({x}px, {y}px)"))
            }
        }
    }
}

/// A `fixed` box's shift, unless it has a transform of its own.
fn pinned(doc: &BaseDocument, id: NodeId, scroll: [f32; 2]) -> Option<[f32; 2]> {
    let ours = written(doc.get_node(id)?).is_some_and(|shift| shift != [0.0; 2]);
    match ours || resolved_style_value(doc, id, "transform") == "none" {
        true => Some(scroll),
        false => Some([0.0; 2]),
    }
}

/// The shift last written to `node`.
fn written(node: &blitz_dom::Node) -> Option<[f32; 2]> {
    let attrs = &node.element_data()?.attrs;
    let attr = attrs.iter().find(|attr| &*attr.name.local == SHIFT_ATTR)?;
    let (x, y) = attr.value.split_once(' ')?;
    Some([x.parse::<f32>().ok()?, y.parse::<f32>().ok()?])
}

/// How far the box has to move on each axis, rounded to a pixel: held by its
/// insets inside the scroller's window, and inside the parent's content box.
fn shift(doc: &BaseDocument, id: NodeId) -> Option<[f32; 2]> {
    let node = doc.get_node(id)?;
    let styles = node.primary_styles()?;
    let position = styles.get_position();
    let parent = doc.get_node(node.layout_parent.get()?)?;
    let own = node.final_layout();
    let (mut at, mut scroller) = ([own.location.x, own.location.y], None);
    let mut current = Some(parent);
    while let Some(ancestor) = current {
        if scrolls(doc, ancestor.id) {
            scroller = Some(ancestor);
            break;
        }
        let location = ancestor.final_layout().location;
        at = [at[0] + location.x, at[1] + location.y];
        current = ancestor.layout_parent.get().and_then(|id| doc.get_node(id));
    }
    let (edge, window) = match scroller {
        Some(scroller) => {
            let (layout, offset) = (scroller.final_layout(), scroller.scroll_offset());
            let (border, bar) = (layout.border, layout.scrollbar_size);
            (
                [offset.x as f32 + border.left, offset.y as f32 + border.top],
                [
                    layout.size.width - border.left - border.right - bar.width,
                    layout.size.height - border.top - border.bottom - bar.height,
                ],
            )
        }
        None => {
            let (scroll, scale) = (doc.viewport_scroll(), doc.viewport().scale_f64());
            let (width, height) = doc.viewport().window_size;
            (
                [scroll.x as f32, scroll.y as f32],
                [
                    (width as f64 / scale) as f32,
                    (height as f64 / scale) as f32,
                ],
            )
        }
    };
    let outer = parent.final_layout();
    let (border, padding, margin) = (outer.border, outer.padding, own.margin);
    // A scroller holds the box over its whole content, not just its window.
    let extent = match scroller.is_some_and(|scroller| scroller.id == parent.id) {
        true => [
            outer.size.width + outer.scroll_width(),
            outer.size.height + outer.scroll_height(),
        ],
        false => [outer.size.width, outer.size.height],
    };
    let axes = [
        (
            [&position.left, &position.right],
            [border.left + padding.left, border.right + padding.right],
            [margin.left, margin.right],
            own.size.width,
        ),
        (
            [&position.top, &position.bottom],
            [border.top + padding.top, border.bottom + padding.bottom],
            [margin.top, margin.bottom],
            own.size.height,
        ),
    ];
    let mut shift = [0.0; 2];
    for (axis, ([start, end], [lo, hi], [before, after], size)) in axes.into_iter().enumerate() {
        let (start, end) = (inset(start), inset(end));
        // Taffy 0.15 lays a sticky box out in flow, its insets unapplied.
        let (place, at) = ([own.location.x, own.location.y][axis], at[axis]);
        let mut moved = 0.0_f32;
        if let Some(end) = end {
            let limit = edge[axis] + window[axis] - end.resolve(Length::new(window[axis])).px();
            moved = moved.min(limit - (at + size));
        }
        if let Some(start) = start {
            let limit = edge[axis] + start.resolve(Length::new(window[axis])).px();
            moved = moved.max(limit - at);
        }
        let back = (place - before - lo).max(0.0);
        let ahead = (extent[axis] - hi - (place + size + after)).max(0.0);
        shift[axis] = moved.clamp(-back, ahead).round();
    }
    Some(shift)
}

/// The inset's length or percentage, `None` for `auto`.
fn inset(value: &Inset) -> Option<&LengthPercentage> {
    match value {
        GenericInset::LengthPercentage(value) => Some(value),
        _ => None,
    }
}

fn scrolls(doc: &BaseDocument, id: NodeId) -> bool {
    matches!(
        resolved_style_value(doc, id, "overflow-y").as_str(),
        "auto" | "scroll" | "hidden"
    )
}
