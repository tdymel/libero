//! The value maths and ARIA numbers `ProgressBar` and `CircularProgress` share.

use crate::utils::warn;

/// The drawn share, in `0.0..=1.0`. A broken range warns and draws empty.
pub(crate) fn fraction(component: &str, value: f64, min: f64, max: f64) -> f64 {
    if max <= min || !(min.is_finite() && max.is_finite()) {
        warn(&format!("{component}: `max` must be greater than `min`."));
        return 0.0;
    }
    if !value.is_finite() {
        warn(&format!("{component}: `value` must be a finite number."));
        return 0.0;
    }
    (value.clamp(min, max) - min) / (max - min)
}

/// `aria-valuenow`, clamped like the fill. A non-finite input reports `min`, as the
/// fill draws empty; `min > max` reports the raw value, as `clamp` panics on it.
pub(crate) fn value_now(value: f64, min: f64, max: f64) -> f64 {
    if !(value.is_finite() && min.is_finite() && max.is_finite()) {
        min
    } else if min < max {
        value.clamp(min, max)
    } else {
        value
    }
}

pub(crate) fn percentage(fraction: f64) -> String {
    format!("{}%", (fraction * 100.0).round())
}

/// Whole numbers without a decimal tail: "3 of 10", not "3 of 10.0". `None` for a
/// non-finite one, which ARIA cannot read ("inf", "NaN").
pub(crate) fn aria_number(value: f64) -> Option<String> {
    if !value.is_finite() {
        None
    } else if value.fract() == 0.0 && value.abs() < 1e15 {
        Some(format!("{}", value as i64))
    } else {
        Some(format!("{value}"))
    }
}

/// A colour var's hex in a theme sheet, following a var that names another var.
#[cfg(test)]
pub(crate) fn sheet_hex(css: &str, value: String) -> crate::theme::HexColor {
    let mut value = value;
    loop {
        let name = value.trim_start_matches("var(").trim_end_matches(')');
        let (_, rest) = css.split_once(&format!("{name}:")).expect("declared");
        let rest = rest.trim_start();
        match rest.strip_prefix("var(") {
            Some(next) => value = next.split(')').next().unwrap().to_string(),
            None => break crate::theme::HexColor::parse(&rest[..7]).expect("a hex"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fraction(value: f64, min: f64, max: f64) -> f64 {
        super::fraction("Test", value, min, max)
    }

    #[test]
    fn the_fraction_maps_the_range_onto_zero_to_one() {
        assert_eq!(fraction(0.0, 0.0, 100.0), 0.0);
        assert_eq!(fraction(50.0, 0.0, 100.0), 0.5);
        assert_eq!(fraction(100.0, 0.0, 100.0), 1.0);
    }

    #[test]
    fn a_range_that_is_not_zero_to_a_hundred() {
        assert_eq!(fraction(5.0, 0.0, 10.0), 0.5);
        assert_eq!(fraction(30.0, 20.0, 60.0), 0.25);
    }

    #[test]
    fn a_value_outside_the_range_is_clamped_not_wrapped() {
        assert_eq!(fraction(-40.0, 0.0, 100.0), 0.0);
        assert_eq!(fraction(140.0, 0.0, 100.0), 1.0);
    }

    #[test]
    fn a_max_at_or_below_min_draws_empty() {
        assert_eq!(fraction(5.0, 10.0, 10.0), 0.0);
        assert_eq!(fraction(5.0, 10.0, 0.0), 0.0);
    }

    #[test]
    fn non_finite_input_draws_empty_rather_than_emitting_nan_percent() {
        assert_eq!(fraction(f64::NAN, 0.0, 100.0), 0.0);
        assert_eq!(fraction(f64::INFINITY, 0.0, 100.0), 0.0);
        assert_eq!(fraction(0.0, 0.0, f64::INFINITY), 0.0);
    }

    #[test]
    fn aria_valuenow_is_clamped_like_the_fill() {
        assert_eq!(value_now(140.0, 0.0, 100.0), 100.0);
        assert_eq!(value_now(-5.0, 0.0, 100.0), 0.0);
        assert_eq!(value_now(42.0, 0.0, 100.0), 42.0);
        assert_eq!(value_now(5.0, 10.0, 0.0), 5.0);
        assert_eq!(value_now(f64::NAN, 0.0, 100.0), 0.0);
    }

    /// Todo 2400: `inf` drew empty and read "0%", but `aria-valuenow` said `max`.
    #[test]
    fn a_non_finite_value_or_bound_reports_what_the_fill_draws() {
        for (value, max) in [
            (f64::INFINITY, 100.0),
            (f64::NEG_INFINITY, 100.0),
            (50.0, f64::INFINITY),
        ] {
            assert_eq!(fraction(value, 0.0, max), 0.0);
            assert_eq!(value_now(value, 0.0, max), 0.0, "{value} of {max}");
        }
    }

    #[test]
    fn the_percentage_is_rounded_to_whole_units() {
        assert_eq!(percentage(0.0), "0%");
        assert_eq!(percentage(0.425), "43%");
        assert_eq!(percentage(1.0), "100%");
    }

    #[test]
    fn whole_aria_bounds_print_without_a_decimal_tail() {
        let number = |value| aria_number(value).expect("finite");
        assert_eq!(number(10.0), "10");
        assert_eq!(number(0.0), "0");
        assert_eq!(number(-3.0), "-3");
        assert_eq!(number(2.5), "2.5");
    }

    #[test]
    fn a_non_finite_aria_bound_is_omitted() {
        assert_eq!(aria_number(f64::INFINITY), None);
        assert_eq!(aria_number(f64::NEG_INFINITY), None);
        assert_eq!(aria_number(f64::NAN), None);
    }
}
