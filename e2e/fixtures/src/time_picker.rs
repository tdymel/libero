//! `TimePicker`, digital and analog, each echoing its value.

use dioxus::prelude::*;
use libero::{
    chrono::{NaiveDate, NaiveDateTime, NaiveTime},
    components::{ChronoPicker, DateRange, Flex, TimePicker},
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/time-picker", || rsx! { TimePickerPage {} }),
    ("/time-picker/date-time", || rsx! { DateTimePage {} }),
    ("/time-picker/date-time-pinned", || rsx! { DateTimePage { today: NaiveDate::from_ymd_opt(2026, 3, 18) } }),
    ("/time-picker/analog", || rsx! { AnalogPage {} }),
    ("/time-picker/range", || rsx! { RangePage {} }),
];

/// An empty date-time range picker on the digital clock, `today` pinned to
/// 2026-03-18, echoing its value.
#[component]
fn RangePage() -> Element {
    let mut range = use_signal(|| None::<DateRange<NaiveDateTime>>);
    let shown = range().map(|value| value.to_string()).unwrap_or_default();

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "360px",
            ChronoPicker::<DateRange<NaiveDateTime>> {
                variant: "digital",
                twelve_hour: false,
                today: NaiveDate::from_ymd_opt(2026, 3, 18),
                value: range(),
                onchange: move |next: Option<DateRange<NaiveDateTime>>| range.set(next),
            }
            span { id: "range-value", {shown} }
        }
    }
}

/// An analog face at a one-minute step, with seconds, from 10:00 with `min`
/// 09:30, and a digital picker with the same `min` (its early hours disabled).
#[component]
fn AnalogPage() -> Element {
    let mut fine = use_signal(|| NaiveTime::from_hms_opt(10, 0, 0));
    let mut limited = use_signal(|| NaiveTime::from_hms_opt(10, 0, 0));
    let shown = fine().map(|value| value.to_string()).unwrap_or_default();

    rsx! {
        TimePicker {
            id: "fine",
            variant: "analog",
            twelve_hour: false,
            step: 1,
            with_seconds: true,
            min: NaiveTime::from_hms_opt(9, 30, 0),
            value: fine(),
            onchange: move |next: Option<NaiveTime>| fine.set(next),
        }
        span { id: "fine-value", {shown} }
        TimePicker {
            id: "limited",
            variant: "digital",
            twelve_hour: false,
            min: NaiveTime::from_hms_opt(9, 30, 0),
            value: limited(),
            onchange: move |next: Option<NaiveTime>| limited.set(next),
        }
    }
}

/// 24 hours and a 5-minute step pinned, so the options never depend on the
/// theme's locale.
#[component]
fn TimePickerPage() -> Element {
    let mut digital = use_signal(|| NaiveTime::from_hms_opt(9, 30, 0));
    let mut analog = use_signal(|| NaiveTime::from_hms_opt(9, 30, 0));
    let shown = |value: Option<NaiveTime>| value.map(|value| value.to_string()).unwrap_or_default();

    rsx! {
        Flex { direction: "column", gap: "md",
            TimePicker {
                id: "digital",
                variant: "digital",
                twelve_hour: false,
                step: 5,
                value: digital(),
                onchange: move |next: Option<NaiveTime>| digital.set(next),
            }
            span { id: "digital-value", {shown(digital())} }
            TimePicker {
                id: "analog",
                variant: "analog",
                twelve_hour: false,
                step: 5,
                value: analog(),
                onchange: move |next: Option<NaiveTime>| analog.set(next),
            }
            span { id: "analog-value", {shown(analog())} }
        }
    }
}

/// An empty date-time picker. Without `today` a time picked first takes its day
/// from the clock; the baseline pins it, so the snapshot keeps its date.
#[component]
fn DateTimePage(today: Option<NaiveDate>) -> Element {
    let mut moment = use_signal(|| None::<NaiveDateTime>);
    let shown = moment().map(|value| value.to_string()).unwrap_or_default();

    rsx! {
        ChronoPicker::<NaiveDateTime> {
            variant: "digital",
            twelve_hour: false,
            today,
            value: moment(),
            onchange: move |next: Option<NaiveDateTime>| moment.set(next),
        }
        span { id: "moment-value", {shown} }
    }
}
