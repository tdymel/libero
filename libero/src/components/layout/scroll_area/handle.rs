use dioxus::prelude::*;

use crate::{
    hooks::{ElementHandle, use_element},
    platform::ElementApi,
};

/// Scrolls a [`ScrollArea`](super::ScrollArea) from an event handler.
///
/// The percent props are declarative: they re-apply only when their value
/// changes, so asking for the same position twice - after the reader has
/// scrolled away in between - does nothing. A handle is a command, and every
/// call scrolls.
///
/// ```ignore
/// let area = use_scroll_area();
/// rsx! {
///     ScrollArea { handle: area, .. }
///     Button { onclick: move |_| area.scroll_to_percent(None, Some(0.0)), "Top" }
/// }
/// ```
///
/// `Copy`, so any number of handlers can hold it. A call before the bound
/// `ScrollArea` has mounted, or with none bound at all, does nothing.
#[derive(Clone, Copy, PartialEq)]
pub struct ScrollAreaHandle {
    pub(super) element: ElementHandle,
}

/// A handle for one [`ScrollArea`](super::ScrollArea). Pass it as that area's
/// `handle`.
pub fn use_scroll_area() -> ScrollAreaHandle {
    ScrollAreaHandle {
        element: use_element(),
    }
}

impl ScrollAreaHandle {
    /// Scrolls to an offset in px, clamped by the browser to the range.
    pub fn scroll_to(&self, x: f64, y: f64) {
        let _ = self.element.scroll_to(x, y);
    }

    /// Scrolls to a percent (0-100) of each axis's range. `None` leaves that
    /// axis where it is.
    pub fn scroll_to_percent(&self, x: Option<f64>, y: Option<f64>) {
        scroll_to_percent(self.element, x, y);
    }
}

/// Where `pct` percent of a `max` px range lands, or `current` for `None`.
fn percent_offset(pct: Option<f64>, max: f64, current: f64) -> f64 {
    pct.map_or(current, |pct| max.max(0.0) * pct.clamp(0.0, 100.0) / 100.0)
}

/// Shared by the handle and the `scroll_position_*` props.
///
/// Started here, awaited in the task: a read resolves where it is called (see
/// `ElementApi::dimensions`). Off the web each is a round-trip, so the
/// awaiting has to happen in a task rather than inline.
pub(super) fn scroll_to_percent(root: ElementHandle, x: Option<f64>, y: Option<f64>) {
    if (x.is_none() && y.is_none()) || !root.is_mounted() {
        return;
    }
    let (content, viewport_size, offset) =
        (root.scroll_size(), root.dimensions(), root.scroll_offset());
    spawn(async move {
        let (Ok(scroll_size), Ok(viewport), Ok((current_x, current_y))) =
            (content.await, viewport_size.await, offset.await)
        else {
            return;
        };
        let _ = root.scroll_to(
            percent_offset(x, scroll_size.width - viewport.width, current_x),
            percent_offset(y, scroll_size.height - viewport.height, current_y),
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_percent_lands_on_its_share_of_the_range() {
        assert_eq!(percent_offset(Some(20.0), 500.0, 70.0), 100.0);
        assert_eq!(percent_offset(Some(150.0), 500.0, 70.0), 500.0);
        assert_eq!(percent_offset(Some(-5.0), 500.0, 70.0), 0.0);
    }

    /// `None` keeps the axis where it is, and content that fits has no range.
    #[test]
    fn none_keeps_the_current_offset() {
        assert_eq!(percent_offset(None, 500.0, 70.0), 70.0);
        assert_eq!(percent_offset(Some(50.0), -10.0, 0.0), 0.0);
    }
}
