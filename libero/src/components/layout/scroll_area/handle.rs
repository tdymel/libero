use dioxus::prelude::*;

use super::scroll_area::UNLAID_TRIES;
use crate::{
    hooks::{ElementHandle, use_element},
    platform::{ElementApi, when_laid_out},
};

/// Scrolls a [`ScrollArea`](super::ScrollArea) from an event handler. Unlike
/// the percent props, every call scrolls; before mount it does nothing.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::{Button, ScrollArea, use_scroll_area};
/// # fn app() -> Element {
/// let area = use_scroll_area();
/// rsx! {
///     ScrollArea { handle: area, "…" }
///     Button { onclick: move |_| area.scroll_to_percent(None, Some(0.0)), "Top" }
/// }
/// # }
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct ScrollAreaHandle {
    /// For `Scroller` and `Carousel` to read px offsets for a drag or a step.
    pub(crate) element: ElementHandle,
}

/// A handle for one [`ScrollArea`](super::ScrollArea). Pass it as that area's
/// `handle`.
pub fn use_scroll_area() -> ScrollAreaHandle {
    ScrollAreaHandle {
        element: use_element(),
    }
}

impl ScrollAreaHandle {
    /// Scrolls to an offset in px, clamped by the browser to the range. `x`
    /// counts from the inline start, the right edge under `dir="rtl"`.
    pub fn scroll_to(&self, x: f64, y: f64) {
        let _ = self
            .element
            .scroll_to(physical_x(x, self.element.is_rtl()), y);
    }

    /// Scrolls to a percent (0-100) of each axis's range. `None` leaves that
    /// axis where it is.
    pub fn scroll_to_percent(&self, x: Option<f64>, y: Option<f64>) {
        scroll_to_percent(self.element, x, y);
    }
}

/// `scrollLeft` counts down from 0 under RTL, so its size is the distance from
/// the inline start in either direction.
pub(crate) fn inline_x(scroll_left: f64) -> f64 {
    scroll_left.abs()
}

/// [`inline_x`] back to the `scrollLeft` the platform takes.
pub(crate) fn physical_x(x: f64, rtl: bool) -> f64 {
    if rtl { -x } else { x }
}

/// Where `pct` percent of a `max` px range lands, or `current` for `None`.
fn percent_offset(pct: Option<f64>, max: f64, current: f64) -> f64 {
    pct.map_or(current, |pct| max.max(0.0) * pct.clamp(0.0, 100.0) / 100.0)
}

/// Shared by the handle and the `scroll_position_*` props. Reads start here and
/// are awaited in a task: off the web each is a round-trip.
pub(super) fn scroll_to_percent(root: ElementHandle, x: Option<f64>, y: Option<f64>) {
    scroll_to_percent_tried(root, x, y, UNLAID_TRIES);
}

/// An area of no size is asked again once laid out: natively one mounted this
/// frame has no range yet, and the scroll would stay at 0 (todo 916).
fn scroll_to_percent_tried(root: ElementHandle, x: Option<f64>, y: Option<f64>, tries: u8) {
    if (x.is_none() && y.is_none()) || !root.is_mounted() {
        return;
    }
    let (content, viewport_size, offset, rtl) = (
        root.scroll_size(),
        root.dimensions(),
        root.scroll_offset(),
        root.is_rtl(),
    );
    spawn(async move {
        let (Ok(scroll_size), Ok(viewport), Ok((current_x, current_y))) =
            (content.await, viewport_size.await, offset.await)
        else {
            return;
        };
        if viewport.width <= 0.0 && viewport.height <= 0.0 && tries > 0 {
            return when_laid_out(move || scroll_to_percent_tried(root, x, y, tries - 1));
        }
        let x = percent_offset(x, scroll_size.width - viewport.width, inline_x(current_x));
        let _ = root.scroll_to(
            physical_x(x, rtl),
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

    /// Under RTL `scrollLeft` runs 0 down to minus the range.
    #[test]
    fn an_rtl_offset_counts_from_the_inline_start() {
        assert_eq!(inline_x(-120.0), 120.0);
        assert_eq!(inline_x(120.0), 120.0);
        assert_eq!(physical_x(120.0, true), -120.0);
        assert_eq!(physical_x(120.0, false), 120.0);
    }
}
