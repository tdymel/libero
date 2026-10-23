//! Disabled date and time pickers (todo 3881). `#flip` toggles them all, by their own
//! `disabled` or, on `/date/disabled-fieldset`, by a `Fieldset` around them; `#changes` counts picks.
//! `/date/extras`: presets, week numbers, `open_to` and per-unit time steps (3988, 4003).

use dioxus::prelude::*;
use libero::chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeDelta};
use libero::components::{
    ChronoPicker, DateField, DateLevel, DatePicker, DatePreset, DateRange, DateRangePicker,
    Fieldset, Flex, MonthPicker, TextField, TimePicker, YearPicker,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/date/disabled", || rsx! { DisabledPickersPage { in_fieldset: false } }),
    ("/date/disabled-fieldset", || rsx! { DisabledPickersPage { in_fieldset: true } }),
    ("/date/fieldset-submit", || rsx! { FieldsetSubmitPage {} }),
    ("/date/extras", || rsx! { ExtrasPage {} }),
];

const DAY: Option<NaiveDate> = NaiveDate::from_ymd_opt(2026, 10, 21);
const TIME: Option<NaiveTime> = NaiveTime::from_hms_opt(9, 30, 0);

fn moment() -> Option<NaiveDateTime> {
    DAY.zip(TIME).map(|(day, time)| day.and_time(time))
}

/// Every picker, each in a `div` with an id, sharing one change counter.
#[component]
fn Pickers(disabled: bool, changes: Signal<u32>, named: bool) -> Element {
    let mut changes = changes;
    let name = |name: &str| named.then(|| name.to_string());
    let range = DAY.map(|day| DateRange::new(day, DAY));
    let moments = moment().map(|at| DateRange::new(at, Some(at)));
    rsx! {
        div { id: "date",
            DatePicker { value: DAY, today: DAY, name: name("date"), disabled, onchange: move |_| changes += 1 }
        }
        div { id: "range",
            DateRangePicker { value: range, today: DAY, columns: 1usize, name: name("range"), disabled, onchange: move |_| changes += 1 }
        }
        div { id: "month",
            MonthPicker { value: DAY, today: DAY, name: name("month"), disabled, onchange: move |_| changes += 1 }
        }
        div { id: "year",
            YearPicker { value: DAY, today: DAY, name: name("year"), disabled, onchange: move |_| changes += 1 }
        }
        div { id: "time",
            TimePicker { value: TIME, variant: "digital", name: name("time"), disabled, onchange: move |_| changes += 1 }
        }
        div { id: "analog",
            TimePicker { value: TIME, variant: "analog", name: name("analog"), disabled, onchange: move |_| changes += 1 }
        }
        div { id: "duration",
            ChronoPicker::<TimeDelta> {
                value: Some(TimeDelta::minutes(90)),
                name: name("duration"),
                disabled,
                onchange: move |_: Option<TimeDelta>| changes += 1,
            }
        }
        div { id: "datetime",
            ChronoPicker::<NaiveDateTime> {
                value: moment(),
                today: DAY,
                name: name("datetime"),
                disabled,
                onchange: move |_: Option<NaiveDateTime>| changes += 1,
            }
        }
        div { id: "moments",
            ChronoPicker::<DateRange<NaiveDateTime>> {
                value: moments,
                today: DAY,
                name: name("moments"),
                disabled,
                onchange: move |_: Option<DateRange<NaiveDateTime>>| changes += 1,
            }
        }
    }
}

#[component]
fn DisabledPickersPage(in_fieldset: bool) -> Element {
    let mut off = use_signal(|| false);
    let changes = use_signal(|| 0);
    let own = off() && !in_fieldset;
    rsx! {
        Flex { direction: "column", gap: "md", align: "start",
            button { id: "flip", onclick: move |_| off.toggle(), "Flip" }
            span { id: "changes", "data-changes": "{changes}" }
            if in_fieldset {
                Fieldset::<()> { label: "When", disabled: off(),
                    Pickers { disabled: false, changes, named: false }
                }
            } else {
                Pickers { disabled: own, changes, named: false }
            }
        }
    }
}

/// A disabled `Fieldset` posts no picker value; `outside` proves the submit is read.
#[component]
fn FieldsetSubmitPage() -> Element {
    let changes = use_signal(|| 0);
    rsx! {
        form { id: "sent",
            Fieldset::<()> { label: "Locked", disabled: true,
                Pickers { disabled: false, changes, named: true }
            }
            TextField { label: "Outside", name: "outside", value: "x" }
            button { id: "submit", r#type: "submit", "Send" }
        }
    }
}

/// Each picker writes its value into the `data-value` of the readout after it.
#[component]
fn ExtrasPage() -> Element {
    let mut presets = use_signal(|| DAY);
    let mut field = use_signal(|| None::<NaiveDate>);
    let mut year_first = use_signal(|| None::<NaiveDate>);
    let mut time = use_signal(|| NaiveTime::from_hms_opt(14, 0, 30));
    let day = |month, day| NaiveDate::from_ymd_opt(2026, month, day).expect("a day");
    let offered = vec![
        DatePreset::new("Start of month", day(10, 1)),
        DatePreset::new("Next month", day(11, 1)),
    ];
    let shown = |value: Option<String>| value.unwrap_or_default();
    rsx! {
        Flex { direction: "column", gap: "md", align: "start",
            div { id: "presets",
                DatePicker {
                    value: presets(),
                    today: DAY,
                    presets: offered.clone(),
                    week_numbers: true,
                    onchange: move |next| presets.set(next),
                }
                span { id: "presets-value", "data-value": shown(presets().map(|v| v.to_string())) }
            }
            div { id: "field",
                DateField {
                    label: "Due",
                    value: field(),
                    today: DAY,
                    presets: offered,
                    onchange: move |next| field.set(next),
                }
                span { id: "field-value", "data-value": shown(field().map(|v| v.to_string())) }
            }
            div { id: "year-first",
                DatePicker {
                    value: year_first(),
                    today: DAY,
                    open_to: DateLevel::Year,
                    onchange: move |next| year_first.set(next),
                }
                span { id: "year-first-value", "data-value": shown(year_first().map(|v| v.to_string())) }
            }
            div { id: "steps",
                TimePicker {
                    value: time(),
                    variant: "digital",
                    twelve_hour: false,
                    with_seconds: true,
                    hour_step: 2,
                    second_step: 15,
                    onchange: move |next| time.set(next),
                }
                span { id: "steps-value", "data-value": shown(time().map(|v| v.to_string())) }
            }
        }
    }
}
