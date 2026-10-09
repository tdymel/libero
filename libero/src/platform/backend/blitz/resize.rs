//! Blitz reports no `resize`, mutation, intersection or window resize: watched
//! nodes and the window are compared at each [`Outlet`] flush, after each press
//! or key, after a scroll (intersections), and every [`POLL`] while any is watched.
//!
//! [`Outlet`]: super::Outlet

use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
    time::Duration,
};

use blitz_dom::BaseDocument;
use dioxus::prelude::*;
use dioxus_native_dom::{NodeHandle, NodeId};
use style::values::computed::Overflow;

use super::{Doc, client_rect, doc, resolved_style_value, when_free, when_laid_out};
use crate::platform::{
    ContentSubscription, Dimensions, ScrollSubscription, TimerSubscription,
    backend::{origin::Origin, thread},
    resize::measured_resize,
};

/// How late a change no press or key preceded is seen at most: a window
/// resize, a timer's render, a picture that loaded.
pub(super) const POLL: Duration = Duration::from_millis(500);

/// After a press or key: past the frame that lays out what it rendered.
const SETTLE: Duration = Duration::from_millis(30);

enum Kind {
    /// The border box, as `ResizeObserver` reports it.
    Resize(Rc<dyn Fn(Event<ResizeData>)>),
    /// The scroll size, where a subtree change shows.
    Content(Rc<dyn Fn()>),
    /// The visible share inside a root, as `IntersectionObserver` reports it.
    Intersect(Box<Intersect>),
}

