use super::date_common::{
    SIZES, calendar_controls, day_limits, has_days, has_time, is_mini, is_on, is_weekend,
    moment_limits, shared_controls, shown, step_of, time_limits, today_of, twelve_hour_of,
};
use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use libero::components::{Code, DateLevel, DatePicker, DateRange, Flex, Kbd, Text};

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
                        .doc("Days that cannot be picked."),
                    prop("allow_deselect", "bool").default("false").doc("Clicking the picked day again clears it. Only for a day."),
                    prop("columns", "usize").default("1, or 2 for a range").doc("Months side by side, for a day or a range of days."),
                    prop("calendar", "CalendarVariant").default("full").doc("A month of days, or `mini`: one row of days with buttons that page it. For a day or a date-time."),
                    prop("days", "usize").default("7").doc("Days in the mini calendar's row."),
                    prop("variant", "TimePickerVariant").default("analog").doc("Columns of numbers or a clock face, for values with a time."),
                    prop("with_seconds", "bool").default("false").doc("A seconds column. Digital only."),
                    prop("step", "u8").default("5").doc("Minutes between the offered minutes. Defaults to the theme's `TimePickerDefaults::step`."),
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
                    " turns a day picker into a month or a year picker, and "
                    Code { source: "calendar: \"mini\"" }
                    " into one row of days - Mantine's MiniCalendar."
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
                    ". For one value type there are "
                    Code { source: "DayPicker" }
                    ", "
                    Code { source: "MonthPicker" }
                    ", "
                    Code { source: "YearPicker" }
                    ", "
                    Code { source: "TimePicker" }
                    " and "
                    Code { source: "DateRangePicker" }
                    ", with only the props that type uses and no turbofish."
                }
            },
            Demo {
                component: "DatePicker",
                children_text: "",
                controls: [vec![
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
                    Control::toggle("variant", ["analog", "digital"]).default("analog").hidden_when(|values| !has_time(values)).code(|_, values| {
                        match values.str("variant").as_str() {
                            "digital" => vec![r#"variant: "digital""#.to_string()],
                            _ => vec![],
                        }
                    }),
                    // A range defaults to two months, so `1` prints there.
                    Control::toggle("columns", ["1", "2", "3"])
                        .default("1")
                        .hidden_when(|values| !matches!(values.str("value").as_str(), "date" | "date-range") || is_mini(values))
                        .code(|_, values| {
                            let default = if values.str("value") == "date-range" { "2" } else { "1" };
                            match values.str("columns").as_str() {
                                columns if columns == default => vec![],
                                columns => vec![format!("columns: {columns}")],
                            }
                        }),
                    Control::switch("allow_deselect").hidden_when(|values| values.str("value") != "date"),
                    Control::switch("exclude_weekends").hidden_when(|values| !has_days(values)).code(|_, values| {
                        match is_on(values, "exclude_weekends") {
                            true => vec!["exclude_date: |day: NaiveDate| day.weekday().num_days_from_monday() >= 5".to_string()],
                            false => vec![],
                        }
                    }),
                ], calendar_controls(), shared_controls()].concat(),
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
                Text {
                    "The mini calendar's days are one tab stop: "
                    Kbd { "←" } " " Kbd { "→" } " move a day and slide the row past its ends, "
                    Kbd { "Home" } " " Kbd { "End" } " go to the row's ends, and "
                    Kbd { "PageUp" } " " Kbd { "PageDown" } " move a row's worth of days. The buttons page the row."
                }
                Text { "A date-time picks the day first; picking it moves focus into the clock." }
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
    let columns = values.str("columns").parse::<usize>().ok();
    let calendar = values.str("calendar");
    let days = values.str("days").parse::<usize>().ok();
    let step = step_of(&values);
    let with_seconds = is_on(&values, "with_seconds");
    let twelve_hour = twelve_hour_of(&values);
    let today = today_of(&values);
    let (min_day, max_day) = day_limits(&values);
    let (min_time, max_time) = time_limits(&values);
    let (min_moment, max_moment) = moment_limits(&values);

    let (picker, readout) = match values.str("value").as_str() {
        "month" => (
            rsx! {
                DatePicker {
                    value: month(), onchange: move |next| month.set(next), level: DateLevel::Month,
                    min: min_day, max: max_day, today, size,
                }
            },
            shown(month()),
        ),
        "year" => (
            rsx! {
                DatePicker {
                    value: year(), onchange: move |next| year.set(next), level: DateLevel::Year,
                    min: min_day, max: max_day, today, size,
                }
            },
            shown(year()),
        ),
        "time" => (
            rsx! {
                DatePicker {
                    value: time(), onchange: move |next| time.set(next),
                    min: min_time, max: max_time, variant, with_seconds, step, twelve_hour, size,
                }
            },
            shown(time()),
        ),
        "date-time" => (
            rsx! {
                DatePicker {
                    value: date_time(), onchange: move |next| date_time.set(next),
                    min: min_moment, max: max_moment, exclude_date, today, calendar, days,
                    variant, with_seconds, step, twelve_hour, size,
                }
            },
            shown(date_time()),
        ),
        "date-range" => (
            rsx! {
                DatePicker {
                    value: date_range(), onchange: move |next| date_range.set(next),
                    min: min_day, max: max_day, exclude_date, columns, today, size,
                }
            },
            shown(date_range()),
        ),
        "date-time-range" => (
            rsx! {
                DatePicker {
                    value: date_time_range(), onchange: move |next| date_time_range.set(next),
                    min: min_moment, max: max_moment, exclude_date, today,
                    variant, with_seconds, step, twelve_hour, size,
                }
            },
            shown(date_time_range()),
        ),
        _ => (
            rsx! {
                DatePicker {
                    value: date(), onchange: move |next| date.set(next),
                    min: min_day, max: max_day, allow_deselect, exclude_date, columns, calendar, days, today, size,
                }
            },
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
