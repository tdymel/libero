//! Date and time pickers and fields over `chrono`'s naive types - a calendar
//! day, a wall-clock time, or both, with no time zone - and over a
//! `TimeDelta`, a duration. What is ours is the
//! part `chrono` cannot do for a UI: theme-named formatting, lenient reading
//! of typed text, and the components.
//!
//! `ChronoPicker` and `ChronoField` hold every value type; the typed pickers and
//! fields are the same components for one value type each.

// A pub type in a private module shows up in rustdoc but cannot be named.
#![deny(unnameable_types)]

mod calendar;
mod date_value;
mod flows;
mod format;
#[cfg(test)]
mod locales;
mod parse;
mod parse_time;
mod picker_field;
mod props;
mod range;
mod spin_column;
mod today;

// The deny above overrides the crate's `allow`, so the modules whose `Props`
// derive leaks builders allow it again (todo 450).
#[allow(unnameable_types)]
mod chrono_field;
#[allow(unnameable_types)]
mod chrono_picker;
#[allow(unnameable_types)]
mod duration;
#[allow(unnameable_types)]
mod fields;
#[allow(unnameable_types)]
mod pickers;
#[allow(unnameable_types)]
mod time_picker;

pub use calendar::DateLevel;
pub use chrono_field::{ChronoField, ChronoFieldProps};
pub use chrono_picker::{ChronoPicker, ChronoPickerProps};
pub use date_value::DateValue;
pub use fields::{
    DateField, DateFieldProps, DateRangeField, DateRangeFieldProps, DateTimeField,
    DateTimeFieldProps, DateTimeRangeField, DateTimeRangeFieldProps, TimeField, TimeFieldProps,
};
pub use pickers::{
    DatePicker, DatePickerProps, DateRangePicker, DateRangePickerProps, MonthPicker,
    MonthPickerProps, YearPicker, YearPickerProps,
};
pub use range::{DateRange, ParseRangeError};
pub use time_picker::{TimePicker, TimePickerProps};
