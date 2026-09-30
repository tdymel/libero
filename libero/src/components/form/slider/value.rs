//! The value scale, kept out of the component so it is testable without a DOM.

use crate::{components::common::decimals, utils::warn};

/// Rounds away the drift `min + step * n` accumulates - the `0.1 + 0.2` problem.
pub(super) fn round_to(value: f64, decimals: usize) -> f64 {
    let factor = 10_f64.powi(decimals as i32);
    (value * factor).round() / factor
}

/// Finite, ordered bounds for `snap`: `f64::clamp` panics otherwise, so a
/// broken range collapses to a point and draws empty.
pub(super) fn sane_bounds(min: f64, max: f64) -> (f64, f64) {
    if min.is_finite() && max.is_finite() && min <= max {
        return (min, max);
    }
    warn("Slider: `min` must be finite, and `max` finite and not below it.");
    let min = if min.is_finite() { min } else { 0.0 };
    let max = if max.is_finite() { max.max(min) } else { min };
    (min, max)
}

/// [`sane_bounds`] with `max` pulled down to the last `step` grid value, as a
/// native range input does: else the thumb stops short of the track's end (1558).
pub(super) fn grid_bounds(min: f64, max: f64, step: f64) -> (f64, f64) {
    let (min, max) = sane_bounds(min, max);
    (min, snap_towards(max, min, step.max(0.0), true))
}

/// `raw` snapped to the `step` grid from `min`, clamped and rounded; `step: 0`
/// is continuous. Bounds come from [`sane_bounds`]; `NaN` reads as `min`.
pub(super) fn snap(raw: f64, min: f64, max: f64, step: f64) -> f64 {
    if raw.is_nan() {
        return min;
    }
    if step <= 0.0 {
        return raw.clamp(min, max);
    }

    let value = min + step * ((raw - min) / step).round();
    // `min`'s precision counts too: `0.5 + 1.0 * n` at zero decimals leaves
    // the grid.
    round_to(value.clamp(min, max), decimals(step).max(decimals(min)))
}

/// `value` moved onto the `step` grid in one direction, `down` or up. A
/// `min_range` that is no multiple of `step` clamps a thumb between two grid points.
fn snap_towards(value: f64, min: f64, step: f64, down: bool) -> f64 {
    if step <= 0.0 {
        return value;
    }
    let steps = (value - min) / step;
    // A value already on the grid stays put despite float drift.
    let steps = match steps.round() {
        near if (steps - near).abs() < 1e-9 => near,
        _ if down => steps.floor(),
        _ => steps.ceil(),
    };
    round_to(min + step * steps, decimals(step).max(decimals(min)))
}

/// `value`'s position in `min..=max` as a 0-1 fraction. A zero-width range
/// has no position, so it reads as empty.
pub(super) fn fraction(value: f64, min: f64, max: f64) -> f64 {
    if max <= min {
        return 0.0;
    }
    ((value - min) / (max - min)).clamp(0.0, 1.0)
}

/// One thumb, or two: what `SliderCore` renders and emits. The skins differ
/// only in which of the two they build.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::components::form) enum SliderCoreValue {
    Single(f64),
    Range { from: f64, to: f64 },
}

impl SliderCoreValue {
    /// The filled bar's span, as 0-1 fractions of the track. A single thumb
    /// fills from the track's start; a range fills between its two thumbs.
    pub(super) fn bar(self, min: f64, max: f64) -> (f64, f64) {
        match self {
            Self::Single(value) => (0.0, fraction(value, min, max)),
            Self::Range { from, to } => (fraction(from, min, max), fraction(to, min, max)),
        }
    }

    /// Every thumb, in track order.
    pub(super) fn thumbs(self) -> impl Iterator<Item = f64> {
        let (first, second) = match self {
            Self::Single(value) => (value, None),
            Self::Range { from, to } => (from, Some(to)),
        };
        std::iter::once(first).chain(second)
    }

    pub(in crate::components::form) fn thumb(self, index: usize) -> f64 {
        match (self, index) {
            (Self::Single(value), _) | (Self::Range { from: value, .. }, 0) => value,
            (Self::Range { to, .. }, _) => to,
        }
    }

