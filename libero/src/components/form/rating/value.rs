//! A rating's value maths: pure, so unit-tested here.

/// Absorbs float noise from `value * fractions`: 0.1 * 3 is 0.30000000000000004.
const EPSILON: f64 = 1e-9;

/// `value` within `0..=count`; NaN reads as unrated.
pub(super) fn sane(value: f64, count: u8) -> f64 {
    if value.is_nan() {
        return 0.0;
    }
    value.clamp(0.0, f64::from(count))
}

/// What a press `along` the row picks, 0 at its inline start and 1 at its end:
/// the grid step it falls in, never below one step.
pub(super) fn value_at(along: f64, count: u8, fractions: u8) -> f64 {
    let steps = f64::from(fractions);
    let raw = along.clamp(0.0, 1.0) * f64::from(count) * steps;
    ((raw - EPSILON).ceil().max(1.0) / steps).min(f64::from(count))
}

/// The value a symbol's `zone`-th slice picks: symbol 0, slice 0 of halves is 0.5.
pub(super) fn zone_value(symbol: u8, zone: u8, fractions: u8) -> f64 {
    f64::from(symbol) + f64::from(zone + 1) / f64::from(fractions)
}

/// How much of symbol `index` is filled, 0 to 1.
pub(super) fn fill(value: f64, index: u8) -> f64 {
    (value - f64::from(index)).clamp(0.0, 1.0)
}

/// `value` moved `delta` grid steps within `lowest..=count`. An off-grid value
/// first snaps the way it moves: 3.7 up a whole star is 4, down is 3.
pub(super) fn stepped(value: f64, delta: i32, fractions: u8, lowest: f64, count: u8) -> f64 {
    let steps = f64::from(fractions);
    let grid = value * steps;
    let base = if delta > 0 {
        (grid + EPSILON).floor()
    } else {
        (grid - EPSILON).ceil()
    };
    ((base + f64::from(delta)) / steps).clamp(lowest, f64::from(count))
}

/// `value` with at most two decimals, trailing zeros dropped: `3`, `3.5`, `4.25`.
pub(super) fn number_text(value: f64, separator: &str) -> String {
    let text = format!("{:.2}", (value * 100.0).round() / 100.0);
    let text = text.trim_end_matches('0').trim_end_matches('.');
    text.replacen('.', separator, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_press_picks_the_step_it_lands_in() {
        assert_eq!(value_at(0.0, 5, 1), 1.0);
        assert_eq!(value_at(0.05, 5, 2), 0.5);
        assert_eq!(value_at(0.15, 5, 2), 1.0);
        assert_eq!(value_at(0.5, 5, 1), 3.0);
        assert_eq!(value_at(1.0, 5, 2), 5.0);
        // Exactly on a boundary stays in the lower step.
        assert_eq!(value_at(0.4, 5, 1), 2.0);
        assert_eq!(value_at(1.5, 5, 1), 5.0);
        assert_eq!(value_at(-1.0, 5, 1), 1.0);
    }

    #[test]
    fn a_zone_is_its_slice_end() {
        assert_eq!(zone_value(0, 0, 2), 0.5);
        assert_eq!(zone_value(0, 1, 2), 1.0);
        assert_eq!(zone_value(2, 0, 1), 3.0);
        assert_eq!(zone_value(3, 2, 4), 3.75);
    }

    #[test]
    fn symbols_fill_up_to_the_value() {
        assert_eq!(fill(3.5, 0), 1.0);
        assert_eq!(fill(3.5, 3), 0.5);
        assert_eq!(fill(3.5, 4), 0.0);
    }

    #[test]
    fn steps_snap_the_way_they_move() {
        assert_eq!(stepped(3.0, 1, 1, 1.0, 5), 4.0);
        assert_eq!(stepped(3.7, 1, 1, 1.0, 5), 4.0);
        assert_eq!(stepped(3.7, -1, 1, 1.0, 5), 3.0);
        assert_eq!(stepped(0.0, 1, 2, 0.5, 5), 0.5);
        assert_eq!(stepped(0.5, -1, 2, 0.5, 5), 0.5);
        assert_eq!(stepped(0.5, -1, 2, 0.0, 5), 0.0);
        assert_eq!(stepped(4.5, 2, 2, 0.5, 5), 5.0);
        assert_eq!(stepped(0.3, 1, 10, 0.1, 5), 0.4);
    }

    #[test]
    fn out_of_range_values_clamp() {
        assert_eq!(sane(f64::NAN, 5), 0.0);
        assert_eq!(sane(-1.0, 5), 0.0);
        assert_eq!(sane(7.0, 5), 5.0);
    }

    #[test]
    fn numbers_print_short_with_the_separator() {
        assert_eq!(number_text(3.0, "."), "3");
        assert_eq!(number_text(3.5, ","), "3,5");
        assert_eq!(number_text(4.25, "."), "4.25");
        assert_eq!(number_text(1.0 / 3.0, "."), "0.33");
        assert_eq!(number_text(0.0, "."), "0");
    }
}
