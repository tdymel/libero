//! `TimePicker`, digital and analog, each echoing its value.

use dioxus::prelude::*;
use libero::{
    chrono::{NaiveDateTime, NaiveTime},
    components::{DatePicker, Flex, TimePicker},
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/time-picker", || rsx! { TimePickerPage {} }),
    ("/time-picker/date-time", || rsx! { DateTimePage {} }),
];

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

/// An empty date-time picker with no `today` prop: a time picked first takes
/// its day from the clock.
#[component]
fn DateTimePage() -> Element {
    let mut moment = use_signal(|| None::<NaiveDateTime>);
    let shown = moment().map(|value| value.to_string()).unwrap_or_default();

    rsx! {
        DatePicker::<NaiveDateTime> {
            variant: "digital",
            twelve_hour: false,
            value: moment(),
            onchange: move |next: Option<NaiveDateTime>| moment.set(next),
        }
        span { id: "moment-value", {shown} }
    }
}
