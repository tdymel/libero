//! What a numeric field's value is made of: parse, format, step, clamp. A
//! component is generic over it, so the value is the caller's own type rather
//! than an `f64` every call site converts back.

use std::fmt::Display;
use std::ops::{Add, Sub};
use std::str::FromStr;

/// A value a `NumberField` can hold.
///
/// Every primitive number implements it, so the common cases need no code at
/// all. A custom type - cents, a unit-tagged newtype - implements
/// [`NumberValue::default_step`] and inherits the rest from its `FromStr` and
/// `Display`, overriding [`NumberValue::format`] only when what it shows
/// differs from what it parses.
///
/// `f32` and `f64` follow `Formats::decimal_separator` in the field; a custom
/// type writes and reads its own separator in `format` and `parse`.
///
/// The bounds are what the field actually does with a value: compare it to
/// `min`/`max`, add and subtract a step, read it out of the edit buffer, and
/// write it back in.
pub trait NumberValue:
    Copy + PartialOrd + Add<Output = Self> + Sub<Output = Self> + FromStr + Display + 'static
{
    /// The step a press of the stepper moves by when the caller names none.
    /// The one thing std has no trait for, which is why it is the only
    /// required method.
    fn default_step() -> Self;

    /// The value the edit buffer holds, or `None` while the text is not yet a
    /// number - `""`, `"-"`, `"1."` as someone types `"1.5"`.
    fn parse(text: &str) -> Option<Self> {
        text.trim().parse().ok()
    }

    /// What the control shows when the value arrives from outside. Override it
    /// when the display differs from the parse - two decimal places, a unit.
    fn format(&self) -> String {
        self.to_string()
    }

    /// Where a stepper starts from when the field is empty. `None` means the
    /// press does nothing, which is the honest answer for a type with no zero.
    fn zero() -> Option<Self> {
        Self::parse("0")
    }

    fn step_up(self, by: Self) -> Self {
        self + by
    }

    fn step_down(self, by: Self) -> Self {
        self - by
    }

    /// Clamped into the field's range. A `min` above `max` is the caller's
    /// error and `max` wins, the way a native input resolves it.
    fn clamp_between(self, min: Option<Self>, max: Option<Self>) -> Self {
        let mut value = self;
        if let Some(min) = min
            && value < min
        {
            value = min;
        }
        if let Some(max) = max
            && value > max
        {
            value = max;
        }
        value
    }
}

/// The digits after the point in `value`'s shortest form: 2 for `0.25`.
fn decimals(value: impl Display) -> usize {
    value
        .to_string()
        .split_once('.')
        .map_or(0, |(_, fraction)| fraction.len())
}

/// `sum` rounded to as many decimals as the finer of `value` and `by` has, so
/// binary noise does not reach the field: `0.1 + 0.2` is `0.3`, not
/// `0.30000000000000004`.
fn rounded<T: Copy + Display + FromStr>(sum: T, value: T, by: T) -> T {
    let places = decimals(value).max(decimals(by));
    format!("{sum:.places$}").parse().unwrap_or(sum)
}

/// One line per primitive: everything but the step unit comes from the trait's
/// default bodies. An integer also saturates when it steps, so a press at the
/// edge of its type stays there instead of overflowing - a panic in a debug
/// build, a wrap-around in release. A float rounds to the precision of its
/// step, or of its value where that is finer.
macro_rules! number_value {
    (float: $($float:ty),+; int: $($int:ty),+ $(,)?) => {
        $(
            impl NumberValue for $float {
                fn default_step() -> Self {
                    1.0
                }

                /// Rust's `from_str` accepts `nan`, `inf` and `infinity` in
                /// any case, and the field can do nothing sensible with
                /// either: every comparison with `NaN` is false, so it slips
                /// past both `min` and `max` in `clamp_between`, and once it is
                /// committed the steppers are stuck - `NaN + step` is `NaN`,
                /// and so is `inf - step`. So they are not a number.
                fn parse(text: &str) -> Option<Self> {
                    text.trim().parse::<Self>().ok().filter(|value| value.is_finite())
                }

                fn step_up(self, by: Self) -> Self {
                    rounded(self + by, self, by)
                }

                fn step_down(self, by: Self) -> Self {
                    rounded(self - by, self, by)
                }
            }
        )+
        $(
            impl NumberValue for $int {
                fn default_step() -> Self {
                    1
                }

                fn step_up(self, by: Self) -> Self {
                    self.saturating_add(by)
                }

                fn step_down(self, by: Self) -> Self {
                    self.saturating_sub(by)
                }
            }
        )+
    };
}

number_value! {
    float: f32, f64;
    int: i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize,
}

#[cfg(test)]
mod tests {
    use super::NumberValue;

    /// `f64::from_str` accepts them, `clamp_between` cannot hold them - every
    /// comparison with `NaN` is false, so it passed both bounds, and `inf`
    /// passed an open `max`. The field then showed `NaN`, reported
    /// `aria-valuenow="NaN"`, and its steppers were stuck.
    #[test]
    fn a_float_field_does_not_accept_nan_or_infinity() {
        for text in ["nan", "NaN", "inf", "-inf", "infinity", "INFINITY"] {
            assert_eq!(<f64 as NumberValue>::parse(text), None, "{text}");
            assert_eq!(<f32 as NumberValue>::parse(text), None, "{text}");
        }

        assert_eq!(<f64 as NumberValue>::parse(" 1.5 "), Some(1.5));
        assert_eq!(<f64 as NumberValue>::parse("1."), Some(1.0));
        assert_eq!(<f64 as NumberValue>::parse("-"), None);
        assert_eq!(<f64 as NumberValue>::zero(), Some(0.0));
    }
}
