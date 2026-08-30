//! The typed date and time fields: each names its own value type and fills
//! the part of [`FieldOptions`] that type uses, then draws through the same
//! path as `DateField`.

use dioxus::prelude::*;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use super::{
    DateRange,
    date_field::{FieldOptions, date_field},
    format::uses_twelve_hours,
    picker_field::picker_field,
    props::date_props,
};
use crate::{components::Input, theme::DateDefaults};

/// Whether a day passes `min`, `max` and `exclude_date`.
pub(super) fn day_allowed(
    min: Option<NaiveDate>,
    max: Option<NaiveDate>,
    exclude_date: Option<Callback<NaiveDate, bool>>,
) -> impl Fn(NaiveDate) -> bool + Copy + 'static {
    move |day| {
        !(min.is_some_and(|min| day < min)
            || max.is_some_and(|max| day > max)
            || exclude_date.is_some_and(|exclude| exclude.call(day)))
    }
}

/// Whether a moment passes `min`, `max`, and `exclude_date` on its day.
pub(super) fn moment_allowed(
    min: Option<NaiveDateTime>,
    max: Option<NaiveDateTime>,
    exclude_date: Option<Callback<NaiveDate, bool>>,
) -> impl Fn(NaiveDateTime) -> bool + Copy + 'static {
    move |moment| {
        !(min.is_some_and(|min| moment < min)
            || max.is_some_and(|max| moment > max)
            || exclude_date.is_some_and(|exclude| exclude.call(moment.date())))
    }
}

/// The time format a field shows when the caller names none: the theme's,
/// unless the caller asks for seconds or the other clock.
pub(super) fn time_format(
    names: &DateDefaults,
    twelve_hour: Option<bool>,
    with_seconds: bool,
) -> String {
    match (twelve_hour, with_seconds) {
        (None, false) => names.time_format.to_string(),
        (twelve, seconds) => {
            let twelve = twelve.unwrap_or_else(|| uses_twelve_hours(names.time_format));
            match (twelve, seconds) {
                (false, false) => "HH:mm",
                (false, true) => "HH:mm:ss",
                (true, false) => "h:mm A",
                (true, true) => "h:mm:ss A",
            }
            .to_string()
        }
    }
}

date_props! {
    field DayFieldProps(NaiveDate, NaiveDate): format, limits, exclude_date, today, close_on_change
}

/// A text field holding a day, with a `DayPicker` in a dropdown - Mantine's
/// `DateInput`.
///
/// Controlled: it renders `value` and asks for a new one through `onchange`.
/// Typed text stays as typed until the field blurs or Enter is pressed; then
/// it is read leniently against `format`. Text that is not an accepted day
/// stays, and the field shows an error.
#[component]
pub fn DayField(props: DayFieldProps) -> Element {
    let options = FieldOptions {
        format: props.format.clone(),
        min: props.min,
        max: props.max,
        exclude_date: props.exclude_date,
        close_on_change: props.close_on_change,
        ..FieldOptions::default()
    };
    date_field::<NaiveDate>(picker_field!(props, props.today), options)
}

date_props! {
    field TimeFieldProps(NaiveTime, NaiveTime): time_format, limits, clock
}

/// A text field holding a time, with a `TimePicker` in a dropdown - Mantine's
/// `TimeInput` with a picker. Typing reads `13:05`, `1:05 pm`, `1305`.
#[component]
pub fn TimeField(props: TimeFieldProps) -> Element {
    let options = FieldOptions {
        time_format: props.time_format.clone(),
        min: props.min,
        max: props.max,
        variant: props.variant.clone(),
        with_seconds: props.with_seconds,
        step: props.step,
        twelve_hour: props.twelve_hour,
        ..FieldOptions::default()
    };
    date_field::<NaiveTime>(picker_field!(props, None), options)
}

date_props! {
    field DateTimeFieldProps(NaiveDateTime, NaiveDateTime): format, time_format, limits, exclude_date, today, clock
}

/// A text field holding a day and a time. The dropdown picks the day, then
/// the time - a `SegmentedControl` goes back - Mantine's `DateTimePicker`.
/// The text shows the day, then the time after a space.
#[component]
pub fn DateTimeField(props: DateTimeFieldProps) -> Element {
    let options = FieldOptions {
        format: props.format.clone(),
        time_format: props.time_format.clone(),
        min: props.min,
        max: props.max,
        exclude_date: props.exclude_date,
        variant: props.variant.clone(),
        with_seconds: props.with_seconds,
        step: props.step,
        twelve_hour: props.twelve_hour,
        ..FieldOptions::default()
    };
    date_field::<NaiveDateTime>(picker_field!(props, props.today), options)
}

date_props! {
    field DateRangeFieldProps(DateRange<NaiveDate>, NaiveDate): format, limits, exclude_date, today, columns, close_on_change
}

/// A text field holding a range of days, with two months in a dropdown -
/// Mantine's `DatePickerInput type="range"`. An `end` of `None` is a range
/// still being picked. The text joins both days with the theme's
/// `range_separator`; typing takes `–`, ` - ` or ` to ` between them.
#[component]
pub fn DateRangeField(props: DateRangeFieldProps) -> Element {
    let options = FieldOptions {
        format: props.format.clone(),
        min: props.min,
        max: props.max,
        exclude_date: props.exclude_date,
        columns: props.columns,
        close_on_change: props.close_on_change,
        ..FieldOptions::default()
    };
    date_field::<DateRange<NaiveDate>>(picker_field!(props, props.today), options)
}

date_props! {
    field DateTimeRangeFieldProps(DateRange<NaiveDateTime>, NaiveDateTime): format, time_format, limits, exclude_date, today, clock
}

/// A text field holding a range of moments. The dropdown picks the start - a
/// day, then a time - before the end; `SegmentedControl`s switch sides and
/// parts.
#[component]
pub fn DateTimeRangeField(props: DateTimeRangeFieldProps) -> Element {
    let options = FieldOptions {
        format: props.format.clone(),
        time_format: props.time_format.clone(),
        min: props.min,
        max: props.max,
        exclude_date: props.exclude_date,
        variant: props.variant.clone(),
        with_seconds: props.with_seconds,
        step: props.step,
        twelve_hour: props.twelve_hour,
        ..FieldOptions::default()
    };
    date_field::<DateRange<NaiveDateTime>>(picker_field!(props, props.today), options)
}
