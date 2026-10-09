//! `position: sticky` and `fixed`, which Blitz lays out as `relative` and
//! `absolute`, done by `transform` at each flush and scroll. The window size is
//! polled in `resize.rs`: Blitz reports no resize.

use std::cell::Cell;
use std::collections::HashMap;

use blitz_dom::{BaseDocument, QualName, ns};
use dioxus_native_dom::NodeId;
use style::computed_values::position::T as Position;
use style::values::computed::{Length, LengthPercentage, position::Inset};
use style::values::generics::position::GenericInset;

use super::{
    PORTAL_ROOT_ATTR, anchor, node_is_rtl, resize, resolved_style_value, run_or_defer,
    when_laid_out,
};

/// The shift last written, `"x y"` in px, so an unchanged one writes nothing.
const SHIFT_ATTR: &str = "data-lsx-sticky-shift";

/// The inline `width` and `height` last written to a `fixed` box, `-` for one left alone.
const SIZE_ATTR: &str = "data-lsx-fixed-size";

/// A `fixed` box's inline size per axis, `None` where its own style sizes it.
type Size = [Option<f32>; 2];

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

/// Moves each sticky box to where its scroller's edges hold it, and each `fixed`
/// box onto the window.
pub(super) fn sync(doc: &mut BaseDocument) {
    // A box no longer sticky or fixed (a breakpoint changed) is moved and sized back too.
    let mut boxes = Vec::new();
    doc.visit(|id, node| {
        let position = node
            .primary_styles()
            .map(|styles| styles.clone_position())
            .filter(|position| matches!(position, Position::Sticky | Position::Fixed));
        let moved = written(node).is_some_and(|shift| shift != [0.0; 2]);
        if position.is_some() || moved || written_size(node) != [None; 2] {
            boxes.push((id, position));
        }
    });
    resize::watch_placed(doc, boxes.iter().any(|(_, position)| position.is_some()));
    // In tree order: a `fixed` box reads the shifts of the boxes it sits in.
    let mut shifts = HashMap::new();
    let (mut changes, mut resized) = (Vec::new(), Vec::new());
    for (id, position) in boxes {
        let placed = match position {
            Some(Position::Sticky) => shift(doc, id).map(|shift| (shift, [None; 2])),
            Some(_) => fixed(doc, id, &shifts),
            None => Some(([0.0; 2], [None; 2])),
        };
        let (Some((shift, size)), Some(node)) = (placed, doc.get_node(id)) else {
            continue;
        };
        let written = written(node).unwrap_or([0.0; 2]);
        shifts.insert(id, shift);
        // A re-render may have replaced the inline style under the mark.
        let lost = written != [0.0; 2] && resolved_style_value(doc, id, "transform") == "none";
        if shift != written || lost {
            changes.push((id, shift));
        }
        let had = written_size(node);
        let lost = ["width", "height"]
            .into_iter()
            .zip(had)
            .any(|(property, had)| {
                had.is_some() && resolved_style_value(doc, id, property) == "auto"
            });
        if size != had || lost {
            resized.push((id, had, size));
        }
    }
    if changes.is_empty() && resized.is_empty() {
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
    let name = QualName::new(None, ns!(), SIZE_ATTR.into());
    for (id, had, size) in resized {
        for ((property, had), size) in ["width", "height"].into_iter().zip(had).zip(size) {
            match (had, size) {
                (_, Some(px)) => mutator.set_style_property(id, property, &format!("{px}px")),
                // Only an inline size the shim wrote goes.
                (Some(_), None) => mutator.remove_style_property(id, property),
                (None, None) => {}
            }
        }
        match size == [None; 2] {
            true => mutator.clear_attribute(id, name.clone()),
            false => {
                let [width, height] = size.map(|px| px.map_or("-".to_owned(), |px| px.to_string()));
                mutator.set_attribute(id, name.clone(), &format!("{width} {height}"));
            }
        }
    }
}

/// The inline size last written to `node`, `[None; 2]` if none.
fn written_size(node: &blitz_dom::Node) -> Size {
    let Some((width, height)) = attribute(node, SIZE_ATTR).and_then(|size| size.split_once(' '))
    else {
        return [None; 2];
    };
    [width.parse().ok(), height.parse().ok()]
}

/// The shift last written to `node`.
fn written(node: &blitz_dom::Node) -> Option<[f32; 2]> {
    let (x, y) = attribute(node, SHIFT_ATTR)?.split_once(' ')?;
    Some([x.parse::<f32>().ok()?, y.parse::<f32>().ok()?])
}

fn attribute<'a>(node: &'a blitz_dom::Node, name: &str) -> Option<&'a str> {
    let attrs = &node.element_data()?.attrs;
    let attr = attrs.iter().find(|attr| &*attr.name.local == name)?;
    Some(&attr.value)
}