struct Intersect {
    root: Option<NodeId>,
    /// Top, right, bottom, left.
    margin: [Margin; 4],
    /// Sorted.
    thresholds: Vec<f64>,
    /// Intersecting, then how many thresholds the ratio reached.
    last: Option<(bool, usize)>,
    callback: Rc<dyn Fn(bool, f64)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Margin {
    Px(f64),
    Percent(f64),
}

struct ViewportWatcher {
    id: u64,
    origin: Origin,
    callback: Rc<dyn Fn()>,
}

struct Watched {
    id: u64,
    anchor: NodeHandle,
    node_id: NodeId,
    last: Option<(f32, f32)>,
    origin: Origin,
    kind: Kind,
}

/// One document's watched nodes.
#[derive(Default)]
pub(super) struct Watch {
    watched: RefCell<Vec<Watched>>,
    next: Cell<u64>,
    viewport: RefCell<Vec<ViewportWatcher>>,
    /// The window's size and scale at the last check.
    window: Cell<Option<((u32, u32), f64)>>,
    poll: RefCell<Option<Box<dyn TimerSubscription>>>,
    settle: RefCell<Option<Box<dyn TimerSubscription>>>,
    scrolled: Cell<bool>,
}

pub(in crate::platform::backend) fn on_resize(
    mounted: &Rc<MountedData>,
    callback: Box<dyn Fn(Event<ResizeData>)>,
) -> Option<Box<dyn ContentSubscription>> {
    watch(mounted, Kind::Resize(Rc::from(callback)))
}

pub(in crate::platform::backend) fn on_content_change(
    mounted: &Rc<MountedData>,
    callback: Box<dyn Fn()>,
) -> Option<Box<dyn ContentSubscription>> {
    watch(mounted, Kind::Content(Rc::from(callback)))
}

pub(in crate::platform::backend) fn on_intersection(
    target: &Rc<MountedData>,
    root: Option<&Rc<MountedData>>,
    root_margin: &str,
    thresholds: &[f64],
    callback: Box<dyn Fn(bool, f64)>,
) -> Option<Box<dyn ContentSubscription>> {
    let root = match root {
        Some(root) => Some(root.downcast::<NodeHandle>()?.node_id()),
        None => None,
    };
    let mut thresholds = thresholds.to_vec();
    thresholds.sort_by(f64::total_cmp);
    let intersect = Intersect {
        root,
        margin: parse_margin(root_margin),
        thresholds,
        last: None,
        callback: Rc::from(callback),
    };
    watch(target, Kind::Intersect(Box::new(intersect)))
}

/// Polled like the rest: Blitz reports no window resize.
pub(in crate::platform::backend) fn on_viewport_resize(
    callback: Box<dyn Fn()>,
) -> Option<Box<dyn ScrollSubscription>> {
    let doc = doc()?;
    let id = doc.resize.next.replace(doc.resize.next.get() + 1);
    if doc.resize.window.get().is_none() {
        doc.resize.window.set(window(&doc));
    }
    doc.resize.viewport.borrow_mut().push(ViewportWatcher {
        id,
        origin: Origin::here(),
        callback: Rc::from(callback),
    });
    poll(&doc);
    Some(Box::new(Subscription(id, Rc::downgrade(&doc))))
}

fn watch(mounted: &Rc<MountedData>, kind: Kind) -> Option<Box<dyn ContentSubscription>> {
    let anchor = mounted.downcast::<NodeHandle>()?.clone();
    let doc = doc()?;
    let id = doc.resize.next.replace(doc.resize.next.get() + 1);
    doc.resize.watched.borrow_mut().push(Watched {
        id,
        node_id: anchor.node_id(),
        anchor,
        last: None,
        origin: Origin::here(),
        kind,
    });
    poll(&doc);
    // The first measure, once laid out.
    settle_soon(&doc);
    Some(Box::new(Subscription(id, Rc::downgrade(&doc))))
}

/// Starts the [`POLL`] unless it runs; [`Subscription`]'s drop stops it.
fn poll(doc: &Rc<Doc>) {
    let mut poll = doc.resize.poll.borrow_mut();
    if poll.is_none() {
        let weak = Rc::downgrade(doc);
        *poll = thread::timer().map(|timer| {
            timer.every(
                POLL,
                Box::new(move || {
                    if let Some(doc) = weak.upgrade() {
                        check(&doc);
                    }
                }),
            )
        });
    }
}

/// The window's size and scale, `None` while the document is borrowed.
fn window(doc: &Doc) -> Option<((u32, u32), f64)> {
    let anchor = doc.anchor()?;
    let base = anchor.try_doc()?;
    Some((base.viewport().window_size, base.viewport().scale_f64()))
}

struct Subscription(u64, Weak<Doc>);

impl ContentSubscription for Subscription {}

impl ScrollSubscription for Subscription {}

impl Drop for Subscription {
    fn drop(&mut self) {
        let Some(doc) = self.1.upgrade() else {
            return;
        };
        let empty = {
            let mut watched = doc.resize.watched.borrow_mut();
            let mut viewport = doc.resize.viewport.borrow_mut();
            watched.retain(|watched| watched.id != self.0);
            viewport.retain(|watcher| watcher.id != self.0);
            watched.is_empty() && viewport.is_empty()
        };
        if empty {
            // Taken out first, so no borrow is held across their drop.
            let timers = (
                doc.resize.poll.borrow_mut().take(),
                doc.resize.settle.borrow_mut().take(),
            );
            drop(timers);
        }
    }
}

/// A press or a key reached the wrapper: whatever it renders is measured once
/// laid out. A later one restarts the wait.
pub(super) fn pressed() {
    if let Some(doc) = doc() {
        settle_soon(&doc);
    }
}

/// Something scrolled: intersections are measured once the scroll is laid out.
pub(super) fn scrolled() {
    let Some(doc) = doc() else {
        return;
    };
    let watching = doc
        .resize
        .watched
        .borrow()
        .iter()
        .any(|watched| matches!(watched.kind, Kind::Intersect(_)));
    if !watching || doc.resize.scrolled.replace(true) {
        return;
    }
    let weak = Rc::downgrade(&doc);
    when_laid_out(Box::new(move || {
        if let Some(doc) = weak.upgrade() {
            doc.resize.scrolled.set(false);
            check(&doc);
        }
    }));
}

fn settle_soon(doc: &Rc<Doc>) {
    if doc.resize.watched.borrow().is_empty() {
        return;
    }
    let weak = Rc::downgrade(doc);
    let wait = thread::timer().map(|timer| {
        timer.after(
            SETTLE,
            Box::new(move || {
                if let Some(doc) = weak.upgrade() {
                    when_free(Box::new(move || check(&doc)));
                }
            }),
        )
    });
    drop(doc.resize.settle.replace(wait));
}

enum Fire {
    Resize(Rc<dyn Fn(Event<ResizeData>)>, Dimensions, Dimensions),
    Content(Rc<dyn Fn()>),
    Intersect(Rc<dyn Fn(bool, f64)>, bool, f64),
    Viewport(Rc<dyn Fn()>),
}

/// Measures every watched node and the window and calls those that changed,
/// each in its own scope. A document still borrowed is left for the next check.
pub(super) fn check(doc: &Doc) {
    let mut fired: Vec<(Origin, Fire)> = Vec::new();
    let resized = window(doc).is_some_and(|now| {
        doc.resize
            .window
            .replace(Some(now))
            .is_some_and(|before| before != now)
    });
    if resized {
        let viewport = doc.resize.viewport.borrow();
        fired.extend(
            viewport
                .iter()
                .map(|watcher| (watcher.origin, Fire::Viewport(watcher.callback.clone()))),
        );
    }
    let measured: Vec<(Origin, Fire)> = 'measured: {
        let mut watched = doc.resize.watched.borrow_mut();
        let Some(anchor) = watched.first().map(|watched| watched.anchor.clone()) else {
            break 'measured Vec::new();
        };
        let Some(base) = anchor.try_doc() else {
            break 'measured Vec::new();
        };
        watched
            .iter_mut()
            .filter_map(|watched| {
                if let Kind::Intersect(intersect) = &mut watched.kind {
                    let node = base.get_node(watched.node_id)?;
                    if !node.flags.is_in_document() {
                        return None;
                    }
                    let (is_intersecting, ratio) = intersection(&base, watched.node_id, intersect);
                    let reached = intersect
                        .thresholds
                        .iter()
                        .take_while(|&&at| at <= ratio)
                        .count();
                    if intersect.last.replace((is_intersecting, reached))
                        == Some((is_intersecting, reached))
                    {
                        return None;
                    }
                    let fire = Fire::Intersect(intersect.callback.clone(), is_intersecting, ratio);
                    return Some((watched.origin, fire));
                }
                let (size, border, content) = measure(&base, watched)?;
                let previous = watched.last.replace(size);
                if previous == Some(size) {
                    return None;
                }
                let first = previous.is_none();
                let fire = match &watched.kind {
                    Kind::Resize(callback) => Fire::Resize(callback.clone(), border, content),
                    // A `MutationObserver` reports nothing on observe.
                    Kind::Content(_) if first => return None,
                    Kind::Content(callback) => Fire::Content(callback.clone()),
                    Kind::Intersect(_) => return None,
                };
                Some((watched.origin, fire))
            })
            .collect()
    };
    fired.extend(measured);
    run(fired);
}

