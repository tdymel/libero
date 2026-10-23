//! A numeric field's value: parse, format, step, clamp, over the caller's own type.

use std::fmt::Display;
use std::ops::{Add, Sub};
use std::str::FromStr;

/// A value a `NumberField` can hold. Every primitive number implements it; a custom
/// type needs only [`NumberValue::default_step`] on top of `FromStr` and `Display`.
///
/// Whole cents shown with a decimal point:
///
/// ```rust
/// # use dioxus::prelude::*;
/// use libero::components::{NumberField, NumberValue};
/// use std::fmt;
///
/// #[derive(Clone, Copy, PartialEq, PartialOrd)]
/// struct Cents(i64);
///
/// impl std::ops::Add for Cents {
///     type Output = Self;
///     fn add(self, other: Self) -> Self {
///         Cents(self.0.saturating_add(other.0))
///     }
/// }
///
/// impl std::ops::Sub for Cents {
///     type Output = Self;
///     fn sub(self, other: Self) -> Self {
///         Cents(self.0.saturating_sub(other.0))
///     }
/// }
///
/// impl std::str::FromStr for Cents {
///     type Err = &'static str;
///     fn from_str(text: &str) -> Result<Self, Self::Err> {
///         let number = text.parse::<f64>().map_err(|_| "not a number")?;
///         // `f64` parses "nan" and "inf" too.
///         if !number.is_finite() {
///             return Err("not a finite number");
///         }
///         Ok(Cents((number * 100.0).round() as i64))
///     }
/// }
///
/// impl fmt::Display for Cents {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         let sign = if self.0 < 0 { "-" } else { "" };
///         let cents = self.0.unsigned_abs();
///         write!(f, "{sign}{}.{:02}", cents / 100, cents % 100)
///     }
/// }
///
/// impl NumberValue for Cents {
///     fn default_step() -> Self {
///         Cents(50)
///     }
/// }
///
/// # fn app() -> Element {
/// let mut price = use_signal(|| None::<Cents>);
/// rsx! {
///     NumberField {
///         label: "Price",
///         placeholder: "0.00",
///         value: price(),
///         onchange: move |next| price.set(next),
///     }
/// }
/// # }
/// ```
pub trait NumberValue:
    Copy + PartialOrd + Add<Output = Self> + Sub<Output = Self> + FromStr + Display + 'static
{
    /// The step a stepper press moves by when the caller names none.
    fn default_step() -> Self;

    /// The edit buffer's value, `None` while it is not yet a number (`"-"`; `"1."` for an integer).
    fn parse(text: &str) -> Option<Self> {
        text.trim().parse().ok()
    }

    /// What the control shows. Override when display differs from parse.
    fn format(&self) -> String {
        self.to_string()
    }

    /// Where a stepper starts on an empty field. `None`: the press does nothing.
    fn zero() -> Option<Self> {
        Self::parse("0")
    }

    fn step_up(self, by: Self) -> Self {
        self + by
    }

    fn step_down(self, by: Self) -> Self {
        self - by
    }

    /// Clamped into the field's range; `max` wins over a larger `min`, as natively.
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
pub(crate) fn decimals(value: impl Display) -> usize {
    value
        .to_string()
        .split_once('.')
        .map_or(0, |(_, fraction)| fraction.len())
}

/// `sum` rounded to the finer precision of `value` and `by`: `0.1 + 0.2` is `0.3`.
fn rounded<T: Copy + Display + FromStr>(sum: T, value: T, by: T) -> T {
    let places = decimals(value).max(decimals(by));
    format!("{sum:.places$}").parse().unwrap_or(sum)
}

/// Integers saturate when they step, rather than overflow; floats round to their precision.
macro_rules! number_value {
    (float: $($float:ty),+; int: $($int:ty),+ $(,)?) => {
        $(
            impl NumberValue for $float {
                fn default_step() -> Self {
                    1.0
                }

                /// Rejects `nan` and `inf`, which `from_str` accepts: they slip past
                /// `clamp_between` and jam the steppers.
                fn parse(text: &str) -> Option<Self> {
                    text.trim().parse::<Self>().ok().filter(|value| value.is_finite())
                }

                fn step_up(self, by: Self) -> Self {
                    let next = rounded(self + by, self, by);
                    if next.is_finite() { next } else { self }
                }

                fn step_down(self, by: Self) -> Self {
                    let next = rounded(self - by, self, by);
                    if next.is_finite() { next } else { self }
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

    /// `NaN` passed both bounds, showed as `aria-valuenow="NaN"` and jammed the steppers.
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

    #[test]
    fn a_float_step_past_its_range_keeps_the_value() {
        assert_eq!(f64::MIN.step_down(f64::MAX), f64::MIN);
        assert_eq!(f32::MAX.step_up(f32::MAX), f32::MAX);
    }
}
