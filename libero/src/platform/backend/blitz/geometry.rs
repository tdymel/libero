use blitz_dom::{BaseDocument, Node};
use dioxus_native_dom::NodeId;

use super::sticky;

/// One axis of the padding box less the scrollbar, `clientWidth`/`clientHeight`.
pub(super) fn client_extent(size: f32, border: f32, bar: f32) -> f64 {
    f64::from((size - border - bar).max(0.0))
}

/// `x, y, width, height` of a node's border box in the viewport, its own scroll
/// offset added back: `get_client_bounding_rect` subtracts it.
pub(super) fn client_rect(doc: &BaseDocument, node_id: NodeId) -> Option<(f64, f64, f64, f64)> {
    if let Some(rect) = boxless_rect(doc, node_id) {
        return Some(rect);
    }
    if let Some(rect) = transformed_rect(doc, node_id) {
        return Some(rect);
    }
    let rect = doc.get_client_bounding_rect(node_id)?;
    let own = doc.get_node(node_id)?.scroll_offset();
    let (x, y) = pinned_scroll(doc, node_id);
    Some((
        rect.x + own.x + x,
        rect.y + own.y + y,
        rect.width,
        rect.height,
    ))
}

/// The viewport scroll Blitz's client rect takes off a box in a `fixed` child
/// of the root element, which Blitz paints unscrolled.
fn pinned_scroll(doc: &BaseDocument, id: NodeId) -> (f64, f64) {
    let root = doc.root_element().id;
    let mut current = doc.get_node(id);
    while let Some(node) = current {
        let block = node.containing_block();
        let fixed = node.primary_styles().is_some_and(|styles| {
            styles.clone_position() == style::computed_values::position::T::Fixed
        });
        // `sticky.rs` moved a z-indexed one back already.
        let shifted = node.element_data().is_some_and(|data| {
            data.attrs()
                .iter()
                .any(|attr| &*attr.name.local == sticky::SHIFT_ATTR && &*attr.value != "0 0")
        });
        if fixed && block == Some(root) && !shifted {
            let scroll = doc.viewport_scroll();
            return (scroll.x, scroll.y);
        }
        current = block.and_then(|id| doc.get_node(id));
    }
    (0.0, 0.0)
}

/// A table row or row group, which lays out no box in Blitz: the union of its
/// children's rects, moved cells included (todo 1520). `None` for any other node.
fn boxless_rect(doc: &BaseDocument, node_id: NodeId) -> Option<(f64, f64, f64, f64)> {
    let node = doc.get_node(node_id)?;
    let element = node.element_data()?;
    let size = node.final_layout().size;
    if !matches!(&*element.name.local, "tr" | "tbody" | "thead" | "tfoot")
        || size.width != 0.0
        || size.height != 0.0
    {
        return None;
    }
    let (left, top, right, bottom) = node
        .children
        .iter()
        .filter(|&&child| doc.get_node(child).is_some_and(Node::is_element))
        .filter_map(|&child| client_rect(doc, child))
        .filter(|&(_, _, width, height)| width > 0.0 || height > 0.0)
        .fold(
            (
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
            ),
            |(left, top, right, bottom), (x, y, width, height)| {
                (
                    left.min(x),
                    top.min(y),
                    right.max(x + width),
                    bottom.max(y + height),
                )
            },
        );
    (right >= left).then_some((left, top, right - left, bottom - top))
}

/// The rect with `transform`s, which Blitz's client rect leaves out: corners
/// mapped up the containing blocks. `None` when no box on the way has one.
fn transformed_rect(doc: &BaseDocument, node_id: NodeId) -> Option<(f64, f64, f64, f64)> {
    let node = doc.get_node(node_id)?;
    let size = node.unrounded_layout().size;
    let (width, height) = (f64::from(size.width), f64::from(size.height));
    let mut corners = [(0.0, 0.0), (width, 0.0), (0.0, height), (width, height)];
    // The cached transform is in device pixels: only its translation scales.
    let scale = doc.viewport().scale_f64();
    let mut transformed = false;
    let mut current = Some(node);
    while let Some(node) = current {
        let boxed = matches!(
            node.data,
            blitz_dom::NodeData::Element(_) | blitz_dom::NodeData::AnonymousBlock(_)
        );
        if let Some(transform) = boxed
            .then(|| node.transform().as_deref().copied())
            .flatten()
        {
            let [a, b, c, d, e, f] = transform.as_coeffs();
            for (x, y) in &mut corners {
                (*x, *y) = (a * *x + c * *y + e / scale, b * *x + d * *y + f / scale);
            }
            transformed = true;
        }
        let location = node.final_layout().location;
        let parent = node.containing_block().and_then(|id| doc.get_node(id));
        let scroll = parent.map(|parent| *parent.scroll_offset());
        for (x, y) in &mut corners {
            *x += f64::from(location.x) - scroll.map_or(0.0, |s| s.x);
            *y += f64::from(location.y) - scroll.map_or(0.0, |s| s.y);
        }
        current = parent;
    }
    if !transformed {
        return None;
    }
    let (scroll, pinned) = (doc.viewport_scroll(), pinned_scroll(doc, node_id));
    let xs = corners.map(|(x, _)| x - scroll.x + pinned.0);
    let ys = corners.map(|(_, y)| y - scroll.y + pinned.1);
    let min_x = xs.iter().copied().fold(f64::INFINITY, f64::min);
    let min_y = ys.iter().copied().fold(f64::INFINITY, f64::min);
    let max_x = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let max_y = ys.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    Some((min_x, min_y, max_x - min_x, max_y - min_y))
}
