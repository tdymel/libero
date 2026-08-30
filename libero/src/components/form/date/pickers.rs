use dioxus::prelude::*;

use chrono::{Datelike, NaiveDate};

use super::{
    DateRange,
    calendar::{Calendar, Level, Selection, first_of_month},
};
use crate::{
    components::{Input, common::base_props},
    hooks::use_theme,
    theme::Size,
};

base_props! {
    pub struct DatePickerProps {
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

/// A month of days to pick one from - Mantine's `DatePicker`.
///
/// Controlled: it renders `value` and asks for a new one through `onchange`.
/// The month shown is its own state: it opens on `value`'s month, else
/// today's, and the arrows page it. The heading climbs to months and years.
///
/// A WAI-ARIA grid. One day is a tab stop; the arrow keys move a day or a
/// week, Home and End to the week's ends, Page Up and Page Down a month, with
/// Shift a year. Enter and Space pick.
#[component]
pub fn DatePicker(props: DatePickerProps) -> Element {
    let theme = use_theme();
    let value = props.value;
    let onchange = props.onchange;
    let allow_deselect = props.allow_deselect.unwrap_or(false);
    rsx! {
        Calendar {
            selection: Selection::Single(value),
            onpick: move |day: NaiveDate| {
                let next = match allow_deselect && value == Some(day) {
                    true => None,
                    false => Some(day),
                };
                if let Some(onchange) = &onchange {
                    onchange.call(next);
                }
            },
            columns: props.columns.unwrap_or(1),
            lowest: Level::Day,
            min: props.min,
            max: props.max,
            exclude_date: props.exclude_date,
            today: props.today,
            size: props.size.copied_or(theme.date_picker.size),
            focusable: props.focusable.unwrap_or(true),
            hidden: props.name.map(|name| (name, value.map(|day| day.to_string()).unwrap_or_default())),
            class: props.class,
            sx: props.sx,
            states: props.states,
            attributes: props.attributes,
        }
    }
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
    let theme = use_theme();
    let value = props.value;
    let onchange = props.onchange;
    rsx! {
        Calendar {
            selection: Selection::Range(value),
            onpick: move |day: NaiveDate| {
                if let Some(onchange) = &onchange {
                    onchange.call(Some(DateRange::pick(value, day)));
                }
            },
            columns: props.columns.unwrap_or(2),
            lowest: Level::Day,
            min: props.min,
            max: props.max,
            exclude_date: props.exclude_date,
            today: props.today,
            size: props.size.copied_or(theme.date_picker.size),
            focusable: props.focusable.unwrap_or(true),
            hidden: props.name.map(|name| (name, value.map(|range| range.to_string()).unwrap_or_default())),
            class: props.class,
            sx: props.sx,
            states: props.states,
            attributes: props.attributes,
        }
    }
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

/// The months of a year to pick one from - Mantine's `MonthPicker`. The
/// heading climbs to a decade of years.
#[component]
pub fn MonthPicker(props: MonthPickerProps) -> Element {
    let theme = use_theme();
    let value = props.value.map(first_of_month);
    let onchange = props.onchange;
    rsx! {
        Calendar {
            selection: Selection::Single(value),
            onpick: move |month: NaiveDate| {
                if let Some(onchange) = &onchange {
                    onchange.call(Some(month));
                }
            },
            lowest: Level::Month,
            min: props.min,
            max: props.max,
            today: props.today,
            size: props.size.copied_or(theme.date_picker.size),
            focusable: props.focusable.unwrap_or(true),
            hidden: props.name.map(|name| (name, value.map(|day| day.to_string()).unwrap_or_default())),
            class: props.class,
            sx: props.sx,
            states: props.states,
            attributes: props.attributes,
        }
    }
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

/// A decade of years to pick one from - Mantine's `YearPicker`.
#[component]
pub fn YearPicker(props: YearPickerProps) -> Element {
    let theme = use_theme();
    let value = props
        .value
        .and_then(|day| NaiveDate::from_ymd_opt(day.year(), 1, 1));
    let onchange = props.onchange;
    rsx! {
        Calendar {
            selection: Selection::Single(value),
            onpick: move |year: NaiveDate| {
                if let Some(onchange) = &onchange {
                    onchange.call(Some(year));
                }
            },
            lowest: Level::Year,
            min: props.min,
            max: props.max,
            today: props.today,
            size: props.size.copied_or(theme.date_picker.size),
            focusable: props.focusable.unwrap_or(true),
            hidden: props.name.map(|name| (name, value.map(|day| day.to_string()).unwrap_or_default())),
            class: props.class,
            sx: props.sx,
            states: props.states,
            attributes: props.attributes,
        }
    }
}