    /// On the step grid and inside the range - and, for a range, ordered.
    pub(super) fn snapped(self, min: f64, max: f64, step: f64) -> Self {
        match self {
            Self::Single(value) => Self::Single(snap(value, min, max, step)),
            Self::Range { from, to } => {
                let from = snap(from, min, max, step);
                Self::Range {
                    from,
                    to: snap(to, min, max, step).max(from),
                }
            }
        }
    }

    /// Moves one thumb to `raw`, snapped and stopped `min_range` short of its
    /// neighbour, so the pair stays ordered.
    pub(super) fn moved(
        self,
        index: usize,
        raw: f64,
        min: f64,
        max: f64,
        step: f64,
        min_range: f64,
    ) -> Self {
        let value = snap(raw, min, max, step);
        match self {
            Self::Single(_) => Self::Single(value),
            // The track clamp beats `min_range`; the gap clamp snaps back
            // towards the thumb's origin, to stay on the grid.
            Self::Range { from, to } => match index {
                0 => Self::Range {
                    from: snap_towards(value.min(to - min_range), min, step, true).max(min),
                    to,
                },
                _ => Self::Range {
                    from,
                    to: snap_towards(value.max(from + min_range), min, step, false).min(max),
                },
            },
        }
    }

    /// The thumb a pointer aiming at `target` grabs: the closer one, else the
    /// one `heading` lies towards. `None` for thumbs on one spot while the
    /// pointer has not moved: its first move picks (1060).
    pub(super) fn nearest(self, target: f64, heading: f64) -> Option<usize> {
        let Self::Range { from, to } = self else {
            return Some(0);
        };
        let (to_from, to_to) = ((target - from).abs(), (target - to).abs());
        match () {
            _ if to_from < to_to => Some(0),
            _ if to_to < to_from => Some(1),
            _ if heading != target => Some(usize::from(heading > target)),
            _ if from == to => None,
            _ => Some(0),
        }
    }