fn run(fired: Vec<(Origin, Fire)>) {
    for (origin, fire) in fired {
        origin.run(|| match fire {
            Fire::Resize(callback, border, content) => callback(measured_resize(border, content)),
            Fire::Content(callback) | Fire::Viewport(callback) => callback(),
            Fire::Intersect(callback, is_intersecting, ratio) => callback(is_intersecting, ratio),
        });
    }
}

/// `(is_intersecting, ratio)` of `target` inside its root's padding box grown by
/// the margin, clipped by every scroller or `overflow` clip in between.
fn intersection(doc: &BaseDocument, target: NodeId, intersect: &Intersect) -> (bool, f64) {
    let Some((x, y, width, height)) = client_rect(doc, target) else {
        return (false, 0.0);
    };
    let mut visible = Some([x, y, x + width, y + height]);
    let mut current = doc.get_node(target);
    while let Some(node) = current {
        let Some(style) = node.primary_styles() else {
            break;
        };
        if style.get_box().display.is_none() {
            return (false, 0.0);
        }
        // Clips above the root do not count, as in `IntersectionObserver`.
        if Some(node.id) == intersect.root {
            break;
        }
        if node.id != target {
            let (clips_x, clips_y) = (
                style.get_box().overflow_x != Overflow::Visible,
                style.get_box().overflow_y != Overflow::Visible,
            );
            if (clips_x || clips_y)
                && let Some(padding) = padding_box(doc, node.id)
            {
                let clip = [
                    if clips_x { padding[0] } else { f64::MIN },
                    if clips_y { padding[1] } else { f64::MIN },
                    if clips_x { padding[2] } else { f64::MAX },
                    if clips_y { padding[3] } else { f64::MAX },
                ];
                visible = visible.and_then(|rect| overlap(rect, clip));
            }
        }
        current = node.parent.and_then(|parent| doc.get_node(parent));
    }
    let root = match intersect.root {
        Some(root) => padding_box(doc, root),
        None => {
            let scale = doc.viewport().scale_f64();
            let (width, height) = doc.viewport().window_size;
            Some([
                0.0,
                0.0,
                f64::from(width) / scale,
                f64::from(height) / scale,
            ])
        }
    };
    let Some(root) = root else {
        return (false, 0.0);
    };
    let root = grow(root, &intersect.margin);
    let Some(seen) = visible.and_then(|rect| overlap(rect, root)) else {
        return (false, 0.0);
    };
    let area = width * height;
    let ratio = if area > 0.0 {
        ((seen[2] - seen[0]) * (seen[3] - seen[1]) / area).clamp(0.0, 1.0)
    } else {
        1.0
    };
    (true, ratio)
}

