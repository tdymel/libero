use super::date_common::{SIZES, is_on, is_weekend, shown};
use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use libero::components::{Code, DateLevel, DatePicker, DateRange, Flex, Kbd, List, ListItem, Text};

const KINDS: [&str; 7] = [
    "date",
    "month",
    "year",
    "time",
    "date-time",
    "date-range",
    "date-time-range",
];

fn day(day: u32) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(2026, 9, day)
}

fn moment(day: u32, hour: u32) -> Option<NaiveDateTime> {
    NaiveDate::from_ymd_opt(2026, 9, day)
        .zip(NaiveTime::from_hms_opt(hour, 0, 0))
        .map(|(day, time)| NaiveDateTime::new(day, time))
}

#[component]
pub fn DatePickerPage() -> Element {
    rsx! {
        DocPage {
            title: "DatePicker",
            source: "libero/src/components/form/date/date_picker.rs",
            markdown: "/md/date_picker.md",
            properties: vec![
                props("DatePicker<V: DateValue>", vec![
                    prop("value", "Option<V>").doc("The picked value; strictly controlled. Its type picks the picker."),
                    prop("onchange", "EventHandler<Option<V>>").doc("Called with the value the caller should hold next."),
                    prop("level", "DateLevel").default("Day").doc("Picks a `NaiveDate` as a day, a month (its first day) or a year (its January 1). Ignored for every other value."),
                    prop("min", "V::Bound").doc("The earliest value that can be picked. For a range, the earliest end."),
                    prop("max", "V::Bound").doc("The latest value, likewise."),
                    prop("exclude_date", "Callback<NaiveDate, bool>")
                        .doc("Days that cannot be picked. A `Callback` always compares equal, so changing only this closure does not redraw the picker."),
                    prop("allow_deselect", "bool").default("false").doc("Clicking the picked day again clears it. Only for a day."),
                    prop("columns", "usize").default("1, or 2 for a range").doc("Months side by side, for a day or a range of days."),
                    prop("variant", "TimePickerVariant").default("analog").doc("Columns of numbers or a clock face, for values with a time."),
                    prop("with_seconds", "bool").default("false").doc("A seconds column. Digital only."),
                    prop("step", "u8").default("1").doc("Minutes between the offered minutes."),
                    prop("twelve_hour", "bool").default("theme").doc("A 12-hour clock. Defaults to whether `DateDefaults::time_format` is one."),
                    prop("today", "NaiveDate").doc("The day marked as today. Unset, the platform clock answers after mount - on the web; elsewhere no day is marked."),
                    prop("size", "Size").default("md").doc("Cell, option and font size."),
                    prop("name", "String").doc("Emits a hidden input of that name, posting the value as ISO 8601."),
                    prop("focusable", "bool").default("true").doc("`false` keeps the picker out of the tab order, for a picker inside a dropdown whose input keeps focus."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "One picker for every date and time value. The value's type picks what it draws: "
                    Code { source: "NaiveDate" }
                    " a month of days, "
                    Code { source: "NaiveTime" }
                    " a clock, "
                    Code { source: "NaiveDateTime" }
                    " the day and then the time, and a "
                    Code { source: "DateRange" }
                    " of either a start and an end. "
                    Code { source: "level" }
                    " turns a day picker into a month or a year picker."
                }
                Text {
                    "Controlled through "
                    Code { source: "value" }
                    " and "
                    Code { source: "onchange" }
                    ". The month shown is the picker's own state: it opens on the value's month, else today's. "
                    "Names, the first weekday and the heading format come from the theme's "
                    Code { source: "DateDefaults" }
                    ". A typed "
                    Code { source: "value" }
                    " alone does not name the type - a typed handler or a turbofish does, as on "
                    Code { source: "DateField" }
                    "."
                }
            },
            Demo {
                component: "DatePicker",
                children_text: "",
                controls: vec![
                    Control::select("value", KINDS).default("date").code(|_, values| {
                        match values.str("value").as_str() {
                            "month" => vec!["value: month() /* Option<NaiveDate> */".to_string(), "level: DateLevel::Month".to_string()],
                            "year" => vec!["value: year() /* Option<NaiveDate> */".to_string(), "level: DateLevel::Year".to_string()],
                            "time" => vec!["value: time() /* Option<NaiveTime> */".to_string()],
                            "date-time" => vec!["value: moment() /* Option<NaiveDateTime> */".to_string()],
                            "date-range" => vec!["value: range() /* Option<DateRange<NaiveDate>> */".to_string()],
                            "date-time-range" => vec!["value: range() /* Option<DateRange<NaiveDateTime>> */".to_string()],
                            _ => vec!["value: day() /* Option<NaiveDate> */".to_string()],
                        }
                    }),
                    Control::slider("size", SIZES).default("md"),
                    Control::toggle("variant", ["analog", "digital"]).default("analog").code(|_, values| {
                        match values.str("variant").as_str() {
                            "digital" => vec![r#"variant: "digital""#.to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("allow_deselect"),
                    Control::switch("exclude_weekends").code(|_, values| {
                        match is_on(values, "exclude_weekends") {
                            true => vec!["exclude_date: |day: NaiveDate| day.weekday().num_days_from_monday() >= 5".to_string()],
                            false => vec![],
                        }
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    DatePickerDemo { values }
                },
            }

            DocSection {
                title: "Keyboard",
                Text {
                    "Days, months and years are one tab stop each: the picked cell, else today, else the first. "
                    Kbd { "←" } " " Kbd { "→" } " move a cell, "
                    Kbd { "↑" } " " Kbd { "↓" } " a week or a row, "
                    Kbd { "Home" } " " Kbd { "End" } " to the ends of the week or row, and "
                    Kbd { "PageUp" } " " Kbd { "PageDown" } " page a month, a year or a decade - with "
                    Kbd { "Shift" } " a year among days. "
                    Kbd { "Enter" } " and " Kbd { "Space" } " pick. Enter on a heading climbs a level and keeps focus on the new heading."
                }
                Text {
                    "The analog clock face is one tab stop: "
                    Kbd { "↑" } " " Kbd { "→" } " step the hand forward - an hour, or "
                    Code { source: "step" }
                    " minutes - and "
                    Kbd { "↓" } " " Kbd { "←" } " back, skipping what "
                    Code { source: "min" }
                    " and "
                    Code { source: "max" }
                    " rule out. "
                    Kbd { "Enter" } " moves from the hour to the minute. "
                    "Digital columns are one tab stop each: "
                    Kbd { "↑" } " " Kbd { "↓" } " " Kbd { "Home" } " " Kbd { "End" } " move within the column, "
                    Kbd { "Enter" } " picks, and " Kbd { "Tab" } " goes to the next column."
                }
                Text { "A date-time picks the day first; picking it moves focus into the clock." }
            }
            DocSection {
                title: "Alternatives",
                Text { "The same picker for one value type each, with only the props that type uses and no turbofish." }
                List {
                    ListItem { Code { source: "DayPicker" } " - a " Code { source: "NaiveDate" } " day." }
                    ListItem { Code { source: "MonthPicker" } " - a month, as " Code { source: "level: DateLevel::Month" } "." }
                    ListItem { Code { source: "YearPicker" } " - a year, as " Code { source: "level: DateLevel::Year" } "." }
                    ListItem { Code { source: "TimePicker" } " - a " Code { source: "NaiveTime" } ", digital or analog." }
                    ListItem { Code { source: "DateRangePicker" } " - a " Code { source: "DateRange<NaiveDate>" } "." }
                }
            }
        }
    }
}

/// One signal per value type, so switching types keeps what was picked.
#[component]
fn DatePickerDemo(values: DemoValues) -> Element {
    let mut date = use_signal(|| day(14));
    let mut month = use_signal(|| day(1));
    let mut year = use_signal(|| NaiveDate::from_ymd_opt(2026, 1, 1));
    let mut time = use_signal(|| NaiveTime::from_hms_opt(9, 30, 0));
    let mut date_time = use_signal(|| moment(14, 9));
    let mut date_range = use_signal(|| day(14).map(|start| DateRange::new(start, day(18))));
    let mut date_time_range =
        use_signal(|| moment(14, 9).map(|start| DateRange::new(start, moment(16, 17))));

    let size = values.str("size");
    let variant = values.str("variant");
    let allow_deselect = is_on(&values, "allow_deselect");
    let exclude_date = is_on(&values, "exclude_weekends").then(|| Callback::new(is_weekend));

    let (picker, readout) = match values.str("value").as_str() {
        "month" => (
            rsx! { DatePicker { value: month(), onchange: move |next| month.set(next), level: DateLevel::Month, size } },
            shown(month()),
        ),
        "year" => (
            rsx! { DatePicker { value: year(), onchange: move |next| year.set(next), level: DateLevel::Year, size } },
            shown(year()),
        ),
        "time" => (
            rsx! { DatePicker { value: time(), onchange: move |next| time.set(next), variant, size } },
            shown(time()),
        ),
        "date-time" => (
            rsx! { DatePicker { value: date_time(), onchange: move |next| date_time.set(next), variant, exclude_date, size } },
            shown(date_time()),
        ),
        "date-range" => (
            rsx! { DatePicker { value: date_range(), onchange: move |next| date_range.set(next), exclude_date, size } },
            shown(date_range()),
        ),
        "date-time-range" => (
            rsx! { DatePicker { value: date_time_range(), onchange: move |next| date_time_range.set(next), variant, exclude_date, size } },
            shown(date_time_range()),
        ),
        _ => (
            rsx! { DatePicker { value: date(), onchange: move |next| date.set(next), allow_deselect, exclude_date, size } },
            shown(date()),
        ),
    };
    rsx! {
        Flex { direction: "column", gap: "sm", align: "flex-start",
            {picker}
            Text { size: "sm", {readout} }
        }
    }
}
