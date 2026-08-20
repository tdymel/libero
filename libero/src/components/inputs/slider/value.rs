//! The value scale, kept out of the component so it is testable without a DOM.

/// Decimals `step` is written with - `0.1` -> 1. Rust's shortest-repr `f64`
/// formatting is what makes this exact.
pub(super) fn decimals(step: f64) -> usize {
    format!("{step}")
        .split_once('.')
        .map_or(0, |(_, fraction)| fraction.len())
}

/// Rounds away the drift `min + step * n` accumulates - the `0.1 + 0.2` problem.
pub(super) fn round_to(value: f64, decimals: usize) -> f64 {
    let factor = 10_f64.powi(decimals as i32);
    (value * factor).round() / factor
}

/// The nearest valid value to `raw`: snapped to the `step` grid from `min`,
/// clamped, and rounded to that grid's precision.
///
/// `step: 0` is continuous - no grid, and no rounding either, or every value
/// would land on a whole number.
pub(super) fn snap(raw: f64, min: f64, max: f64, step: f64) -> f64 {
    if step <= 0.0 {
        return raw.clamp(min, max);
    }

    let value = min + step * ((raw - min) / step).round();
    // The grid is `min + step * n`, so `min`'s precision counts as much as
    // `step`'s: rounding `0.5 + 1.0 * n` to `step`'s zero decimals would walk
    // the value off its own grid.
    round_to(value.clamp(min, max), decimals(step).max(decimals(min)))
}

/// `value`'s position in `min..=max` as a 0-1 fraction. A zero-width range
/// has no position, so it reads as empty.
pub(super) fn fraction(value: f64, min: f64, max: f64) -> f64 {
    if max <= min {
        return 0.0;
    }
    ((value - min) / (max - min)).clamp(0.0, 1.0)
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
}
