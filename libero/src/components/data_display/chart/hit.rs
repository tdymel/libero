//! Which mark a pointer or a key lands on, from the harvested [`Geometry`](super::engine::Geometry).

use std::{cmp::Ordering, f32::consts::TAU};

use super::engine::{Drawn, Mark, MarkShape};

impl Drawn {
    /// Maps marks from their series' place among the drawn ones to `shown`, its place in
    /// `series`, and sorts them by series then x for the keys: a lying bar chart's run up.
    pub(super) fn with_series(mut self, shown: &[usize]) -> Self {
        for mark in &mut self.geometry.marks {
            mark.series = shown.get(mark.series).copied().unwrap_or(mark.series);
        }
        self.geometry
            .marks
            .sort_by_key(|mark| (mark.series, mark.index));
        self
    }

    /// The same for a scatter, each series' points sorted left to right, then bottom up.
    pub(super) fn with_points(mut self, shown: &[usize]) -> Self {
        for mark in &mut self.geometry.marks {
            mark.series = shown.get(mark.series).copied().unwrap_or(mark.series);
        }
        self.geometry.marks.sort_by(|a, b| {
            let (ax, ay) = anchor(&a.shape);
            let (bx, by) = anchor(&b.shape);
            a.series
                .cmp(&b.series)
                .then(ax.total_cmp(&bx))
                .then(by.total_cmp(&ay))
        });
        self
    }

    /// The same for a pie's slices, which marks name by index.
    pub(super) fn with_indices(mut self, shown: &[usize]) -> Self {
        for mark in &mut self.geometry.marks {
            mark.index = shown.get(mark.index).copied().unwrap_or(mark.index);
        }
        self
    }
}

/// The mark under `(x, y)`, in SVG units: a line's nearest column then its nearest
/// point, the bar under it or else the nearest in x, the slice under it.
pub(super) fn hit(marks: &[Mark], x: f32, y: f32) -> Option<usize> {
    let by = |key: &dyn Fn(&Mark) -> Option<(f32, f32)>| {
        marks
            .iter()
            .enumerate()
            .filter_map(|(i, mark)| key(mark).map(|key| (i, key)))
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
    };
    let point = |mark: &Mark| match mark.shape {
        MarkShape::Point { x: mx, y: my } => Some(((mx - x).abs(), (my - y).abs())),
        _ => None,
    };
    let rect = |mark: &Mark| match mark.shape {
        MarkShape::Rect {
            x: left,
            y: top,
            width,
            height,
        } => Some((
            (left - x).max(x - left - width).max(0.0),
            (top - y).max(y - top - height).max(0.0),
        )),
        _ => None,
    };
    if let MarkShape::Sector { .. } = marks.first()?.shape {
        return marks.iter().position(|mark| match &mark.shape {
            MarkShape::Sector { path } => contains(&outline(path), x, y),
            _ => false,
        });
    }
    if !marks.iter().any(|mark| rect(mark).is_some()) {
        return by(&point);
    }
    // A line among bars wins near its points; elsewhere the bar under the pointer does.
    by(&|mark| point(mark).filter(|(dx, dy)| dx.hypot(*dy) <= POINT_REACH)).or_else(|| by(&rect))
}

/// The point nearest `(x, y)` by distance: a scatter's or a radar's, which share no columns.
pub(super) fn nearest(marks: &[Mark], x: f32, y: f32) -> Option<usize> {
    let distance = |mark: &Mark| {
        let (mx, my) = anchor(&mark.shape);
        (mx - x).hypot(my - y)
    };
    (0..marks.len()).min_by(|a, b| distance(&marks[*a]).total_cmp(&distance(&marks[*b])))
}

