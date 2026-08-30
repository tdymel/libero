//! The pickers for one value type each - what `DatePicker` draws, under a
//! name and with only the props that type uses.

use dioxus::prelude::*;

use chrono::NaiveDate;

use super::{
    DateRange,
    calendar::DateLevel,
    date_value::{DateValue, PickerArgs, PickerOptions},
};
use crate::{
    components::{Input, common::base_props},
    theme::Size,
};

base_props! {
    pub struct DayPickerProps {
        /// The picked day; strictly controlled. `None` picks nothing.
        #[props(default)]
        value: Option<NaiveDate>,
        /// Called with the day the caller should hold next.
        #[props(default)]
        onchange: Option<EventHandler<Option<NaiveDate>>>,
        /// The earliest day that can be picked.
        #[props(default)]
        min: Option<NaiveDate>,
        /// The latest day that can be picked.
        #[props(default)]
        max: Option<NaiveDate>,
        /// Days that cannot be picked, on top of `min` and `max`. A
        /// `Callback` always compares equal, so changing only this closure
        /// does not redraw the picker.
        #[props(default)]
        exclude_date: Option<Callback<NaiveDate, bool>>,
        /// Clicking the picked day again clears it.
        #[props(default)]
        allow_deselect: Option<bool>,
        /// Months side by side.
        #[props(default)]
        columns: Option<usize>,
        /// The day marked as today. Unset, the platform clock answers after
        /// mount - on the web; elsewhere no day is marked.
        #[props(default)]
        today: Option<NaiveDate>,
        #[props(default, into)]
        size: Input<Size>,
        /// Emits a hidden input of that name, posting the day as ISO 8601.
        #[props(default, into)]
        name: Option<String>,
        /// `false` keeps the days and buttons out of the tab order - for a
        /// picker inside a dropdown whose text input must keep focus. On by
        /// default.
        #[props(default)]
        focusable: Option<bool>,
    }
}

/// A month of days to pick one from - Mantine's `DatePicker`, and what
/// `DatePicker` draws for a `NaiveDate`.
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
    NaiveDate::picker(PickerArgs {
        value: props.value,
        onchange: props.onchange,
        options: PickerOptions {
            min: props.min,
            max: props.max,
            exclude_date: props.exclude_date,
            allow_deselect: props.allow_deselect.unwrap_or(false),
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

base_props! {
    pub struct DateRangePickerProps {
        /// The picked range; strictly controlled. A range whose `end` is
        /// `None` is waiting for its second pick.
        #[props(default)]
        value: Option<DateRange<NaiveDate>>,
        /// Called with the range the caller should hold next: a new start on
        /// the first pick, the end on the second - swapped in when it comes
        /// first.
        #[props(default)]
        onchange: Option<EventHandler<Option<DateRange<NaiveDate>>>>,
        #[props(default)]
        min: Option<NaiveDate>,
        #[props(default)]
        max: Option<NaiveDate>,
        /// Days that cannot be picked, on top of `min` and `max`.
        #[props(default)]
        exclude_date: Option<Callback<NaiveDate, bool>>,
        /// Months side by side. Two by default.
        #[props(default)]
        columns: Option<usize>,
        #[props(default)]
        today: Option<NaiveDate>,
        #[props(default, into)]
        size: Input<Size>,
        /// Emits a hidden input of that name, posting the range as an ISO 8601
        /// interval: `2026-09-01/2026-09-05`.
        #[props(default, into)]
        name: Option<String>,
        #[props(default)]
        focusable: Option<bool>,
    }
}

/// Two months side by side to pick a start and an end from - Mantine's
/// `DatePicker type="range"`. While the end is missing, the days up to the one
/// under the mouse preview the range.
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

base_props! {
    pub struct MonthPickerProps {
        /// The picked month, as its first day; strictly controlled.
        #[props(default)]
        value: Option<NaiveDate>,
        /// Called with the first day of the picked month.
        #[props(default)]
        onchange: Option<EventHandler<Option<NaiveDate>>>,
        #[props(default)]
        min: Option<NaiveDate>,
        #[props(default)]
        max: Option<NaiveDate>,
        #[props(default)]
        today: Option<NaiveDate>,
        #[props(default, into)]
        size: Input<Size>,
        /// Emits a hidden input posting the month's first day as ISO 8601.
        #[props(default, into)]
        name: Option<String>,
        #[props(default)]
        focusable: Option<bool>,
    }
}

/// The months of a year to pick one from - Mantine's `MonthPicker`, and
/// `DatePicker` at `DateLevel::Month`. The heading climbs to a decade of years.
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

base_props! {
    pub struct YearPickerProps {
        /// The picked year, as its January 1; strictly controlled.
        #[props(default)]
        value: Option<NaiveDate>,
        /// Called with January 1 of the picked year.
        #[props(default)]
        onchange: Option<EventHandler<Option<NaiveDate>>>,
        #[props(default)]
        min: Option<NaiveDate>,
        #[props(default)]
        max: Option<NaiveDate>,
        #[props(default)]
        today: Option<NaiveDate>,
        #[props(default, into)]
        size: Input<Size>,
        /// Emits a hidden input posting the year's January 1 as ISO 8601.
        #[props(default, into)]
        name: Option<String>,
        #[props(default)]
        focusable: Option<bool>,
    }
}

/// A decade of years to pick one from - Mantine's `YearPicker`, and
/// `DatePicker` at `DateLevel::Year`.
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
