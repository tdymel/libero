//! An outside press's click swallowed (4839): no capture phase, so the wrapper's press decides
//! and [`refused::release`](super::refused::release) prevents the release, as for a disabled control.

use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

use super::{HIT, Origin, anchor, element::mounted_node, media_query::media_query};
use crate::platform::{MediaQuerySubscription, OutsidePress, PressSubscription};

thread_local! {
    static SWALLOWS: RefCell<Vec<Weak<Swallow>>> = const { RefCell::new(Vec::new()) };
    /// Whether the press under way loses its click.
    static SWALLOWING: Cell<bool> = const { Cell::new(false) };
    /// The swallow whose press is under way, told if it loses its click.
    static PENDING: RefCell<Option<Rc<Swallow>>> = const { RefCell::new(None) };
}

struct Swallow {
    matches: Rc<Cell<bool>>,
    outside: OutsidePress,
    onswallow: Rc<dyn Fn()>,
    /// The subscribing scope: `outside` reads its signals there.
    origin: Origin,
}

struct BlitzSwallow {
    _swallow: Rc<Swallow>,
    _watch: Box<dyn MediaQuerySubscription>,
}

impl PressSubscription for BlitzSwallow {}

pub(in crate::platform::backend) fn swallow_outside_clicks(
    media: &str,
    outside: OutsidePress,
    onswallow: Rc<dyn Fn()>,
) -> Option<Box<dyn PressSubscription>> {
    let matches = Rc::new(Cell::new(false));
    let watch = media_query()?.watch(
        media,
        Box::new({
            let matches = matches.clone();
            move |now| matches.set(now)
        }),
    );
    let swallow = Rc::new(Swallow {
        matches,
        outside,
        onswallow,
        origin: Origin::here(),
    });
    SWALLOWS.with_borrow_mut(|swallows| {
        swallows.retain(|swallow| swallow.strong_count() > 0);
        swallows.push(Rc::downgrade(&swallow));
    });
    Some(Box::new(BlitzSwallow {
        _swallow: swallow,
        _watch: watch,
    }))
}

/// At each press, once [`HIT`] holds its node.
pub(super) fn pressed() {
    let live: Vec<Rc<Swallow>> =
        SWALLOWS.with_borrow(|swallows| swallows.iter().filter_map(Weak::upgrade).collect());
    let target = HIT
        .get()
        .filter(|_| live.iter().any(|swallow| swallow.matches.get()))
        .zip(anchor())
        .map(|(hit, anchor)| mounted_node(anchor, hit));
    let pending = target.and_then(|target| {
        live.into_iter().find(|swallow| {
            swallow.matches.get() && swallow.origin.run(|| (swallow.outside)(&target))
        })
    });
    SWALLOWING.set(pending.is_some());
    PENDING.set(pending);
}

/// A key ends the swallow, as on the web.
pub(super) fn clear() {
    SWALLOWING.set(false);
    PENDING.take();
}

/// Whether the press now released loses its click; asked once per release.
pub(super) fn take() -> bool {
    let pending = PENDING.take();
    let taken = SWALLOWING.replace(false);
    if let Some(swallow) = pending.filter(|_| taken) {
        swallow.origin.run(|| (swallow.onswallow)());
    }
    taken
}
