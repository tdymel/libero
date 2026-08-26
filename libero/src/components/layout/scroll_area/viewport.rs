use std::{cell::Cell, ops::Range, rc::Rc};

use dioxus::prelude::*;

use crate::hooks::ElementHandle;

/// Where a `ScrollArea` is scrolled to and how tall it is, in px. Vertical
/// only: nothing windows a row on the horizontal axis yet.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct ScrollGeometry {
    pub offset: f64,
    pub viewport: f64,
}

/// Space standing in for the rows a `Virtualize` did not render, which its
/// `ScrollArea` pads itself with.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct ContentOffsets {
    pub leading: f64,
    pub trailing: f64,
}

/// The contract between `ScrollArea` and `Virtualize`: geometry down, offsets
/// back up. Private both ways - neither appears in either component's props,
/// and `ScrollArea` never reads `Virtualize`.
#[derive(Clone)]
pub(super) struct ScrollViewport {
    /// The box holding the rows. It sizes to them, so measuring it gives the
    /// real content height - `scroll_size` on the container floors at the
    /// viewport, which a short probe never clears.
    pub content: ElementHandle,
    /// `None` until the first measurement lands.
    pub geometry: Signal<Option<ScrollGeometry>>,
    pub offsets: Signal<ContentOffsets>,
    /// Set by a `Virtualize`: the content box stops being `display: contents`
    /// and becomes a real box that can carry the offsets.
    pub virtualized: Signal<bool>,
    // A plain cell, not a signal: claiming happens during a child's first
    // render, and dirtying the ScrollArea's scope from there would cost a
    // render pass for something nothing draws.
    claimed: Rc<Cell<bool>>,
}

impl ScrollViewport {
    pub fn new(
        content: ElementHandle,
        geometry: Signal<Option<ScrollGeometry>>,
        offsets: Signal<ContentOffsets>,
        virtualized: Signal<bool>,
    ) -> Self {
        Self {
            content,
            geometry,
            offsets,
            virtualized,
            claimed: Rc::new(Cell::new(false)),
        }
    }

    /// `true` for the first caller only. There is one set of offsets, so a
    /// second `Virtualize` would fight the first over the padding.
    pub fn claim(&self) -> bool {
        !self.claimed.replace(true)
    }
}

/// The rows a window shows, and the px reserved on either side of them.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Window {
    pub range: Range<usize>,
    pub offsets: ContentOffsets,
}

impl Window {
    /// Every row, reserving nothing - what an unmeasurable list falls back to.
    pub fn all(count: usize) -> Self {
        Self {
            range: 0..count,
            offsets: ContentOffsets::default(),
        }
    }
}

/// A length only counts if it is a real, positive number of px - an unmounted
/// element measures as zero, and a zero pitch divides the whole list by it.
fn measurable(px: f64) -> bool {
    px.is_finite() && px > 0.0
}

/// The rows visible at `geometry`, plus `overscan` beyond each edge.
///
/// `pitch` is a row's height *plus the gap below it*, so `count * pitch`
/// overshoots the real content by one trailing gap - a few px at the very
/// bottom, against a scroll range of thousands.
pub(super) fn window(
    count: usize,
    pitch: f64,
    geometry: ScrollGeometry,
    overscan: usize,
) -> Window {
    if count == 0 {
        return Window::all(0);
    }
    if !measurable(pitch) || !measurable(geometry.viewport) {
        return Window::all(count);
    }

    let first = (geometry.offset.max(0.0) / pitch).floor() as usize;
    // The partial row at each edge is why this rounds up and adds one.
    let rows = (geometry.viewport / pitch).ceil() as usize + 1;

    let start = first.saturating_sub(overscan).min(count);
    let end = first
        .saturating_add(rows)
        .saturating_add(overscan)
        .clamp(start, count);

    Window {
        range: start..end,
        offsets: ContentOffsets {
            leading: start as f64 * pitch,
            trailing: (count - end) as f64 * pitch,
        },
    }
}

/// Row pitch from two probe renders - one row tall, then `rows` tall.
///
/// The subtraction is the point: whatever else shares the scroll area is a
/// constant in both heights and cancels, so only the rows are measured.
pub(super) fn probed_pitch(single: f64, batch: f64, rows: usize) -> Option<f64> {
    let pitch = match rows {
        0 | 1 => single,
        rows => (batch - single) / (rows - 1) as f64,
    };

    measurable(pitch).then_some(pitch)
}

#[cfg(test)]
mod tests {
    use super::*;

    const GEOMETRY: ScrollGeometry = ScrollGeometry {
        offset: 0.0,
        viewport: 100.0,
    };

    #[test]
    fn the_first_window_starts_at_the_top() {
        let window = window(1000, 20.0, GEOMETRY, 0);

        assert_eq!(window.range, 0..6);
        assert_eq!(window.offsets.leading, 0.0);
        assert_eq!(window.offsets.trailing, (1000 - 6) as f64 * 20.0);
    }

    #[test]
    fn overscan_widens_both_edges() {
        let geometry = ScrollGeometry {
            offset: 400.0,
            ..GEOMETRY
        };

        assert_eq!(window(1000, 20.0, geometry, 2).range, 18..28);
    }

    /// Nothing to overscan into above the first row.
    #[test]
    fn overscan_does_not_run_off_the_top() {
        assert_eq!(window(1000, 20.0, GEOMETRY, 4).range, 0..10);
    }

    #[test]
    fn the_last_window_ends_at_the_last_row() {
        let geometry = ScrollGeometry {
            offset: 19_900.0,
            ..GEOMETRY
        };
        let window = window(1000, 20.0, geometry, 2);

        assert_eq!(window.range.end, 1000);
        assert_eq!(window.offsets.trailing, 0.0);
    }

    /// Scrolled past the end - a stale offset against a shortened list.
    #[test]
    fn an_offset_beyond_the_content_stays_in_range() {
        let geometry = ScrollGeometry {
            offset: 100_000.0,
            ..GEOMETRY
        };
        let window = window(10, 20.0, geometry, 2);

        assert_eq!(window.range, 10..10);
        assert_eq!(window.offsets.trailing, 0.0);
    }

    #[test]
    fn an_unmeasured_viewport_renders_everything() {
        let geometry = ScrollGeometry {
            viewport: 0.0,
            ..GEOMETRY
        };

        assert_eq!(window(1000, 20.0, geometry, 2), Window::all(1000));
    }

    #[test]
    fn an_unmeasured_pitch_renders_everything() {
        assert_eq!(window(1000, 0.0, GEOMETRY, 2), Window::all(1000));
    }

    /// The probe renders 1 row then 8; 8 rows of 20px with an 4px gap between
    /// them is 20 + 7 * 24.
    #[test]
    fn the_probe_difference_is_the_pitch() {
        assert_eq!(probed_pitch(20.0, 20.0 + 7.0 * 24.0, 8), Some(24.0));
    }

    /// Content the probe did not render is a constant in both heights.
    #[test]
    fn surrounding_content_cancels_out() {
        assert_eq!(
            probed_pitch(80.0 + 20.0, 80.0 + 20.0 + 7.0 * 24.0, 8),
            Some(24.0)
        );
    }

    /// A one-row list has no gap to measure, so its own height is the pitch.
    #[test]
    fn a_single_row_is_its_own_pitch() {
        assert_eq!(probed_pitch(20.0, 20.0, 1), Some(20.0));
    }

    #[test]
    fn a_zero_height_row_is_not_a_pitch() {
        assert_eq!(probed_pitch(0.0, 0.0, 8), None);
    }
}
