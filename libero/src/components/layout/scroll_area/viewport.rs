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

/// Assumed before the `ScrollArea` measured itself (first and server renders):
/// a 1080p screen, so no blank rows on first paint at any common height.
pub(super) const UNMEASURED: ScrollGeometry = ScrollGeometry {
    offset: 0.0,
    viewport: 1080.0,
};

/// What a settled `Virtualize` windows by: with the geometry, its rows and offsets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct WindowSpec {
    pub count: usize,
    pub pitch: f64,
    pub overscan: usize,
    pub keep: Option<usize>,
}

impl WindowSpec {
    /// The window at `geometry`, and where the kept row renders when outside it.
    pub fn at(self, geometry: Option<ScrollGeometry>) -> (Window, Option<(usize, bool)>) {
        let mut visible = window(
            self.count,
            self.pitch,
            geometry.unwrap_or(UNMEASURED),
            self.overscan,
        );
        let kept = keep_beside(&mut visible, self.keep, self.count, self.pitch);
        (visible, kept)
    }
}

/// The private contract between `ScrollArea` and `Virtualize`: geometry down,
/// the window's spec back up.
#[derive(Clone)]
pub(super) struct ScrollViewport {
    /// The box holding the rows; unlike the container's, its height isn't
    /// floored at the viewport.
    pub content: ElementHandle,
    /// `None` until the first measurement lands.
    pub geometry: Signal<Option<ScrollGeometry>>,
    /// The content box pads itself from it and the geometry, in the rows' render
    /// pass: offsets handed up by an effect lagged a whole fling behind (todo 2013).
    pub spec: Signal<Option<WindowSpec>>,
    /// Set by a `Virtualize`: the content box becomes a real box for the offsets.
    pub virtualized: Signal<bool>,
    // Not a signal: claimed in a child's first render, where dirtying the area
    // would cost a render pass for nothing drawn.
    claimed: Rc<Cell<bool>>,
}

impl ScrollViewport {
    pub fn new(
        content: ElementHandle,
        geometry: Signal<Option<ScrollGeometry>>,
        spec: Signal<Option<WindowSpec>>,
        virtualized: Signal<bool>,
    ) -> Self {
        Self {
            content,
            geometry,
            spec,
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

/// The rows visible at `geometry`, plus `overscan` beyond each edge. `pitch`
/// includes the gap, so the total overshoots by one trailing gap.
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

/// Where `keep` renders when the window left it out: right before the window
/// (`Some(true)`) or right after it, one pitch taken from that side's padding
/// so every other row stays put.
pub(super) fn keep_beside(
    window: &mut Window,
    keep: Option<usize>,
    count: usize,
    pitch: f64,
) -> Option<(usize, bool)> {
    let keep = keep.filter(|&keep| keep < count && !window.range.contains(&keep))?;
    let before = keep < window.range.start;
    let side = match before {
        true => &mut window.offsets.leading,
        false => &mut window.offsets.trailing,
    };
    *side = (*side - pitch).max(0.0);
    Some((keep, before))
}

/// Row pitch from two probe renders, one row then `rows`: the subtraction
/// cancels whatever else shares the scroll area.
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

    /// The content box and the rows derive the same window from one spec and geometry.
    #[test]
    fn a_spec_windows_at_the_geometry_and_keeps_its_row_beside() {
        let spec = WindowSpec {
            count: 1000,
            pitch: 20.0,
            overscan: 2,
            keep: Some(3),
        };
        let geometry = ScrollGeometry {
            offset: 400.0,
            ..GEOMETRY
        };
        let (window, kept) = spec.at(Some(geometry));

        assert_eq!(window.range, 18..28);
        assert_eq!(window.offsets.leading, 17.0 * 20.0);
        assert_eq!(kept, Some((3, true)));
        // Unmeasured: 1080px of 20px rows, a partial row and the overscan.
        assert_eq!(spec.at(None).0.range, 0..57);
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

    #[test]
    fn a_kept_row_above_the_window_takes_a_pitch_of_the_leading_space() {
        let geometry = ScrollGeometry {
            offset: 10_000.0,
            ..GEOMETRY
        };
        let mut window = window(1000, 20.0, geometry, 0);
        let leading = window.offsets.leading;

        assert_eq!(
            keep_beside(&mut window, Some(5), 1000, 20.0),
            Some((5, true))
        );
        assert_eq!(window.offsets.leading, leading - 20.0);
    }

    #[test]
    fn a_kept_row_below_the_window_takes_a_pitch_of_the_trailing_space() {
        let mut window = window(1000, 20.0, GEOMETRY, 0);
        let trailing = window.offsets.trailing;

        assert_eq!(
            keep_beside(&mut window, Some(500), 1000, 20.0),
            Some((500, false))
        );
        assert_eq!(window.offsets.trailing, trailing - 20.0);
    }

    #[test]
    fn a_kept_row_in_the_window_or_past_the_list_changes_nothing() {
        let mut window = window(1000, 20.0, GEOMETRY, 0);
        let before = window.clone();

        assert_eq!(keep_beside(&mut window, Some(2), 1000, 20.0), None);
        assert_eq!(keep_beside(&mut window, Some(1000), 1000, 20.0), None);
        assert_eq!(window, before);
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
