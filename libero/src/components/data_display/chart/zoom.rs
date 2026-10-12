//! The x window a zoomable chart draws, and the brush's band of x values. Both work on
//! x indices: the chart draws only the window's slice, the overlay maps marks back.

use dioxus::prelude::*;

use super::{
    ChartHandle,
    engine::{Drawn, Mark, MarkShape, SeriesSpec},
};

/// The fewest x values a zoom narrows to.
pub(super) const MIN_POINTS: usize = 2;

/// A band of x values the brush selected, both ends included.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{ChartRange, LineChart, Series};
/// # fn app() -> Element {
/// let mut range = use_signal(|| None::<ChartRange>);
/// rsx! {
///     LineChart {
///         title: "Visitors",
///         x: ["Mon", "Tue", "Wed", "Thu"],
///         series: vec![Series::new("Visits", [3, 5, 4, 6])],
///         onrange: move |selected| range.set(selected),
///     }
/// }
/// # }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ChartRange {
    /// The first selected x value's place in `x`.
    pub start: usize,
    /// The last selected x value's place in `x`, never before `start`.
    pub end: usize,
    /// The first selected x value, as written on the axis.
    pub from: String,
    /// The last, as written on the axis.
    pub to: String,
}

impl ChartRange {
    pub(super) fn of(a: usize, b: usize, x: &[String]) -> Self {
        let (start, end) = (a.min(b), a.max(b));
        Self {
            start,
            end,
            from: x.get(start).cloned().unwrap_or_default(),
            to: x.get(end).cloned().unwrap_or_default(),
        }
    }
}

/// `window` (start, end exclusive) `factor` times as wide around `at`, its share across,
/// kept inside `0..len` and at least [`MIN_POINTS`] wide.
pub(super) fn zoomed(window: (usize, usize), len: usize, factor: f64, at: f64) -> (usize, usize) {
    let (start, end) = window;
    let width = (end - start) as f64;
    let next = ((width * factor).round() as usize).clamp(MIN_POINTS.min(len), len);
    let anchor = start as f64 + at.clamp(0.0, 1.0) * width;
    let first = (anchor - at.clamp(0.0, 1.0) * next as f64).round().max(0.0) as usize;
    let first = first.min(len - next);
    (first, first + next)
}

/// `window` moved `by` x values, kept inside `0..len`.
pub(super) fn panned(window: (usize, usize), len: usize, by: isize) -> (usize, usize) {
    let width = window.1 - window.0;
    let first = (window.0 as isize + by).clamp(0, (len - width) as isize) as usize;
    (first, first + width)
}

/// The x window of a chart with `zoom` on, by index, shared by its overlay and toolbar.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Zoom {
    /// `None`: the whole axis.
    window: Signal<Option<(usize, usize)>>,
    pub(super) len: usize,
}

impl Zoom {
    fn fit(&self, window: Option<(usize, usize)>) -> (usize, usize) {
        match window {
            Some((start, end)) if end <= self.len && end > start => (start, end),
            _ => (0, self.len),
        }
    }

    /// The drawn window, subscribing.
    pub(super) fn range(&self) -> (usize, usize) {
        self.fit(*self.window.read())
    }

    pub(super) fn peek(&self) -> (usize, usize) {
        self.fit(*self.window.peek())
    }

    pub(super) fn is_zoomed(&self) -> bool {
        self.range() != (0, self.len)
    }

    pub(super) fn set(mut self, window: (usize, usize)) {
        let next = (window != (0, self.len)).then_some(window);
        if *self.window.peek() != next {
            self.window.set(next);
        }
    }

    pub(super) fn reset(self) {
        self.set((0, self.len));
    }

    /// Zooms by `factor` (below 1 in) around `at`, a share across the window.
    pub(super) fn zoom(self, factor: f64, at: f64) {
        self.set(zoomed(self.peek(), self.len, factor, at));
    }

    pub(super) fn pan(self, by: isize) {
        self.set(panned(self.peek(), self.len, by));
    }

    /// Moves the window the least so that `index` is inside it.
    pub(super) fn reveal(self, index: usize) {
        let (start, end) = self.peek();
        if index < start {
            self.pan(index as isize - start as isize);
        } else if index >= end {
            self.pan(index as isize - end as isize + 1);
        }
    }
}

/// The chart's [`Zoom`] over `len` x values while `enabled`, its window the `handle`'s so
/// the page that owns the handle may read it. Called on every render.
pub(super) fn use_zoom(len: usize, enabled: bool, handle: Option<ChartHandle>) -> Option<Zoom> {
    let own = use_signal(|| None);
    let window = handle.map_or(own, |handle| handle.window());
    enabled.then_some(Zoom { window, len })
}

/// Each series cut to `start..end`; a marked point outside it is not drawn.
pub(super) fn windowed(specs: Vec<SeriesSpec>, window: (usize, usize)) -> Vec<SeriesSpec> {
    specs
        .into_iter()
        .map(|mut spec| {
            spec.extras.mark_points = (spec.extras.mark_points.into_iter())
                .filter(|(index, _)| (window.0..window.1).contains(index))
                .map(|(index, text)| (index - window.0, text))
                .collect();
            SeriesSpec {
                values: cut(&spec.values, window),
                ..spec
            }
        })
        .collect()
}

/// `values[start..end]`, or nothing past their end.
pub(super) fn cut<T: Clone>(values: &[T], (start, end): (usize, usize)) -> Vec<T> {
    values
        .get(start.min(values.len())..end.min(values.len()))
        .map_or_else(Vec::new, <[T]>::to_vec)
}

impl Drawn {
    /// Marks of a window drawn from `start` name their x value's place in the whole axis.
    pub(super) fn with_offset(mut self, start: usize) -> Self {
        for mark in &mut self.geometry.marks {
            mark.index += start;
        }
        self
    }
}

