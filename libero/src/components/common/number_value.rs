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
    fn clamp_to(self, min: Option<Self>, max: Option<Self>) -> Self {
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

/// One line per primitive: everything but the step unit comes from the trait's
/// default bodies. An integer also saturates when it steps, so a press at the
/// edge of its type stays there instead of overflowing - a panic in a debug
/// build, a wrap-around in release.
macro_rules! number_value {
    (float: $($float:ty),+; int: $($int:ty),+ $(,)?) => {
        $(
            impl NumberValue for $float {
                fn default_step() -> Self {
                    1.0
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
