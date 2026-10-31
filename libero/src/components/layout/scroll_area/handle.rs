use dioxus::prelude::*;

use super::scroll_area::UNLAID_TRIES;
use crate::{
    hooks::{ElementHandle, use_element},
    platform::{ElementApi, when_laid_out_retry},
    theme::ScrollAxis,
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
    /// The area's `scrollbars`, written by the area: an axis it does not scroll stays put (5130).
    pub(crate) axes: Option<CopyValue<ScrollAxis>>,
}

/// A handle for one [`ScrollArea`](super::ScrollArea). Pass it as that area's
/// `handle`.
pub fn use_scroll_area() -> ScrollAreaHandle {
    ScrollAreaHandle {
        element: use_element(),
        axes: Some(use_hook(|| CopyValue::new(ScrollAxis::Both))),
    }
}

impl ScrollAreaHandle {
    /// Scrolls to an offset in px, clamped by the browser to the range. `x`
    /// counts from the inline start, the right edge under `dir="rtl"`.
    /// An axis outside the area's `scrollbars` stays at its start.
    pub fn scroll_to(&self, x: f64, y: f64) {
        let (scrolls_x, scrolls_y) = scrolls(self.axes_or_both());
        let (x, y) = (
            if scrolls_x { x } else { 0.0 },
            if scrolls_y { y } else { 0.0 },
        );
        // Zero is the start either way; the direction read forces a style recalc (todo 2148).
        let x = match x == 0.0 {
            true => 0.0,
            false => physical_x(x, self.element.is_rtl()),
        };
        let _ = self.element.scroll_to(x, y);
    }

    /// Scrolls to a percent (0-100) of each axis's range. `None` leaves that
    /// axis where it is.
    pub fn scroll_to_percent(&self, x: Option<f64>, y: Option<f64>) {
        scroll_to_percent(self.element, self.axes_or_both(), x, y);
    }

    fn axes_or_both(&self) -> ScrollAxis {
        self.axes.map_or(ScrollAxis::Both, |axes| *axes.peek())
    }
}

/// Whether an area with these `scrollbars` scrolls x and y.
fn scrolls(axes: ScrollAxis) -> (bool, bool) {
    match axes {
        ScrollAxis::Vertical => (false, true),
        ScrollAxis::Horizontal => (true, false),
        ScrollAxis::Both => (true, true),
        ScrollAxis::None => (false, false),
    }
}

/// `None` for an axis the area does not scroll, so it stays where it is.
fn on_axes(axes: ScrollAxis, x: Option<f64>, y: Option<f64>) -> (Option<f64>, Option<f64>) {
    let (scrolls_x, scrolls_y) = scrolls(axes);
    (x.filter(|_| scrolls_x), y.filter(|_| scrolls_y))
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
pub(super) fn scroll_to_percent(
    root: ElementHandle,
    axes: ScrollAxis,
    x: Option<f64>,
    y: Option<f64>,
) {
    let (x, y) = on_axes(axes, x, y);
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
            let failed = UNLAID_TRIES - tries;
            return when_laid_out_retry(failed, move || {
                scroll_to_percent_tried(root, x, y, tries - 1)
            });
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

    /// Todo 5130: a horizontal area's "to bottom" moved its hidden y axis out of reach.
    #[test]
    fn an_axis_the_area_does_not_scroll_is_dropped() {
        let both = (Some(1.0), Some(2.0));
        assert_eq!(
            on_axes(ScrollAxis::Horizontal, both.0, both.1),
            (Some(1.0), None)
        );
        assert_eq!(
            on_axes(ScrollAxis::Vertical, both.0, both.1),
            (None, Some(2.0))
        );
        assert_eq!(on_axes(ScrollAxis::Both, both.0, both.1), both);
        assert_eq!(on_axes(ScrollAxis::None, both.0, both.1), (None, None));
    }
}
