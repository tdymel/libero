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
};
use crate::{
    components::{FieldName, Input, Validators, common::field_props},
    theme::{DateDefaults, TimePickerVariant},
};

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

field_props! {
    extends(input);
    pub struct DayFieldProps {
        /// The day in the field; strictly controlled. `None` is the empty
        /// field. Inside a `Form`, a path `name` can supply it instead.
        #[props(default)]
        value: Option<NaiveDate>,
        /// Called with the day the caller should hold next: when typed text is
        /// committed - on blur or Enter - and when a day is picked. Emptied
        /// text commits `None`.
        #[props(default)]
        onchange: Option<EventHandler<Option<NaiveDate>>>,
        /// Rules over the day, shown once the field loses focus or its form is
        /// submitted.
        #[props(default, into)]
        validate: Validators<Option<NaiveDate>>,
        /// How the text shows the day, in dayjs tokens. Defaults to the
        /// theme's `DateDefaults::format`. Typing is lenient either way: only
        /// the order of day, month and year has to match.
        #[props(default, into)]
        format: Option<String>,
        /// The earliest day that can be picked or typed.
        #[props(default)]
        min: Option<NaiveDate>,
        /// The latest day that can be picked or typed.
        #[props(default)]
        max: Option<NaiveDate>,
        /// Days that cannot be picked or typed, on top of `min` and `max`.
        #[props(default)]
        exclude_date: Option<Callback<NaiveDate, bool>>,
        /// The day marked as today, and the year typed text without one
        /// falls back to. Unset, the platform clock answers after mount.
        #[props(default)]
        today: Option<NaiveDate>,
        /// Picking a day closes the dropdown.
        #[props(default)]
        close_on_change: Option<bool>,
        /// What the field posts as - the day in ISO 8601, whatever `format`
        /// shows. A path also binds it to the surrounding `Form`'s value when
        /// it has no `onchange`.
        #[props(default, into)]
        name: FieldName<Option<NaiveDate>>,
        #[props(default, into)]
        placeholder: Option<String>,
    }
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

field_props! {
    extends(input);
    pub struct TimeFieldProps {
        /// The time in the field; strictly controlled.
        #[props(default)]
        value: Option<NaiveTime>,
        /// Called when typed text is committed - on blur or Enter - and when a
        /// part is picked.
        #[props(default)]
        onchange: Option<EventHandler<Option<NaiveTime>>>,
        #[props(default, into)]
        validate: Validators<Option<NaiveTime>>,
        /// How the text shows the time, in dayjs tokens. Defaults to the
        /// theme's `DateDefaults::time_format`, adjusted for `with_seconds`
        /// and `twelve_hour`.
        #[props(default, into)]
        format: Option<String>,
        #[props(default)]
        min: Option<NaiveTime>,
        #[props(default)]
        max: Option<NaiveTime>,
        /// Columns of numbers, or a clock face.
        #[props(default, into)]
        variant: Input<TimePickerVariant>,
        #[props(default)]
        with_seconds: Option<bool>,
        /// Minutes between the offered minutes.
        #[props(default)]
        step: Option<u8>,
        #[props(default)]
        twelve_hour: Option<bool>,
        /// Posts the time as `HH:MM:SS`.
        #[props(default, into)]
        name: FieldName<Option<NaiveTime>>,
        #[props(default, into)]
        placeholder: Option<String>,
    }
}

/// A text field holding a time, with a `TimePicker` in a dropdown - Mantine's
/// `TimeInput` with a picker. Typing reads `13:05`, `1:05 pm`, `1305`.
#[component]
pub fn TimeField(props: TimeFieldProps) -> Element {
    let options = FieldOptions {
        time_format: props.format.clone(),
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

field_props! {
    extends(input);
    pub struct DateTimeFieldProps {
        /// The day and time in the field; strictly controlled.
        #[props(default)]
        value: Option<NaiveDateTime>,
        /// Called when typed text is committed and when a day or a time is
        /// picked.
        #[props(default)]
        onchange: Option<EventHandler<Option<NaiveDateTime>>>,
        #[props(default, into)]
        validate: Validators<Option<NaiveDateTime>>,
        /// How the text shows the day. The time follows it after a space.
        #[props(default, into)]
        format: Option<String>,
        /// How the text shows the time.
        #[props(default, into)]
        time_format: Option<String>,
        #[props(default)]
        min: Option<NaiveDateTime>,
        #[props(default)]
        max: Option<NaiveDateTime>,
        #[props(default)]
        exclude_date: Option<Callback<NaiveDate, bool>>,
        #[props(default)]
        today: Option<NaiveDate>,
        #[props(default, into)]
        variant: Input<TimePickerVariant>,
        #[props(default)]
        with_seconds: Option<bool>,
        #[props(default)]
        step: Option<u8>,
        #[props(default)]
        twelve_hour: Option<bool>,
        /// Posts `2026-09-14T13:05:00`.
        #[props(default, into)]
        name: FieldName<Option<NaiveDateTime>>,
        #[props(default, into)]
        placeholder: Option<String>,
    }
}

/// A text field holding a day and a time. The dropdown picks the day, then
/// the time - a `SegmentedControl` goes back - Mantine's `DateTimePicker`.
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

field_props! {
    extends(input);
    pub struct DateRangeFieldProps {
        /// The range in the field; strictly controlled. An `end` of `None` is
        /// a range still being picked.
        #[props(default)]
        value: Option<DateRange<NaiveDate>>,
        /// Called when typed text is committed and on every pick.
        #[props(default)]
        onchange: Option<EventHandler<Option<DateRange<NaiveDate>>>>,
        #[props(default, into)]
        validate: Validators<Option<DateRange<NaiveDate>>>,
        /// How the text shows each day; the theme's `range_separator` joins
        /// them. Typing takes `–`, ` - ` or ` to ` between them.
        #[props(default, into)]
        format: Option<String>,
        #[props(default)]
        min: Option<NaiveDate>,
        #[props(default)]
        max: Option<NaiveDate>,
        #[props(default)]
        exclude_date: Option<Callback<NaiveDate, bool>>,
        #[props(default)]
        today: Option<NaiveDate>,
        /// Months side by side in the dropdown. Two by default.
        #[props(default)]
        columns: Option<usize>,
        /// Picking the end closes the dropdown.
        #[props(default)]
        close_on_change: Option<bool>,
        /// Posts an ISO 8601 interval: `2026-09-01/2026-09-05`.
        #[props(default, into)]
        name: FieldName<Option<DateRange<NaiveDate>>>,
        #[props(default, into)]
        placeholder: Option<String>,
    }
}

/// A text field holding a range of days, with two months in a dropdown -
/// Mantine's `DatePickerInput type="range"`.
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

field_props! {
    extends(input);
    pub struct DateTimeRangeFieldProps {
        /// The range in the field; strictly controlled.
        #[props(default)]
        value: Option<DateRange<NaiveDateTime>>,
        /// Called when typed text is committed and on every pick.
        #[props(default)]
        onchange: Option<EventHandler<Option<DateRange<NaiveDateTime>>>>,
        #[props(default, into)]
        validate: Validators<Option<DateRange<NaiveDateTime>>>,
        #[props(default, into)]
        format: Option<String>,
        #[props(default, into)]
        time_format: Option<String>,
        #[props(default)]
        min: Option<NaiveDateTime>,
        #[props(default)]
        max: Option<NaiveDateTime>,
        #[props(default)]
        exclude_date: Option<Callback<NaiveDate, bool>>,
        #[props(default)]
        today: Option<NaiveDate>,
        #[props(default, into)]
        variant: Input<TimePickerVariant>,
        #[props(default)]
        with_seconds: Option<bool>,
        #[props(default)]
        step: Option<u8>,
        #[props(default)]
        twelve_hour: Option<bool>,
        /// Posts an ISO 8601 interval of two date-times.
        #[props(default, into)]
        name: FieldName<Option<DateRange<NaiveDateTime>>>,
        #[props(default, into)]
        placeholder: Option<String>,
    }
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