/// `[left, top, right, bottom]` of the box inside its border and scrollbars.
fn padding_box(doc: &BaseDocument, id: NodeId) -> Option<[f64; 4]> {
    let (x, y, width, height) = client_rect(doc, id)?;
    let layout = doc.get_node(id)?.final_layout();
    let (border, bar) = (layout.border, layout.scrollbar_size);
    Some([
        x + f64::from(border.left),
        y + f64::from(border.top),
        x + width - f64::from(border.right + bar.width),
        y + height - f64::from(border.bottom + bar.height),
    ])
}

/// The overlap of two `[left, top, right, bottom]` rects; touching edges count.
fn overlap(a: [f64; 4], b: [f64; 4]) -> Option<[f64; 4]> {
    let rect = [
        a[0].max(b[0]),
        a[1].max(b[1]),
        a[2].min(b[2]),
        a[3].min(b[3]),
    ];
    (rect[0] <= rect[2] && rect[1] <= rect[3]).then_some(rect)
}

fn grow(rect: [f64; 4], margin: &[Margin; 4]) -> [f64; 4] {
    let (width, height) = (rect[2] - rect[0], rect[3] - rect[1]);
    let px = |margin: Margin, of: f64| match margin {
        Margin::Px(px) => px,
        Margin::Percent(percent) => percent / 100.0 * of,
    };
    [
        rect[0] - px(margin[3], width),
        rect[1] - px(margin[0], height),
        rect[2] + px(margin[1], width),
        rect[3] + px(margin[2], height),
    ]
}

/// A `rootMargin`: one to four `px` or `%` lengths in the `margin` order. One
/// that does not parse is no margin.
fn parse_margin(margin: &str) -> [Margin; 4] {
    let lengths: Option<Vec<Margin>> = margin
        .split_whitespace()
        .map(|length| {
            if let Some(px) = length.strip_suffix("px") {
                return px.parse().ok().map(Margin::Px);
            }
            if let Some(percent) = length.strip_suffix('%') {
                return percent.parse().ok().map(Margin::Percent);
            }
            length
                .parse::<f64>()
                .ok()
                .filter(|&zero| zero == 0.0)
                .map(Margin::Px)
        })
        .collect();
    match lengths.as_deref() {
        Some(&[all]) => [all; 4],
        Some(&[block, inline]) => [block, inline, block, inline],
        Some(&[top, inline, bottom]) => [top, inline, bottom, inline],
        Some(&[top, right, bottom, left]) => [top, right, bottom, left],
        _ => [Margin::Px(0.0); 4],
    }
}

