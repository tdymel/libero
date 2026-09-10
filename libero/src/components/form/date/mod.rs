//! Date and time pickers and fields over `chrono`'s naive types - a calendar
//! day, a wall-clock time, or both, with no time zone. What is ours is the
//! part `chrono` cannot do for a UI: theme-named formatting, lenient reading
//! of typed text, and the components.
//!
//! `DatePicker` and `DateField` hold every value type; the typed pickers and
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
mod today;

// The dioxus `Props` builders leak from every module that declares props.
#[allow(unnameable_types, reason = "todo 450")]
mod date_field;
#[allow(unnameable_types, reason = "todo 450")]
mod date_picker;
#[allow(unnameable_types, reason = "todo 450")]
mod fields;
#[allow(unnameable_types, reason = "todo 450")]
mod pickers;
#[allow(unnameable_types, reason = "todo 450")]
mod time_picker;

pub use calendar::DateLevel;
pub use date_field::{DateField, DateFieldProps};
pub use date_picker::{DatePicker, DatePickerProps};
pub use date_value::DateValue;
pub use fields::{
    DateRangeField, DateRangeFieldProps, DateTimeField, DateTimeFieldProps, DateTimeRangeField,
    DateTimeRangeFieldProps, DayField, DayFieldProps, TimeField, TimeFieldProps,
};
pub use pickers::{
    DateRangePicker, DateRangePickerProps, DayPicker, DayPickerProps, MonthPicker,
    MonthPickerProps, YearPicker, YearPickerProps,
};
pub use range::{DateRange, ParseRangeError};
pub use time_picker::{TimePicker, TimePickerProps};
