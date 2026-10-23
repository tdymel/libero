//! Date and time pickers and fields over `chrono`'s naive types and `TimeDelta`.
//! `ChronoPicker` and `ChronoField` hold every value type; the typed ones hold one each.
//! The pickers compile under `date-picker`; the fields and their text under `date-field`.

#![deny(unnameable_types)]

mod calendar;
mod date_value;
#[cfg(feature = "date-field")]
mod duration_field;
#[cfg(feature = "date-field")]
mod field_value;
mod flows;
mod format;
#[cfg(all(test, feature = "date-field"))]
mod locales;
#[cfg(feature = "date-field")]
mod parse;
#[cfg(feature = "date-field")]
mod parse_time;
#[cfg(feature = "date-field")]
mod picker_field;
mod props;
mod range;
mod spin_column;
mod today;

// These `Props` derives leak builders, so they allow it again (todo 450).
#[cfg(feature = "date-field")]
#[allow(unnameable_types)]
mod chrono_field;
#[allow(unnameable_types)]
mod chrono_picker;
#[allow(unnameable_types)]
mod duration;
#[cfg(feature = "date-field")]
#[allow(unnameable_types)]
mod fields;
#[allow(unnameable_types)]
mod pickers;
#[allow(unnameable_types)]
mod time_picker;

pub use calendar::DateLevel;
#[cfg(feature = "date-field")]
pub use chrono_field::{ChronoField, ChronoFieldProps};
pub use chrono_picker::{ChronoPicker, ChronoPickerPart, ChronoPickerProps};
pub use date_value::DateValue;
#[cfg(feature = "date-field")]
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
