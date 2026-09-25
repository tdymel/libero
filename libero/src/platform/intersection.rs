use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::MountedData;

use super::{ContentSubscription, backend};

/// Carries an observed element's tag on a WebView, which holds no DOM handle to
/// observe by. Rendered only where [`observes_by_tag`] answers.
pub(crate) const OBSERVE_ATTR: &str = "data-lsx-observe";

/// On a popup box, its anchor's [`OBSERVE_ATTR`] tag: the WebView's `Hotkey::within`
/// walk goes on from there, as for `aria-controls` (1225).
pub(crate) const OWNER_ATTR: &str = "data-lsx-owner";

static NEXT_TAG: AtomicU64 = AtomicU64::new(0);

/// A tag no other observed element carries.
pub(crate) fn next_observe_tag() -> u64 {
    NEXT_TAG.fetch_add(1, Ordering::Relaxed)
}

/// Whether the renderer finds the observed element by its [`OBSERVE_ATTR`]: a
/// WebView, where the page's script runs. The web and Blitz hold the element.
pub(crate) fn observes_by_tag() -> bool {
    backend::observes_by_tag()
}

/// A computed CSS length of the element carrying `tag`, read by the page's script:
/// a WebView's handles hold no DOM node. `Unsupported` on every other renderer.
pub(crate) fn computed_px_by_tag(tag: u64, property: &str) -> super::Read<Option<f64>> {
    backend::computed_px_by_tag(tag, property)
}

/// Calls `callback` with `(is_intersecting, ratio)` at the first observation
/// and whenever `target` crosses a threshold of its `root` (the viewport for
/// `None`) grown by `root_margin`. The web and a WebView use an
/// `IntersectionObserver`, a WebView on the element tagged `tags`; elsewhere
/// `None`: nothing ever intersects.
pub(crate) fn on_intersection(
    target: &Rc<MountedData>,
    root: Option<&Rc<MountedData>>,
    tags: (u64, Option<u64>),
    root_margin: &str,
    thresholds: &[f64],
    callback: Box<dyn Fn(bool, f64)>,
) -> Option<Box<dyn ContentSubscription>> {
    backend::on_intersection(target, root, tags, root_margin, thresholds, callback)
}