    /// `aria-valuemin`/`aria-valuemax` for one thumb, which is how far it can
    /// actually travel rather than how wide the track is.
    pub(super) fn bounds(
        self,
        index: usize,
        min: f64,
        max: f64,
        step: f64,
        min_range: f64,
    ) -> (f64, f64) {
        match (self, index) {
            (Self::Single(_), _) => (min, max),
            (Self::Range { to, .. }, 0) => {
                (min, snap_towards(to - min_range, min, step, true).max(min))
            }
            (Self::Range { from, .. }, _) => (
                snap_towards(from + min_range, min, step, false).min(max),
                max,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_value_snaps_to_the_step_grid_from_min() {
        assert_eq!(snap(7.0, 0.0, 100.0, 5.0), 5.0);
        assert_eq!(snap(8.0, 0.0, 100.0, 5.0), 10.0);
        assert_eq!(snap(7.0, 1.0, 100.0, 5.0), 6.0);
    }

    #[test]
    fn snapping_keeps_the_steps_precision() {
        assert_eq!(snap(0.30000000000000004, 0.0, 1.0, 0.1), 0.3);
        assert_eq!(decimals(0.25), 2);
        assert_eq!(decimals(1.0), 0);
    }

    #[test]
    fn a_snapped_value_stays_inside_the_range() {
        assert_eq!(snap(-40.0, 0.0, 10.0, 3.0), 0.0);
        assert_eq!(snap(999.0, 0.0, 10.0, 3.0), 10.0);
    }

    /// `min: 0.5, step: 1.0` rounded to `step`'s zero decimals, which moved
    /// every value a whole step and made the arrow key jump two.
    #[test]
    fn an_offset_min_keeps_its_own_precision() {
        assert_eq!(snap(0.5, 0.5, 10.0, 1.0), 0.5);
        assert_eq!(snap(1.5, 0.5, 10.0, 1.0), 1.5);
        assert_eq!(snap(2.4, 0.5, 10.0, 1.0), 2.5);
        assert_eq!(snap(0.1, 0.05, 1.0, 0.1), 0.15);
    }

    /// `Slider { max: -10.0 }` alone is already inverted, because `min`
    /// defaults to 0 - and `f64::clamp` panics on that.
    #[test]
    fn a_broken_range_collapses_instead_of_panicking() {
        assert_eq!(sane_bounds(0.0, 100.0), (0.0, 100.0));
        assert_eq!(sane_bounds(0.0, -10.0), (0.0, 0.0));
        assert_eq!(sane_bounds(10.0, 0.0), (10.0, 10.0));
        assert_eq!(sane_bounds(f64::NAN, 100.0), (0.0, 100.0));
        assert_eq!(sane_bounds(5.0, f64::NAN), (5.0, 5.0));
        assert_eq!(sane_bounds(f64::NEG_INFINITY, f64::INFINITY), (0.0, 0.0));
        // A zero-width range is legal input, not a repair.
        assert_eq!(sane_bounds(5.0, 5.0), (5.0, 5.0));
    }

    /// The whole point of the repair: these called `f64::clamp` and took the
    /// page down.
    #[test]
    fn a_repaired_range_snaps_without_panicking() {
        let (min, max) = sane_bounds(0.0, -10.0);
        assert_eq!(snap(7.0, min, max, 1.0), 0.0);
        let (min, max) = sane_bounds(f64::NAN, f64::NAN);
        assert_eq!(snap(7.0, min, max, 0.0), 0.0);
    }

    /// 0..100 by 30 ends at 90: `snap(100)` lands there, so the track must too (1558).
    #[test]
    fn max_off_the_grid_ends_at_the_last_grid_value() {
        assert_eq!(grid_bounds(0.0, 100.0, 30.0), (0.0, 90.0));
        assert_eq!(grid_bounds(10.0, 110.0, 30.0), (10.0, 100.0));
        assert_eq!(grid_bounds(0.0, 1.0, 0.1), (0.0, 1.0));
        assert_eq!(grid_bounds(0.0, 100.0, 0.0), (0.0, 100.0));
        let (min, max) = grid_bounds(0.0, 100.0, 30.0);
        assert_eq!(fraction(snap(100.0, min, max, 30.0), min, max), 1.0);
    }

    /// A `NaN` value would otherwise reach the CSS as `NaN%`.
    #[test]
    fn a_nan_value_reads_as_min() {
        assert_eq!(snap(f64::NAN, 5.0, 10.0, 1.0), 5.0);
        assert_eq!(snap(f64::NAN, 5.0, 10.0, 0.0), 5.0);
        assert_eq!(fraction(snap(f64::NAN, 5.0, 10.0, 1.0), 5.0, 10.0), 0.0);
    }

    #[test]
    fn a_zero_step_is_continuous_and_unrounded() {
        assert_eq!(snap(3.7, 0.0, 10.0, 0.0), 3.7);
        assert_eq!(snap(-1.0, 0.0, 10.0, 0.0), 0.0);
    }

    #[test]
    fn an_empty_range_has_no_position() {
        assert_eq!(fraction(5.0, 5.0, 5.0), 0.0);
        assert_eq!(fraction(-1.0, 0.0, 10.0), 0.0);
        assert_eq!(fraction(11.0, 0.0, 10.0), 1.0);
        assert_eq!(fraction(2.5, 0.0, 10.0), 0.25);
    }

    #[test]
    fn a_range_thumb_stops_at_its_neighbour() {
        let value = SliderCoreValue::Range {
            from: 20.0,
            to: 80.0,
        };
        assert_eq!(
            value.moved(0, 95.0, 0.0, 100.0, 1.0, 0.0),
            SliderCoreValue::Range {
                from: 80.0,
                to: 80.0
            }
        );
        assert_eq!(
            value.moved(1, 5.0, 0.0, 100.0, 1.0, 0.0),
            SliderCoreValue::Range {
                from: 20.0,
                to: 20.0
            }
        );
    }

    #[test]
    fn a_min_range_keeps_the_thumbs_apart() {
        let value = SliderCoreValue::Range {
            from: 20.0,
            to: 80.0,
        };
        assert_eq!(
            value.moved(0, 95.0, 0.0, 100.0, 1.0, 10.0),
            SliderCoreValue::Range {
                from: 70.0,
                to: 80.0
            }
        );
        assert_eq!(
            value.moved(1, 5.0, 0.0, 100.0, 1.0, 10.0),
            SliderCoreValue::Range {
                from: 20.0,
                to: 30.0
            }
        );
    }

    /// A gap wider than the range has no solution, so the track's own bounds
    /// win and the thumb still lands on the track.
    #[test]
    fn the_track_wins_over_an_impossible_gap() {
        let value = SliderCoreValue::Range { from: 0.0, to: 5.0 };
        assert_eq!(
            value.moved(0, 4.0, 0.0, 100.0, 1.0, 10.0),
            SliderCoreValue::Range { from: 0.0, to: 5.0 }
        );
    }

    #[test]
    fn a_pointer_grabs_the_closer_thumb() {
        let value = SliderCoreValue::Range {
            from: 20.0,
            to: 80.0,
        };
        assert_eq!(value.nearest(30.0, 30.0), Some(0));
        assert_eq!(value.nearest(70.0, 70.0), Some(1));
        // Equidistant: the side the pointer moves to.
        assert_eq!(value.nearest(50.0, 55.0), Some(1));
        assert_eq!(value.nearest(50.0, 45.0), Some(0));
        // Collapsed thumbs part in whichever direction the pointer went.
        let closed = SliderCoreValue::Range {
            from: 50.0,
            to: 50.0,
        };
        // Pressed on the pair: open until it moves (1060).
        assert_eq!(closed.nearest(51.0, 51.0), None);
        assert_eq!(closed.nearest(50.0, 54.0), Some(1));
        assert_eq!(closed.nearest(50.0, 46.0), Some(0));
        assert_eq!(closed.nearest(51.0, 49.0), Some(0));
        assert_eq!(SliderCoreValue::Single(5.0).nearest(9.0, 9.0), Some(0));
    }

    #[test]
    fn a_range_snaps_both_thumbs_and_keeps_them_ordered() {
        let value = SliderCoreValue::Range { from: 7.0, to: 3.0 };
        assert_eq!(
            value.snapped(0.0, 100.0, 5.0),
            SliderCoreValue::Range { from: 5.0, to: 5.0 }
        );
    }

    #[test]
    fn a_thumb_is_bounded_by_its_neighbour() {
        let value = SliderCoreValue::Range {
            from: 20.0,
            to: 80.0,
        };
        assert_eq!(value.bounds(0, 0.0, 100.0, 1.0, 0.0), (0.0, 80.0));
        assert_eq!(value.bounds(1, 0.0, 100.0, 1.0, 0.0), (20.0, 100.0));
        assert_eq!(value.bounds(0, 0.0, 100.0, 1.0, 10.0), (0.0, 70.0));
        assert_eq!(
            SliderCoreValue::Single(1.0).bounds(0, 0.0, 10.0, 1.0, 0.0),
            (0.0, 10.0)
        );
    }

    /// `step: 10, min_range: 3` clamped `from` to 47, off the grid the docs promise.
    #[test]
    fn an_off_grid_min_range_snaps_back_towards_the_thumbs_origin() {
        let value = SliderCoreValue::Range {
            from: 20.0,
            to: 50.0,
        };
        assert_eq!(
            value.moved(0, 45.0, 0.0, 100.0, 10.0, 3.0),
            SliderCoreValue::Range {
                from: 40.0,
                to: 50.0
            }
        );
        assert_eq!(
            value.moved(1, 25.0, 0.0, 100.0, 10.0, 3.0),
            SliderCoreValue::Range {
                from: 20.0,
                to: 30.0
            }
        );
        // A move that stays clear of the gap is untouched.
        assert_eq!(
            value.moved(0, 30.0, 0.0, 100.0, 10.0, 3.0),
            SliderCoreValue::Range {
                from: 30.0,
                to: 50.0
            }
        );
        assert_eq!(value.bounds(0, 0.0, 100.0, 10.0, 3.0), (0.0, 40.0));
        assert_eq!(value.bounds(1, 0.0, 100.0, 10.0, 3.0), (30.0, 100.0));
    }

    /// The grid runs from `min`, and keeps its precision after the second snap.
    #[test]
    fn the_second_snap_keeps_the_grid_from_min() {
        let value = SliderCoreValue::Range { from: 0.5, to: 0.9 };
        assert_eq!(
            value.moved(0, 0.9, 0.1, 1.0, 0.2, 0.05),
            SliderCoreValue::Range { from: 0.7, to: 0.9 }
        );
    }

    #[test]
    fn a_range_fills_between_its_thumbs() {
        let value = SliderCoreValue::Range {
            from: 20.0,
            to: 80.0,
        };
        assert_eq!(value.bar(0.0, 100.0), (0.2, 0.8));
        assert_eq!(SliderCoreValue::Single(25.0).bar(0.0, 100.0), (0.0, 0.25));
    }
}
