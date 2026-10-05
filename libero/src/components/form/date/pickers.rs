//! The pickers for one value type each: `ChronoPicker` with only the props that type uses.

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
    picker DatePickerProps(NaiveDate, NaiveDate): limits, exclude_date, allow_deselect, columns, calendar, today
}

/// A month of days to pick one from; `ChronoPicker` for a `NaiveDate`.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::chrono::NaiveDate;
/// # use libero::components::DatePicker;
/// # fn app() -> Element {
/// let mut date = use_signal(|| None::<NaiveDate>);
/// rsx! { DatePicker { value: date(), onchange: move |v| date.set(v) } }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/chrono-picker>
#[component]
pub fn DatePicker(props: DatePickerProps) -> Element {
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
            calendar: props.calendar.copied_or(theme.chrono_picker.calendar),
            days: props.days.unwrap_or(theme.chrono_picker.days),
            ..PickerOptions::default()
        },
        today: props.today,
        size: props.size,
        focusable: props.focusable.unwrap_or(true),
        name: props.name,
        class: props.class,
        sx: props.sx,
        parts: props.parts,
        states: props.states,
        attributes: props.attributes,
    })
}

date_props! {
    picker DateRangePickerProps(DateRange<NaiveDate>, NaiveDate): limits, exclude_date, columns, today
}

/// Two months side by side to pick a start and an end from; one below `sm` unless `columns` is set.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::chrono::NaiveDate;
/// # use libero::components::{DateRange, DateRangePicker};
/// # fn app() -> Element {
/// let mut range = use_signal(|| None::<DateRange<NaiveDate>>);
/// rsx! { DateRangePicker { value: range(), onchange: move |v| range.set(v) } }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/chrono-picker>
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
        parts: props.parts,
        states: props.states,
        attributes: props.attributes,
    })
}

date_props! {
    picker MonthPickerProps(NaiveDate, NaiveDate): limits, today
}

/// The months of a year to pick one from, held as the month's first day.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::chrono::NaiveDate;
/// # use libero::components::MonthPicker;
/// # fn app() -> Element {
/// let mut month = use_signal(|| None::<NaiveDate>);
/// rsx! { MonthPicker { value: month(), onchange: move |v| month.set(v) } }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/chrono-picker>
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
        parts: props.parts,
        states: props.states,
        attributes: props.attributes,
    })
}

date_props! {
    picker YearPickerProps(NaiveDate, NaiveDate): limits, today
}

/// A decade of years to pick one from, held as the year's January 1.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::chrono::NaiveDate;
/// # use libero::components::YearPicker;
/// # fn app() -> Element {
/// let mut year = use_signal(|| None::<NaiveDate>);
/// rsx! { YearPicker { value: year(), onchange: move |v| year.set(v) } }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/chrono-picker>
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
        parts: props.parts,
        states: props.states,
        attributes: props.attributes,
    })
}