/// A mark's extent along the x axis, which runs down when `down`.
fn extent(shape: &MarkShape, down: bool) -> Option<(f32, f32)> {
    match *shape {
        MarkShape::Point { x, y } => Some(if down { (y, y) } else { (x, x) }),
        MarkShape::Rect {
            x,
            y,
            width,
            height,
        } => Some(if down {
            (y, y + height)
        } else {
            (x, x + width)
        }),
        MarkShape::Sector { .. } => None,
    }
}

/// Where the x value at `index` spans along its axis, over all its marks.
pub(super) fn index_span(marks: &[Mark], index: usize, down: bool) -> Option<(f32, f32)> {
    marks
        .iter()
        .filter(|mark| mark.index == index)
        .filter_map(|mark| extent(&mark.shape, down))
        .reduce(|(a, b), (c, d)| (a.min(c), b.max(d)))
}

/// The x value whose marks' middle is nearest `at` along the axis.
pub(super) fn index_at(marks: &[Mark], at: f32, down: bool) -> Option<usize> {
    let mut indices: Vec<usize> = marks.iter().map(|mark| mark.index).collect();
    indices.sort_unstable();
    indices.dedup();
    indices
        .into_iter()
        .filter_map(|index| {
            let (from, to) = index_span(marks, index, down)?;
            Some((index, ((from + to) / 2.0 - at).abs()))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_zoom_keeps_its_anchor_and_stays_inside() {
        // Halving 0..10 around the middle.
        assert_eq!(zoomed((0, 10), 10, 0.5, 0.5), (3, 8));
        // Around the start, the start stays.
        assert_eq!(zoomed((0, 10), 10, 0.5, 0.0), (0, 5));
        // Out past the axis stops at the whole.
        assert_eq!(zoomed((2, 6), 10, 4.0, 0.5), (0, 10));
        // Never narrower than two points.
        assert_eq!(zoomed((4, 6), 10, 0.1, 0.5), (4, 6));
        assert_eq!(zoomed((0, 1), 1, 0.5, 0.5), (0, 1));
    }

    /// A marked point keeps its place in the window, or leaves with the rest of the cut.
    #[test]
    fn a_window_moves_its_marked_points() {
        let spec = SeriesSpec {
            name: "A".into(),
            values: vec![Some(1.0); 6],
            color: crate::theme::HexColor::new(0x228BE6),
            dashed: false,
            extras: super::super::engine::SeriesExtras {
                mark_points: vec![(1, "low".into()), (4, "high".into())],
                ..Default::default()
            },
        };
        let cut = windowed(vec![spec], (3, 6));
        assert_eq!(cut[0].values.len(), 3);
        assert_eq!(cut[0].extras.mark_points, [(1, "high".to_string())]);
    }

    #[test]
    fn a_pan_stops_at_either_end() {
        assert_eq!(panned((2, 5), 10, 3), (5, 8));
        assert_eq!(panned((2, 5), 10, 30), (7, 10));
        assert_eq!(panned((2, 5), 10, -9), (0, 3));
    }

    #[test]
    fn a_range_is_ordered_and_named() {
        let x: Vec<String> = ["Mon", "Tue", "Wed"].map(String::from).to_vec();
        assert_eq!(
            ChartRange::of(2, 0, &x),
            ChartRange {
                start: 0,
                end: 2,
                from: "Mon".into(),
                to: "Wed".into()
            }
        );
    }

    fn bar(index: usize, x: f32) -> Mark {
        Mark {
            series: 0,
            index,
            value: 1.0,
            shape: MarkShape::Rect {
                x,
                y: 0.0,
                width: 10.0,
                height: 5.0,
            },
        }
    }

    #[test]
    fn the_pointer_lands_on_the_nearest_x_value() {
        let marks = [bar(3, 0.0), bar(3, 10.0), bar(4, 40.0)];
        assert_eq!(index_span(&marks, 3, false), Some((0.0, 20.0)));
        assert_eq!(index_at(&marks, 12.0, false), Some(3));
        assert_eq!(index_at(&marks, 33.0, false), Some(4));
        // Down the side, a bar's height is its extent.
        assert_eq!(index_span(&marks, 4, true), Some((0.0, 5.0)));
    }

    /// The plot takes the gestures and says its keys; it draws no buttons.
    #[test]
    fn a_zoomable_chart_has_its_keys_and_no_buttons() {
        use crate::{
            LiberoProvider,
            components::data_display::{LineChart, Series},
        };
        let mut dom = VirtualDom::new(|| {
            rsx! {
                LiberoProvider {
                    LineChart {
                        title: "Visitors",
                        x: ["Mon", "Tue", "Wed"],
                        series: vec![Series::new("Visits", [1.0, 2.0, 3.0])],
                        zoom: true,
                        onrange: |_| {},
                    }
                }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("data-gestures=\"true\""), "{html}");
        assert!(!html.contains("<button"), "{html}");
        assert!(
            html.contains("0 resets the zoom.") && html.contains("Escape clears it."),
            "{html}"
        );
        // The whole axis is drawn until a zoom.
        assert!(html.contains("Wed"), "{html}");
    }

    #[test]
    fn a_window_cuts_every_series_and_offsets_its_marks() {
        let spec = SeriesSpec {
            name: "a".into(),
            values: vec![Some(1.0), Some(2.0), None, Some(4.0)],
            color: crate::theme::HexColor::new(0),
            dashed: false,
            extras: Default::default(),
        };
        assert_eq!(windowed(vec![spec], (1, 3))[0].values, [Some(2.0), None]);
        assert_eq!(cut(&[1, 2, 3], (2, 9)), [3]);
    }
}
