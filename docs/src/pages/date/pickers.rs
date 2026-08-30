use super::common::{SIZES, is_on, is_weekend, shown};
use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::chrono::{NaiveDate, NaiveTime};
use libero::components::{
    Code, DateRange, DateRangePicker, Flex, MonthPicker, Text, TimePicker, YearPicker,
};

#[component]
pub fn DateRangePickerPage() -> Element {
    rsx! {
        DocPage {
            title: "DateRangePicker",
            source: "libero/src/components/form/date/pickers.rs",
            markdown: "/md/date_range_picker.md",
            properties: vec![
                props("DateRangePicker", vec![
                    prop("value", "Option<DateRange<NaiveDate>>").doc("The picked range; strictly controlled. An `end` of `None` waits for the second pick."),
                    prop("onchange", "EventHandler<Option<DateRange<NaiveDate>>>").doc("A new start on the first pick, the end on the second - swapped in when it comes first."),
                    prop("columns", "usize").default("2").doc("Months side by side."),
                    prop("min", "NaiveDate").doc("The earliest day that can be picked."),
                    prop("max", "NaiveDate").doc("The latest day that can be picked."),
                    prop("exclude_date", "Callback<NaiveDate, bool>").doc("Days that cannot be picked."),
                    prop("today", "NaiveDate").doc("The day marked as today."),
                    prop("size", "Size").default("md").doc("Day cell size and font size."),
                    prop("name", "String").doc("Posts an ISO 8601 interval, `2026-09-01/2026-09-05`."),
                    prop("focusable", "bool").default("true").doc("`false` keeps days and buttons out of the tab order."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "Months side by side to pick a start and an end from. The value is a "
                    Code { source: "DateRange<NaiveDate>" }
                    " whose "
                    Code { source: "end" }
                    " is "
                    Code { source: "None" }
                    " while it waits for the second pick - the days up to the one under the mouse preview it."
                }
            },
            Demo {
                component: "DateRangePicker",
                children_text: "",
                controls: vec![
                    Control::slider("size", SIZES).default("md"),
                    Control::slider("columns", ["1", "2", "3"]).default("2").code(|_, values| {
                        match values.str("columns").as_str() {
                            "2" => vec![],
                            columns => vec![format!("columns: {columns}usize")],
                        }
                    }),
                    Control::switch("exclude_weekends").code(|_, values| {
                        match is_on(values, "exclude_weekends") {
                            true => vec!["exclude_date: |day: NaiveDate| day.weekday().num_days_from_monday() >= 5".to_string()],
                            false => vec![],
                        }
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    DateRangePickerDemo { values }
                },
            }
        }
    }
}

#[component]
fn DateRangePickerDemo(values: DemoValues) -> Element {
    let mut range = use_signal(|| {
        NaiveDate::from_ymd_opt(2026, 9, 14)
            .map(|start| DateRange::new(start, NaiveDate::from_ymd_opt(2026, 9, 18)))
    });
    rsx! {
        Flex { direction: "column", gap: "sm", align: "flex-start",
            DateRangePicker {
                value: range(),
                onchange: move |next| range.set(next),
                size: values.str("size"),
                columns: values.str("columns").parse::<usize>().ok(),
                exclude_date: is_on(&values, "exclude_weekends").then(|| Callback::new(is_weekend)),
            }
            Text { size: "sm", {shown(range())} }
        }
    }
}

#[component]
pub fn MonthPickerPage() -> Element {
    rsx! {
        DocPage {
            title: "MonthPicker",
            source: "libero/src/components/form/date/pickers.rs",
            markdown: "/md/month_picker.md",
            properties: vec![
                props("MonthPicker", vec![
                    prop("value", "Option<NaiveDate>").doc("The picked month, as its first day; strictly controlled."),
                    prop("onchange", "EventHandler<Option<NaiveDate>>").doc("Called with the first day of the picked month."),
                    prop("min", "NaiveDate").doc("Months ending before it are disabled."),
                    prop("max", "NaiveDate").doc("Months starting after it are disabled."),
                    prop("today", "NaiveDate").doc("Marks today's month. Unset, the platform clock answers after mount."),
                    prop("size", "Size").default("md").doc("Cell size and font size."),
                    prop("name", "String").doc("Posts the month's first day as ISO 8601."),
                    prop("focusable", "bool").default("true").doc("`false` keeps the cells and buttons out of the tab order."),
                ]),
            ],
            lead: rsx! {
                Text { "The months of a year to pick one from - the view a DatePicker's heading climbs to. The value is the month's first day. The heading climbs on to a decade of years, and a year opens its months again." }
            },
            Demo {
                component: "MonthPicker",
                children_text: "",
                controls: vec![
                    Control::slider("size", SIZES).default("md"),
                    Control::switch("limits").code(|_, values| match is_on(values, "limits") {
                        true => vec![
                            "min: NaiveDate::from_ymd_opt(2026, 3, 15)".to_string(),
                            "max: NaiveDate::from_ymd_opt(2026, 10, 1)".to_string(),
                        ],
                        false => vec![],
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    MonthPickerDemo { values }
                },
            }
        }
    }
}

#[component]
fn MonthPickerDemo(values: DemoValues) -> Element {
    let mut value = use_signal(|| NaiveDate::from_ymd_opt(2026, 9, 1));
    let limits = is_on(&values, "limits");
    rsx! {
        Flex { direction: "column", gap: "sm", align: "flex-start",
            MonthPicker {
                value: value(),
                onchange: move |next| value.set(next),
                size: values.str("size"),
                min: limits.then(|| NaiveDate::from_ymd_opt(2026, 3, 15)).flatten(),
                max: limits.then(|| NaiveDate::from_ymd_opt(2026, 10, 1)).flatten(),
            }
            Text { size: "sm", {shown(value())} }
        }
    }
}

#[component]
pub fn YearPickerPage() -> Element {
    rsx! {
        DocPage {
            title: "YearPicker",
            source: "libero/src/components/form/date/pickers.rs",
            markdown: "/md/year_picker.md",
            properties: vec![
                props("YearPicker", vec![
                    prop("value", "Option<NaiveDate>").doc("The picked year, as its January 1; strictly controlled."),
                    prop("onchange", "EventHandler<Option<NaiveDate>>").doc("Called with January 1 of the picked year."),
                    prop("min", "NaiveDate").doc("Years before its year are disabled."),
                    prop("max", "NaiveDate").doc("Years after its year are disabled."),
                    prop("today", "NaiveDate").doc("Marks today's year. Unset, the platform clock answers after mount."),
                    prop("size", "Size").default("md").doc("Cell size and font size."),
                    prop("name", "String").doc("Posts January 1 as ISO 8601."),
                    prop("focusable", "bool").default("true").doc("`false` keeps the cells and buttons out of the tab order."),
                ]),
            ],
            lead: rsx! {
                Text { "The years of a decade to pick one from, with a year of the decades before and after at the edges. The arrows page a decade. The value is the year's January 1." }
            },
            Demo {
                component: "YearPicker",
                children_text: "",
                controls: vec![
                    Control::slider("size", SIZES).default("md"),
                    Control::switch("limits").code(|_, values| match is_on(values, "limits") {
                        true => vec![
                            "min: NaiveDate::from_ymd_opt(2022, 1, 1)".to_string(),
                            "max: NaiveDate::from_ymd_opt(2034, 12, 31)".to_string(),
                        ],
                        false => vec![],
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    YearPickerDemo { values }
                },
            }
        }
    }
}

#[component]
fn YearPickerDemo(values: DemoValues) -> Element {
    let mut value = use_signal(|| NaiveDate::from_ymd_opt(2026, 1, 1));
    let limits = is_on(&values, "limits");
    rsx! {
        Flex { direction: "column", gap: "sm", align: "flex-start",
            YearPicker {
                value: value(),
                onchange: move |next| value.set(next),
                size: values.str("size"),
                min: limits.then(|| NaiveDate::from_ymd_opt(2022, 1, 1)).flatten(),
                max: limits.then(|| NaiveDate::from_ymd_opt(2034, 12, 31)).flatten(),
            }
            Text { size: "sm", {shown(value())} }
        }
    }
}

#[component]
pub fn TimePickerPage() -> Element {
    rsx! {
        DocPage {
            title: "TimePicker",
            source: "libero/src/components/form/date/time_picker.rs",
            markdown: "/md/time_picker.md",
            properties: vec![
                props("TimePicker", vec![
                    prop("value", "Option<NaiveTime>").doc("The picked time; strictly controlled."),
                    prop("onchange", "EventHandler<Option<NaiveTime>>").doc("Called with the time the caller should hold next."),
                    prop("variant", "TimePickerVariant").default("digital").doc("Scrolling columns, or an analog clock face that takes the hour, then the minute."),
                    prop("with_seconds", "bool").default("false").doc("A seconds column. Digital only."),
                    prop("step", "u8").default("1").doc("Minutes between the offered minutes."),
                    prop("twelve_hour", "bool").default("theme").doc("A 12-hour clock with AM and PM. Defaults to whether `DateDefaults::time_format` is one."),
                    prop("min", "NaiveTime").doc("The earliest time that can be picked."),
                    prop("max", "NaiveTime").doc("The latest time that can be picked."),
                    prop("size", "Size").default("md").doc("Option and face size."),
                    prop("name", "String").doc("Posts the time as `HH:MM:SS`."),
                    prop("focusable", "bool").default("true").doc("`false` keeps the buttons out of the tab order."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A time to pick, as columns of hours, minutes and seconds, or as a clock face. The value is a "
                    Code { source: "NaiveTime" }
                    ": a wall-clock time with no date and no time zone. A 24-hour face rings 13 to 00 inside 1 to 12."
                }
            },
            Demo {
                component: "TimePicker",
                children_text: "",
                controls: vec![
                    Control::slider("size", SIZES).default("md"),
                    Control::toggle("variant", ["digital", "analog"]).default("digital"),
                    Control::select("step", ["1", "5", "15"]).default("1").code(|_, values| {
                        match values.str("step").as_str() {
                            "1" => vec![],
                            step => vec![format!("step: {step}u8")],
                        }
                    }),
                    Control::switch("with_seconds"),
                    Control::switch("twelve_hour"),
                    Control::switch("limits").code(|_, values| match is_on(values, "limits") {
                        true => vec![
                            "min: NaiveTime::from_hms_opt(8, 0, 0)".to_string(),
                            "max: NaiveTime::from_hms_opt(18, 30, 0)".to_string(),
                        ],
                        false => vec![],
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    TimePickerDemo { values }
                },
            }
        }
    }
}

#[component]
fn TimePickerDemo(values: DemoValues) -> Element {
    let mut time = use_signal(|| NaiveTime::from_hms_opt(9, 30, 0));
    let limits = is_on(&values, "limits");
    rsx! {
        Flex { direction: "column", gap: "sm", align: "flex-start",
            TimePicker {
                value: time(),
                onchange: move |next| time.set(next),
                size: values.str("size"),
                variant: values.str("variant"),
                step: values.str("step").parse::<u8>().ok(),
                with_seconds: is_on(&values, "with_seconds"),
                twelve_hour: is_on(&values, "twelve_hour"),
                min: limits.then(|| NaiveTime::from_hms_opt(8, 0, 0)).flatten(),
                max: limits.then(|| NaiveTime::from_hms_opt(18, 30, 0)).flatten(),
            }
            Text { size: "sm", {shown(time())} }
        }
    }
}