/// A scatter's key step: along a series in its marks' order (left to right), across to the
/// next or previous series' point nearest in x, then in y.
pub(super) fn walk_free(marks: &[Mark], active: Option<usize>, step: Step) -> Option<usize> {
    let last = marks.len().checked_sub(1)?;
    let Some(active) = active.filter(|active| *active <= last) else {
        return Some(match step {
            Step::Previous | Step::Last => last,
            _ => 0,
        });
    };
    let current = &marks[active];
    let in_series = || (0..marks.len()).filter(|i| marks[*i].series == current.series);
    let target = match step {
        Step::First => in_series().next(),
        Step::Last => in_series().next_back(),
        Step::Next => in_series().find(|i| *i > active),
        Step::Previous => in_series().rfind(|i| *i < active),
        Step::Down | Step::Up => {
            let mut series: Vec<usize> = marks.iter().map(|mark| mark.series).collect();
            series.dedup();
            let at = series.iter().position(|s| *s == current.series)?;
            let target = match step {
                Step::Down => series.get(at + 1),
                _ => at.checked_sub(1).and_then(|at| series.get(at)),
            };
            let (x, y) = anchor(&current.shape);
            let off = |i: &usize| {
                let (mx, my) = anchor(&marks[*i].shape);
                ((mx - x).abs(), (my - y).abs())
            };
            target.and_then(|&target| {
                (0..marks.len())
                    .filter(|i| marks[*i].series == target)
                    .min_by(|a, b| off(a).partial_cmp(&off(b)).unwrap_or(Ordering::Equal))
            })
        }
    };
    Some(target.unwrap_or(active))
}

/// How far from a line's point among bars the pointer still picks it, in SVG units.
const POINT_REACH: f32 = 12.0;

/// Where a mark's tooltip and marker sit: a point itself, a bar's top centre, a slice's
/// middle between its rims.
pub(super) fn anchor(shape: &MarkShape) -> (f32, f32) {
    match shape {
        MarkShape::Point { x, y } => (*x, *y),
        MarkShape::Rect { x, y, width, .. } => (x + width / 2.0, *y),
        MarkShape::Sector { path } => {
            let points = outline(path).concat();
            let count = points.len().max(1) as f32;
            let (sx, sy) = points
                .iter()
                .fold((0.0, 0.0), |(sx, sy), (x, y)| (sx + x, sy + y));
            (sx / count, sy / count)
        }
    }
}

/// An SVG path of moves, lines and circular arcs as closed polygons, arcs sampled.
pub(super) fn outline(path: &str) -> Vec<Vec<(f32, f32)>> {
    let numbers = |text: &str| -> Vec<f32> {
        text.split([' ', ','])
            .filter(|n| !n.is_empty())
            .filter_map(|n| n.parse().ok())
            .collect()
    };
    let mut rings: Vec<Vec<(f32, f32)>> = Vec::new();
    let mut at = (0.0, 0.0);
    let mut rest = path.trim();
    while let Some(command) = rest.chars().next() {
        let body_end = rest[1..]
            .find(|c: char| c.is_ascii_alphabetic())
            .map_or(rest.len(), |end| end + 1);
        let args = numbers(&rest[1..body_end]);
        match (command, args.as_slice()) {
            ('M', [x, y, ..]) => {
                at = (*x, *y);
                rings.push(vec![at]);
            }
            ('L', [x, y, ..]) => {
                at = (*x, *y);
                if let Some(ring) = rings.last_mut() {
                    ring.push(at);
                }
            }
            ('A', [r, _, _, large, sweep, x, y, ..]) => {
                let to = (*x, *y);
                if let Some(ring) = rings.last_mut() {
                    ring.extend(arc(at, to, *r, *large != 0.0, *sweep != 0.0));
                }
                at = to;
            }
            _ => {}
        }
        rest = rest[body_end..].trim_start();
    }
    rings
}

/// The points after `from` along a circular arc to `to`, as SVG's endpoint form draws it.
fn arc(from: (f32, f32), to: (f32, f32), r: f32, large: bool, sweep: bool) -> Vec<(f32, f32)> {
    let (hx, hy) = ((from.0 - to.0) / 2.0, (from.1 - to.1) / 2.0);
    let half = hx.hypot(hy);
    if half == 0.0 || r <= 0.0 {
        return vec![to];
    }
    let r = r.max(half);
    let sign = if large == sweep { -1.0 } else { 1.0 };
    let reach = sign * (r * r - half * half).max(0.0).sqrt() / half;
    let center = (
        reach * hy + (from.0 + to.0) / 2.0,
        -reach * hx + (from.1 + to.1) / 2.0,
    );
    let angle = |(x, y): (f32, f32)| (y - center.1).atan2(x - center.0);
    let start = angle(from);
    let mut delta = angle(to) - start;
    if sweep && delta < 0.0 {
        delta += TAU;
    } else if !sweep && delta > 0.0 {
        delta -= TAU;
    }
    let steps = ((delta.abs() / TAU * 64.0).ceil() as usize).max(2);
    (1..=steps)
        .map(|step| {
            let angle = start + delta * step as f32 / steps as f32;
            (center.0 + r * angle.cos(), center.1 + r * angle.sin())
        })
        .collect()
}

