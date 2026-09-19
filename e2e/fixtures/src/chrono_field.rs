//! `ChronoField` at month level, at day level between two buttons, as a
//! date-time field, on the digital columns and as a duration.

use dioxus::prelude::*;
use libero::{
    chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, Weekday},
    components::{Button, ChronoField, DateLevel, Flex, Input, Text},
    theme::TimePickerVariant,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/chrono-field/month", || rsx! { MonthFieldPage {} }),
    ("/chrono-field/day", || rsx! { DateFieldPage {} }),
    ("/chrono-field/moment", || rsx! { MomentFieldPage {} }),
    ("/chrono-field/digital", || rsx! { DigitalFieldPage {} }),
    ("/chrono-field/duration", || rsx! { DurationFieldPage {} }),
];

/// A duration of 1 h 30 min held, 15 min to 12 h, minutes at 15 and seconds
/// shown. The readout shows it in seconds.
#[component]
fn DurationFieldPage() -> Element {
    let mut length = use_signal(|| TimeDelta::try_minutes(90));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            ChronoField::<TimeDelta> {
                label: "Length",
                min: TimeDelta::try_minutes(15),
                max: TimeDelta::try_hours(12),
                step: 15,
                with_seconds: true,
                value: length(),
                onchange: move |next| length.set(next),
                name: "length",
            }
            Text { id: "readout", {length().map(|length| length.num_seconds().to_string()).unwrap_or_default()} }
        }
    }
}

/// A time field on the digital columns, 09:30 held.
#[component]
fn DigitalFieldPage() -> Element {
    let mut time = use_signal(|| NaiveTime::from_hms_opt(9, 30, 0));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            ChronoField::<NaiveTime> {
                label: "Alarm",
                variant: Input::Value(TimePickerVariant::Digital),
                value: time(),
                onchange: move |next| time.set(next),
            }
            Text { id: "readout", {time().map(|time| time.to_string()).unwrap_or_default()} }
        }
    }
}

/// March 2026 held and `today` pinned, so the grid never depends on the clock.
/// The readout shows the held value in ISO 8601.
#[component]
fn MonthFieldPage() -> Element {
    let mut month = use_signal(|| NaiveDate::from_ymd_opt(2026, 3, 1));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            ChronoField::<NaiveDate> {
                label: "Billing month",
                level: DateLevel::Month,
                today: NaiveDate::from_ymd_opt(2026, 3, 18),
                value: month(),
                onchange: move |next| month.set(next),
            }
            Text { id: "readout", {month().map(|day| day.to_string()).unwrap_or_default()} }
        }
    }
}

/// 2026-03-18 held, `today` pinned to it, days before March 5 and weekends
/// disabled. A button on either side, for Tab order out of the dropdown.
#[component]
fn DateFieldPage() -> Element {
    let mut day = use_signal(|| NaiveDate::from_ymd_opt(2026, 3, 18));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            ChronoField::<NaiveDate> {
                label: "Due date",
                today: NaiveDate::from_ymd_opt(2026, 3, 18),
                min: NaiveDate::from_ymd_opt(2026, 3, 5),
                exclude_date: |day: NaiveDate| matches!(day.weekday(), Weekday::Sat | Weekday::Sun),
                value: day(),
                onchange: move |next| day.set(next),
            }
            Button { id: "after", "After" }
            Text { id: "readout", {day().map(|day| day.to_string()).unwrap_or_default()} }
        }
    }
}

/// 2026-03-18 09:30 held, `today` pinned.
#[component]
fn MomentFieldPage() -> Element {
    let mut moment = use_signal(|| {
        NaiveDate::from_ymd_opt(2026, 3, 18).and_then(|day| day.and_hms_opt(9, 30, 0))
    });

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            ChronoField::<NaiveDateTime> {
                label: "Starts at",
                today: NaiveDate::from_ymd_opt(2026, 3, 18),
                value: moment(),
                onchange: move |next| moment.set(next),
            }
            Button { id: "after", "After" }
            Text { id: "readout", {moment().map(|moment| moment.to_string()).unwrap_or_default()} }
        }
    }
}