/// How far a `fixed` box has to move to sit at its insets in the window, which
/// Blitz resolved against the parent box (`absolute`), and its size on an axis
/// with both insets and an `auto` size. Left alone inside the portal outlet (on
/// the viewport already), under a transformed or filtered ancestor (its
/// containing block, as on the web) or with a transform of its own.
fn fixed(
    doc: &BaseDocument,
    id: NodeId,
    shifts: &HashMap<NodeId, [f32; 2]>,
) -> Option<([f32; 2], Size)> {
    const ALONE: ([f32; 2], Size) = ([0.0; 2], [None; 2]);
    let node = doc.get_node(id)?;
    let styles = node.primary_styles()?;
    let position = styles.get_position();
    let ours = written(node).is_some_and(|shift| shift != [0.0; 2]);
    if !ours && resolved_style_value(doc, id, "transform") != "none" {
        return Some(ALONE);
    }
    let own = node.final_layout();
    // Its painted place in the document, and how far scrolling moved it.
    let (mut at, mut scrolled) = ([own.location.x, own.location.y], [0.0_f32; 2]);
    let mut current = node.layout_parent.get().and_then(|id| doc.get_node(id));
    while let Some(ancestor) = current {
        if attribute(ancestor, PORTAL_ROOT_ATTR).is_some() {
            return Some(ALONE);
        }
        let shift = shifts.get(&ancestor.id).copied();
        let transformed = ["transform", "filter"]
            .into_iter()
            .any(|property| resolved_style_value(doc, ancestor.id, property) != "none");
        if shift.is_none() && transformed {
            return Some(ALONE);
        }
        let ([x, y], location) = (shift.unwrap_or([0.0; 2]), ancestor.final_layout().location);
        let offset = ancestor.scroll_offset();
        at = [at[0] + location.x + x, at[1] + location.y + y];
        scrolled = [scrolled[0] + offset.x as f32, scrolled[1] + offset.y as f32];
        current = ancestor.layout_parent.get().and_then(|id| doc.get_node(id));
    }
    let (scroll, scale) = (doc.viewport_scroll(), doc.viewport().scale_f64());
    let scrolled = [scrolled[0] + scroll.x as f32, scrolled[1] + scroll.y as f32];
    let (width, height) = doc.viewport().window_size;
    let window = [
        (width as f64 / scale) as f32,
        (height as f64 / scale) as f32,
    ];
    let (margin, padding, border) = (own.margin, own.padding, own.border);
    let axes = [
        (
            [&position.left, &position.right],
            [margin.left, margin.right],
            own.size.width,
            padding.left + padding.right + border.left + border.right,
        ),
        (
            [&position.top, &position.bottom],
            [margin.top, margin.bottom],
            own.size.height,
            padding.top + padding.bottom + border.top + border.bottom,
        ),
    ];
    let rtl = node
        .layout_parent
        .get()
        .is_some_and(|parent| node_is_rtl(doc, parent));
    let border_box = resolved_style_value(doc, id, "box-sizing") == "border-box";
    let had = written_size(node);
    let (mut shift, mut sized) = ([0.0; 2], [None; 2]);
    for (axis, ([start, end], [before, after], size, edges)) in axes.into_iter().enumerate() {
        let (start, end) = (inset(start), inset(end));
        let basis = Length::new(window[axis]);
        let property = ["width", "height"][axis];
        let auto = had[axis].is_some() || resolved_style_value(doc, id, property) == "auto";
        // Both insets and no size of its own: it spans the window between them.
        if let (Some(start), Some(end), true) = (start, end, auto) {
            let outer = window[axis] - start.resolve(basis).px() - end.resolve(basis).px();
            let outer = (outer - before - after).max(0.0);
            sized[axis] = Some(match border_box {
                true => outer,
                false => (outer - edges).max(0.0),
            });
        }
        // Over-constrained, the containing block's direction picks the edge.
        let start = start.filter(|_| sized[axis].is_some() || !(axis == 0 && rtl && end.is_some()));
        let place = at[axis] - scrolled[axis];
        shift[axis] = match (start, end) {
            (Some(start), _) => start.resolve(basis).px() + before - place,
            (None, Some(end)) => window[axis] - end.resolve(basis).px() - after - size - place,
            // No inset: its static place, which no scroll moves.
            (None, None) => scrolled[axis],
        }
        .round();
    }
    Some((shift, sized))
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
    // Taffy resolves a relative percentage only against a definite size.
    let definite = [
        true,
        resolved_style_value(doc, parent.id, "height") != "auto",
    ];
    let rtl = node_is_rtl(doc, parent.id);
    let mut shift = [0.0; 2];
    for (axis, ([start, end], [lo, hi], [before, after], size)) in axes.into_iter().enumerate() {
        let (start, end) = (inset(start), inset(end));
        let basis = [outer.size.width, outer.size.height][axis] - lo - hi;
        let relative = |value: &LengthPercentage| match definite[axis] {
            true => Some(value.resolve(Length::new(basis)).px()),
            false => (!value.has_percentage()).then(|| value.resolve(Length::new(0.0)).px()),
        };
        let (start_offset, end_offset) = (
            start.and_then(relative),
            end.and_then(relative).map(|end| -end),
        );
        // Blitz applied the insets as a relative offset.
        let applied = match axis == 0 && rtl {
            true => end_offset.or(start_offset),
            false => start_offset.or(end_offset),
        }
        .unwrap_or(0.0);
        let place = [own.location.x, own.location.y][axis] - applied;
        let at = at[axis] - applied;
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
        shift[axis] = moved.clamp(-back, ahead).round() - applied;
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
