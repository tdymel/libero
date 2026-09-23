use std::rc::Rc;

use dioxus::prelude::MountedData;

use super::{ContentSubscription, backend};

/// Calls `callback` with `(is_intersecting, ratio)` at the first observation
/// and whenever `target` crosses a threshold of its `root` (the viewport for
/// `None`) grown by `root_margin`. Only the web has an `IntersectionObserver`;
/// elsewhere, and where the browser lacks it, `None`: nothing ever intersects.
pub(crate) fn on_intersection(
    target: &Rc<MountedData>,
    root: Option<&Rc<MountedData>>,
    root_margin: &str,
    thresholds: &[f64],
    callback: Box<dyn Fn(bool, f64)>,
) -> Option<Box<dyn ContentSubscription>> {
    backend::on_intersection(target, root, root_margin, thresholds, callback)
}