/// Even-odd over every ring, so a donut's hole is outside.
pub(super) fn contains(rings: &[Vec<(f32, f32)>], x: f32, y: f32) -> bool {
    let mut inside = false;
    for ring in rings {
        for (i, &(ax, ay)) in ring.iter().enumerate() {
            let (bx, by) = ring[(i + 1) % ring.len()];
            if (ay > y) != (by > y) && x < ax + (y - ay) / (by - ay) * (bx - ax) {
                inside = !inside;
            }
        }
    }
    inside
}

/// A key's step from the active mark, by its place in `marks`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Step {
    Next,
    Previous,
    /// The next series at the same x.
    Down,
    Up,
    First,
    Last,
}

impl Step {
    /// Along the series becomes across them, for a chart whose x axis runs down.
    pub(super) fn swapped(self) -> Self {
        match self {
            Self::Next => Self::Down,
            Self::Previous => Self::Up,
            Self::Down => Self::Next,
            Self::Up => Self::Previous,
            step => step,
        }
    }
}

/// The mark a key moves to. Along a series for lines and bars; a pie walks its slices
/// with every arrow.
pub(super) fn walk(marks: &[Mark], active: Option<usize>, step: Step) -> Option<usize> {
    let last = marks.len().checked_sub(1)?;
    let Some(active) = active.filter(|active| *active <= last) else {
        return Some(match step {
            Step::Previous | Step::Last => last,
            _ => 0,
        });
    };
    let flat = matches!(marks[0].shape, MarkShape::Sector { .. })
        || marks.iter().all(|mark| mark.index == marks[0].index);
    let current = &marks[active];
    let same_series = |mark: &&Mark| mark.series == current.series;
    let in_series = || {
        marks
            .iter()
            .enumerate()
            .filter(|(_, mark)| same_series(mark))
    };
    let position = |mark: &Mark| marks.iter().position(|other| other == mark);
    match (step, flat) {
        (Step::First, true) => Some(0),
        (Step::Last, true) => Some(last),
        (Step::Next | Step::Down, true) => Some((active + 1).min(last)),
        (Step::Previous | Step::Up, true) => Some(active.saturating_sub(1)),
        (Step::First, false) => in_series().map(|(i, _)| i).next(),
        (Step::Last, false) => in_series().map(|(i, _)| i).next_back(),
        (Step::Next, false) => in_series()
            .find(|(_, mark)| mark.index > current.index)
            .map_or(Some(active), |(i, _)| Some(i)),
        (Step::Previous, false) => in_series()
            .rfind(|(_, mark)| mark.index < current.index)
            .map_or(Some(active), |(i, _)| Some(i)),
        (Step::Down | Step::Up, false) => {
            let mut series: Vec<usize> = marks.iter().map(|mark| mark.series).collect();
            series.dedup();
            let at = series.iter().position(|s| *s == current.series)?;
            let target = match step {
                Step::Down => series.get(at + 1),
                _ => at.checked_sub(1).and_then(|at| series.get(at)),
            };
            let Some(&target) = target else {
                return Some(active);
            };
            marks
                .iter()
                .filter(|mark| mark.series == target)
                .min_by_key(|mark| mark.index.abs_diff(current.index))
                .and_then(position)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(series: usize, index: usize, x: f32, y: f32) -> Mark {
        Mark {
            series,
            index,
            value: 0.0,
            shape: MarkShape::Point { x, y },
        }
    }

    /// A scatter's marks sort left to right whatever the data order; the pointer picks the
    /// nearest by distance, the keys walk along and across to the nearest.
    #[test]
    fn scatter_points_walk_left_to_right_and_across_to_the_nearest() {
        let drawn = Drawn {
            svg: String::new(),
            geometry: super::super::engine::Geometry {
                width: 100.0,
                height: 100.0,
                plot: super::super::engine::Plot::whole(100.0, 100.0),
                marks: vec![
                    point(0, 0, 50.0, 50.0),
                    point(0, 1, 10.0, 90.0),
                    point(1, 0, 48.0, 10.0),
                    point(1, 1, 80.0, 52.0),
                ],
                hole: 0.0,
                share_of: None,
                cells: false,
            },
        };
        let marks = drawn.with_points(&[0, 1]).geometry.marks;
        let keys: Vec<_> = marks.iter().map(|m| (m.series, m.index)).collect();
        assert_eq!(keys, [(0, 1), (0, 0), (1, 0), (1, 1)]);
        // Nearest in the plane, not by column: (48, 10) is closer in x only.
        assert_eq!(nearest(&marks, 75.0, 50.0), Some(3));
        assert_eq!(walk_free(&marks, Some(0), Step::Next), Some(1));
        assert_eq!(walk_free(&marks, Some(1), Step::Next), Some(1));
        // Across to the nearest in x, however far in y.
        assert_eq!(walk_free(&marks, Some(1), Step::Down), Some(2));
        assert_eq!(walk_free(&marks, Some(3), Step::Up), Some(1));
        assert_eq!(walk_free(&marks, Some(3), Step::First), Some(2));
        assert_eq!(walk_free(&marks, None, Step::Last), Some(3));
    }

    /// As the engine harvests a pie: one series, a slice per index.
    fn sector(index: usize, path: &str) -> Mark {
        Mark {
            series: 0,
            index,
            value: 0.0,
            shape: MarkShape::Sector {
                path: path.to_string(),
            },
        }
    }

    /// Two lines over three x values; the second has a gap at the middle one.
    fn lines() -> Vec<Mark> {
        vec![
            point(0, 0, 10.0, 50.0),
            point(0, 1, 50.0, 40.0),
            point(0, 2, 90.0, 30.0),
            point(1, 0, 10.0, 80.0),
            point(1, 2, 90.0, 70.0),
        ]
    }

    #[test]
    fn a_pointer_picks_the_nearest_column_then_the_nearest_line() {
        let marks = lines();
        assert_eq!(hit(&marks, 12.0, 90.0), Some(3));
        assert_eq!(hit(&marks, 85.0, 0.0), Some(2));
        // The gap's column still holds the other line's point.
        assert_eq!(hit(&marks, 48.0, 80.0), Some(1));
    }

    #[test]
    fn a_pointer_over_a_bar_picks_it_else_the_nearest_in_x() {
        let rect = |series, x| Mark {
            series,
            index: 0,
            value: 0.0,
            shape: MarkShape::Rect {
                x,
                y: 20.0,
                width: 10.0,
                height: 80.0,
            },
        };
        let marks = [rect(0, 0.0), rect(1, 12.0)];
        assert_eq!(hit(&marks, 15.0, 50.0), Some(1));
        // Above both bars, nearer the first.
        assert_eq!(hit(&marks, 9.0, 5.0), Some(0));
    }

    /// A line over bars: its point wins near it, the bar under the pointer elsewhere.
    #[test]
    fn a_line_s_point_among_bars_wins_only_near_it() {
        let bar = Mark {
            series: 0,
            index: 0,
            value: 0.0,
            shape: MarkShape::Rect {
                x: 0.0,
                y: 20.0,
                width: 20.0,
                height: 80.0,
            },
        };
        let marks = [bar, point(1, 0, 10.0, 40.0)];
        assert_eq!(hit(&marks, 12.0, 45.0), Some(1));
        assert_eq!(hit(&marks, 12.0, 90.0), Some(0));
    }

    /// charts-rs' paths: a wedge from 12 to 3 o'clock, a donut's slice, a whole ring.
    #[test]
    fn a_pointer_over_a_slice_picks_it_and_a_donut_s_hole_picks_none() {
        let wedge = sector(0, "M100,50 A50 50 0 0 1 150,100 L100,100 Z");
        let rest = sector(1, "M150,100 A50 50 0 1 1 100,50 L100,100 Z");
        let marks = [wedge, rest];
        assert_eq!(hit(&marks, 130.0, 70.0), Some(0));
        assert_eq!(hit(&marks, 70.0, 130.0), Some(1));
        assert_eq!(hit(&marks, 5.0, 5.0), None);

        let donut = [sector(
            0,
            "M100,50 A50 50 0 0 1 150,100 L130,100 A30 30 0 0 0 100,70 Z",
        )];
        assert_eq!(hit(&donut, 135.0, 65.0), Some(0));
        assert_eq!(hit(&donut, 110.0, 90.0), None);

        let ring = [sector(
            0,
            "M100,50 A50 50 0 1 1 100,150 A50 50 0 1 1 100,50 Z M100,70 A30 30 0 1 0 100,130 A30 30 0 1 0 100,70 Z",
        )];
        assert_eq!(hit(&ring, 100.0, 140.0), Some(0));
        assert_eq!(hit(&ring, 100.0, 100.0), None);
    }

    #[test]
    fn a_slice_s_anchor_is_inside_it() {
        let path = "M100,50 A50 50 0 0 1 150,100 L100,100 Z";
        let (x, y) = anchor(&MarkShape::Sector {
            path: path.to_string(),
        });
        assert!(contains(&outline(path), x, y), "{x},{y}");
    }

    #[test]
    fn keys_walk_a_series_and_cross_to_the_next_at_the_same_x() {
        let marks = lines();
        assert_eq!(walk(&marks, None, Step::Next), Some(0));
        assert_eq!(walk(&marks, None, Step::Last), Some(4));
        assert_eq!(walk(&marks, Some(0), Step::Next), Some(1));
        assert_eq!(walk(&marks, Some(2), Step::Next), Some(2));
        assert_eq!(walk(&marks, Some(0), Step::Last), Some(2));
        assert_eq!(walk(&marks, Some(4), Step::First), Some(3));
        assert_eq!(walk(&marks, Some(0), Step::Down), Some(3));
        // The second line has no point at x 1: the nearest one.
        assert_eq!(walk(&marks, Some(1), Step::Down), Some(3));
        assert_eq!(walk(&marks, Some(4), Step::Up), Some(2));
        assert_eq!(walk(&marks, Some(2), Step::Up), Some(2));
    }

    #[test]
    fn keys_walk_a_pie_slice_by_slice() {
        let path = "M0,0 Z";
        let marks = [sector(0, path), sector(1, path), sector(2, path)];
        assert_eq!(walk(&marks, Some(0), Step::Next), Some(1));
        assert_eq!(walk(&marks, Some(1), Step::Down), Some(2));
        assert_eq!(walk(&marks, Some(2), Step::Next), Some(2));
        assert_eq!(walk(&marks, Some(2), Step::Up), Some(1));
        assert_eq!(walk(&marks, Some(1), Step::Last), Some(2));
        assert_eq!(walk(&marks, Some(1), Step::First), Some(0));
    }

    #[test]
    fn swapped_keys_cross_the_series_sideways() {
        let marks = lines();
        assert_eq!(walk(&marks, Some(0), Step::Next.swapped()), Some(3));
        assert_eq!(walk(&marks, Some(0), Step::Down.swapped()), Some(1));
    }

    #[test]
    fn a_hidden_slice_keeps_the_others_places() {
        let drawn = Drawn {
            svg: String::new(),
            geometry: super::super::engine::Geometry {
                width: 1.0,
                height: 1.0,
                plot: super::super::engine::Plot::whole(1.0, 1.0),
                marks: vec![sector(0, "M0,0 Z"), sector(1, "M0,0 Z")],
                hole: 0.0,
                share_of: None,
                cells: false,
            },
        };
        let indices: Vec<usize> = drawn
            .with_indices(&[0, 2])
            .geometry
            .marks
            .iter()
            .map(|mark| mark.index)
            .collect();
        assert_eq!(indices, [0, 2]);
    }
}