/// What is compared, then the border and content boxes. `None` for a node
/// gone from the document.
fn measure(doc: &BaseDocument, watched: &Watched) -> Option<((f32, f32), Dimensions, Dimensions)> {
    let node = doc.get_node(watched.node_id)?;
    if !node.flags.is_in_document() {
        return None;
    }
    let node = match watched.kind {
        // No box of its own: the parent's scroll size holds its content.
        Kind::Content(_) if resolved_style_value(doc, node.id, "display") == "contents" => {
            doc.get_node(node.parent?)?
        }
        _ => node,
    };
    let layout = node.final_layout();
    let (size, padding, border) = (layout.size, layout.padding, layout.border);
    let dimensions = |width: f32, height: f32| Dimensions {
        width: f64::from(width),
        height: f64::from(height),
    };
    if let Kind::Resize(_) = watched.kind
        && size.width == 0.0
        && size.height == 0.0
        && let Some((width, height)) = boxless_size(doc, node.id)
    {
        let outer = dimensions(width, height);
        return Some(((width, height), outer, outer));
    }
    let outer = dimensions(size.width, size.height);
    let inner = dimensions(
        (size.width - padding.left - padding.right - border.left - border.right).max(0.0),
        (size.height - padding.top - padding.bottom - border.top - border.bottom).max(0.0),
    );
    let compared = match watched.kind {
        Kind::Resize(_) => (size.width, size.height),
        Kind::Content(_) => (
            size.width + layout.scroll_width(),
            size.height + layout.scroll_height(),
        ),
        Kind::Intersect(_) => return None,
    };
    Some((compared, outer, inner))
}

/// The union of the boxes under `id`, which lays out none itself: a table row
/// or row group in Blitz. `None` when nothing under it has a box.
fn boxless_size(doc: &BaseDocument, id: NodeId) -> Option<(f32, f32)> {
    let (mut start, mut end) = ((f64::MAX, f64::MAX), (f64::MIN, f64::MIN));
    let mut stack = vec![id];
    while let Some(id) = stack.pop() {
        let Some(node) = doc.get_node(id) else {
            continue;
        };
        for &child in &node.children {
            let Some(child_node) = doc.get_node(child).filter(|node| node.is_element()) else {
                continue;
            };
            let size = child_node.final_layout().size;
            if size.width == 0.0 && size.height == 0.0 {
                stack.push(child);
                continue;
            }
            if let Some(rect) = doc.get_client_bounding_rect(child) {
                start = (start.0.min(rect.x), start.1.min(rect.y));
                end = (
                    end.0.max(rect.x + rect.width),
                    end.1.max(rect.y + rect.height),
                );
            }
        }
    }
    (end.0 >= start.0).then_some(((end.0 - start.0) as f32, (end.1 - start.1) as f32))
}

#[cfg(test)]
mod tests {
    use super::{Margin, grow, overlap, parse_margin};

    #[test]
    fn a_root_margin_takes_one_to_four_lengths() {
        assert_eq!(parse_margin("0px"), [Margin::Px(0.0); 4]);
        assert_eq!(
            parse_margin("100px 10%"),
            [
                Margin::Px(100.0),
                Margin::Percent(10.0),
                Margin::Px(100.0),
                Margin::Percent(10.0)
            ]
        );
        assert_eq!(
            parse_margin("1px 2px 3px"),
            [
                Margin::Px(1.0),
                Margin::Px(2.0),
                Margin::Px(3.0),
                Margin::Px(2.0)
            ]
        );
        assert_eq!(parse_margin("1em"), [Margin::Px(0.0); 4]);
    }

    #[test]
    fn a_margin_grows_the_root_and_touching_edges_overlap() {
        let root = grow([0.0, 0.0, 100.0, 200.0], &parse_margin("10px 50%"));
        assert_eq!(root, [-50.0, -10.0, 150.0, 210.0]);
        assert_eq!(
            overlap([0.0, 0.0, 10.0, 10.0], [10.0, 0.0, 20.0, 10.0]),
            Some([10.0, 0.0, 10.0, 10.0])
        );
        assert_eq!(
            overlap([0.0, 0.0, 10.0, 10.0], [11.0, 0.0, 20.0, 10.0]),
            None
        );
    }
}
