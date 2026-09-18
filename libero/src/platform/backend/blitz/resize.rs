//! Blitz reports no `resize` and no mutation. So each watched node's size is
//! compared at every [`Outlet`] flush, shortly after each press or key, and
//! every [`POLL`] while anything is watched, which also catches a window
//! resize.
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

use super::{Doc, doc, resolved_style_value, when_free};
use crate::platform::{
    ContentSubscription, Dimensions, TimerSubscription,
    backend::{origin::Origin, thread},
    resize::measured_resize,
};

/// How late a change no press or key preceded is seen at most: a window
/// resize, a timer's render, a picture that loaded.
const POLL: Duration = Duration::from_millis(500);

/// After a press or key: past the frame that lays out what it rendered.
const SETTLE: Duration = Duration::from_millis(30);

enum Kind {
    /// The border box, as `ResizeObserver` reports it.
    Resize(Rc<dyn Fn(Event<ResizeData>)>),
    /// The scroll size, where a subtree change shows.
    Content(Rc<dyn Fn()>),
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
    poll: RefCell<Option<Box<dyn TimerSubscription>>>,
    settle: RefCell<Option<Box<dyn TimerSubscription>>>,
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
    let mut poll = doc.resize.poll.borrow_mut();
    if poll.is_none() {
        let weak = Rc::downgrade(&doc);
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
    drop(poll);
    // The first measure, once laid out.
    settle_soon(&doc);
    Some(Box::new(Subscription(id, Rc::downgrade(&doc))))
}

struct Subscription(u64, Weak<Doc>);

impl ContentSubscription for Subscription {}

impl Drop for Subscription {
    fn drop(&mut self) {
        let Some(doc) = self.1.upgrade() else {
            return;
        };
        let empty = {
            let mut watched = doc.resize.watched.borrow_mut();
            watched.retain(|watched| watched.id != self.0);
            watched.is_empty()
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
}

/// Measures every watched node and calls those whose size changed, each in
/// its own scope. A document still borrowed is left for the next check.
pub(super) fn check(doc: &Doc) {
    let fired: Vec<(Origin, Fire)> = {
        let mut watched = doc.resize.watched.borrow_mut();
        let Some(anchor) = watched.first().map(|watched| watched.anchor.clone()) else {
            return;
        };
        let Some(base) = anchor.try_doc() else {
            return;
        };
        watched
            .iter_mut()
            .filter_map(|watched| {
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
                };
                Some((watched.origin, fire))
            })
            .collect()
    };
    for (origin, fire) in fired {
        origin.run(|| match fire {
            Fire::Resize(callback, border, content) => callback(measured_resize(border, content)),
            Fire::Content(callback) => callback(),
        });
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
    };
    Some((compared, outer, inner))
}
