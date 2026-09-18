//! The pickers for one value type each - what `DatePicker` draws, under a
//! name and with only the props that type uses.

use dioxus::prelude::*;

use chrono::NaiveDate;

use super::{
    DateRange,
    calendar::DateLevel,
    date_value::{PickerArgs, PickerOptions, Sealed},
    props::date_props,
};
use crate::{components::common::Input, hooks::use_theme};

date_props! {
    picker DayPickerProps(NaiveDate, NaiveDate): limits, exclude_date, allow_deselect, columns, calendar, today
}

/// A month of days to pick one from, and what `DatePicker` draws for a
/// `NaiveDate`.
///
/// Controlled: it renders `value` and asks for a new one through `onchange`.
/// The month shown is its own state: it opens on `value`'s month, else
/// today's, and the arrows page it. The heading climbs to months and years.
///
/// A WAI-ARIA grid. One day is a tab stop; the arrow keys move a day or a
/// week, Home and End to the week's ends, Page Up and Page Down a month, with
/// Shift a year. Enter and Space pick.
#[component]
pub fn DayPicker(props: DayPickerProps) -> Element {
    let theme = use_theme();
    NaiveDate::picker(PickerArgs {
        value: props.value,
        onchange: props.onchange,
        options: PickerOptions {
            min: props.min,
            max: props.max,
            exclude_date: props.exclude_date,
            allow_deselect: props.allow_deselect.unwrap_or(false),
            columns: props.columns,
            calendar: props.calendar.copied_or(theme.date_picker.calendar),
            days: props.days.unwrap_or(theme.date_picker.days),
            ..PickerOptions::default()
        },
        today: props.today,
        size: props.size,
        focusable: props.focusable.unwrap_or(true),
        name: props.name,
        class: props.class,
        sx: props.sx,
        states: props.states,
        attributes: props.attributes,
    })
}

date_props! {
    picker DateRangePickerProps(DateRange<NaiveDate>, NaiveDate): limits, exclude_date, columns, today
}

/// Two months side by side to pick a start and an end from. The first pick starts a new range and the second
/// sets its end, swapped in when it comes first. While the end is missing, the
/// days up to the one under the mouse preview the range.
#[component]
pub fn DateRangePicker(props: DateRangePickerProps) -> Element {
    DateRange::<NaiveDate>::picker(PickerArgs {
        value: props.value,
        onchange: props.onchange,
        options: PickerOptions {
            min: props.min,
            max: props.max,
            exclude_date: props.exclude_date,
            columns: props.columns,
            ..PickerOptions::default()
        },
        today: props.today,
        size: props.size,
        focusable: props.focusable.unwrap_or(true),
        name: props.name,
        class: props.class,
        sx: props.sx,
        states: props.states,
        attributes: props.attributes,
    })
}

date_props! {
    picker MonthPickerProps(NaiveDate, NaiveDate): limits, today
}

/// The months of a year to pick one from, and `DatePicker` at `DateLevel::Month`. The heading climbs to a decade of years.
/// A month is held as its first day.
#[component]
pub fn MonthPicker(props: MonthPickerProps) -> Element {
    NaiveDate::picker(PickerArgs {
        value: props.value,
        onchange: props.onchange,
        options: PickerOptions {
            min: props.min,
            max: props.max,
            level: DateLevel::Month,
            ..PickerOptions::default()
        },
        today: props.today,
        size: props.size,
        focusable: props.focusable.unwrap_or(true),
        name: props.name,
        class: props.class,
        sx: props.sx,
        states: props.states,
        attributes: props.attributes,
    })
}

date_props! {
    picker YearPickerProps(NaiveDate, NaiveDate): limits, today
}

/// A decade of years to pick one from, and `DatePicker` at `DateLevel::Year`. A year is held as its January 1.
#[component]
pub fn YearPicker(props: YearPickerProps) -> Element {
    NaiveDate::picker(PickerArgs {
        value: props.value,
        onchange: props.onchange,
        options: PickerOptions {
            min: props.min,
            max: props.max,
            level: DateLevel::Year,
            ..PickerOptions::default()
        },
        today: props.today,
        size: props.size,
        focusable: props.focusable.unwrap_or(true),
        name: props.name,
        class: props.class,
        sx: props.sx,
        states: props.states,
        attributes: props.attributes,
    })
}
