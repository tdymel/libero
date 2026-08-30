use super::common::{SIZES, field_controls, is_on, shown, status_of, text_of};
use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use libero::components::{Code, DateFieldPrototype, DateRange, Flex, Text};

const KINDS: [&str; 5] = ["date", "time", "date-time", "date-range", "date-time-range"];

fn day(day: u32) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(2026, 9, day)
}

fn moment(day: u32, hour: u32) -> Option<NaiveDateTime> {
    NaiveDate::from_ymd_opt(2026, 9, day)
        .zip(NaiveTime::from_hms_opt(hour, 0, 0))
        .map(|(day, time)| NaiveDateTime::new(day, time))
}

#[component]
pub fn DateFieldPrototypePage() -> Element {
    rsx! {
        DocPage {
            title: "DateFieldPrototype",
            source: "libero/src/components/form/date/prototype.rs",
            markdown: "/md/date_field_prototype.md",
            properties: vec![
                props("DateFieldPrototype<V: DateValue>", vec![
                    prop("value", "Option<V>").doc("The value; its type picks the dropdown."),
                    prop("onchange", "EventHandler<Option<V>>").doc("On commit - blur or Enter - and on every pick."),
                    prop("min", "V::Bound").doc("The earliest value. For a range, the earliest end: a `NaiveDate` or `NaiveDateTime`."),
                    prop("max", "V::Bound").doc("The latest value, likewise."),
                    prop("format", "String").default("DateDefaults::format").doc("How the text shows a day."),
                    prop("time_format", "String").default("DateDefaults::time_format").doc("How the text shows a time."),
                    prop("exclude_date", "Callback<NaiveDate, bool>").doc("Days that cannot be picked or typed. Ignored for a time."),
                    prop("variant", "TimePickerVariant").default("digital").doc("The clock, for values with a time."),
                    prop("with_seconds", "bool").default("false").doc("Seconds, for values with a time."),
                    prop("step", "u8").default("1").doc("Minutes between the offered minutes."),
                    prop("twelve_hour", "bool").doc("A 12-hour clock."),
                    prop("columns", "usize").default("2").doc("Months side by side, for a range of days."),
                    prop("close_on_change", "bool").default("true").doc("Picking a day, or a range's end, closes the dropdown."),
                    prop("today", "NaiveDate").doc("The day marked as today."),
                    prop("name", "FieldName<Option<V>>").doc("Posts the value as ISO 8601."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A prototype: one field for every date and time value, where the value's type picks the dropdown. "
                    "It stands in for the five fields - "
                    Code { source: "DateField" }
                    ", "
                    Code { source: "TimeField" }
                    ", "
                    Code { source: "DateTimeField" }
                    ", "
                    Code { source: "DateRangeField" }
                    " and "
                    Code { source: "DateTimeRangeField" }
                    " - over the same engine."
                }
                Text {
                    "A value of "
                    Code { source: "Option<NaiveDate>" }
                    " opens a calendar, "
                    Code { source: "Option<NaiveTime>" }
                    " a clock, "
                    Code { source: "Option<NaiveDateTime>" }
                    " both, and a "
                    Code { source: "DateRange" }
                    " of either picks two. "
                    Code { source: "min" }
                    " and "
                    Code { source: "max" }
                    " take "
                    Code { source: "V::Bound" }
                    ": the value's own type, or a range's end type."
                }
            },
            Demo {
                component: "DateFieldPrototype",
                children_text: "",
                controls: {
                    let mut controls = vec![
                        Control::select("value", KINDS).default("date").code(|_, values| {
                            let value = match values.str("value").as_str() {
                                "time" => "time() /* Option<NaiveTime> */",
                                "date-time" => "moment() /* Option<NaiveDateTime> */",
                                "date-range" => "range() /* Option<DateRange<NaiveDate>> */",
                                "date-time-range" => "range() /* Option<DateRange<NaiveDateTime>> */",
                                _ => "day() /* Option<NaiveDate> */",
                            };
                            vec![format!("value: {value}")]
                        }),
                        Control::slider("size", SIZES).default("md"),
                    ];
                    controls.extend(field_controls());
                    controls
                },
                render: move |values: DemoValues| rsx! {
                    PrototypeDemo { values }
                },
            }
        }
    }
}

/// One signal per value type, so switching types keeps what was picked.
#[component]
fn PrototypeDemo(values: DemoValues) -> Element {
    let mut date = use_signal(|| day(14));
    let mut time = use_signal(|| NaiveTime::from_hms_opt(9, 30, 0));
    let mut date_time = use_signal(|| moment(14, 9));
    let mut date_range = use_signal(|| day(14).map(|start| DateRange::new(start, day(18))));
    let mut date_time_range =
        use_signal(|| moment(14, 9).map(|start| DateRange::new(start, moment(16, 17))));

    let size = values.str("size");
    let label = text_of(&values, "label", "When");
    let description = text_of(&values, "description", "Your local time.");
    let helper = text_of(&values, "helper", "Typing works too.");
    let placeholder = text_of(&values, "placeholder", "Pick one");
    let status = status_of(&values);
    let required = is_on(&values, "required").then_some(true);
    let disabled = is_on(&values, "disabled").then_some(true);

    let (field, readout) = match values.str("value").as_str() {
        "time" => (
            rsx! {
                DateFieldPrototype {
                    value: time(), onchange: move |next| time.set(next),
                    size, label, description, helper, placeholder, status, required, disabled,
                }
            },
            shown(time()),
        ),
        "date-time" => (
            rsx! {
                DateFieldPrototype {
                    value: date_time(), onchange: move |next| date_time.set(next),
                    size, label, description, helper, placeholder, status, required, disabled,
                }
            },
            shown(date_time()),
        ),
        "date-range" => (
            rsx! {
                DateFieldPrototype {
                    value: date_range(), onchange: move |next| date_range.set(next),
                    size, label, description, helper, placeholder, status, required, disabled,
                }
            },
            shown(date_range()),
        ),
        "date-time-range" => (
            rsx! {
                DateFieldPrototype {
                    value: date_time_range(), onchange: move |next| date_time_range.set(next),
                    size, label, description, helper, placeholder, status, required, disabled,
                }
            },
            shown(date_time_range()),
        ),
        _ => (
            rsx! {
                DateFieldPrototype {
                    value: date(), onchange: move |next| date.set(next),
                    size, label, description, helper, placeholder, status, required, disabled,
                }
            },
            shown(date()),
        ),
    };
    rsx! {
        Flex { direction: "column", gap: "sm", sx: libero::sx::sx().width("100%").max_width("440px"),
            {field}
            Text { size: "sm", {readout} }
        }
    }
}
