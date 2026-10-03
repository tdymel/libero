use super::date_common::{
    calendar_controls, day_limits, has_days, has_time, is_mini, is_on, is_weekend, moment_limits,
    shared_controls, shown, step_of, time_limits, today_of, twelve_hour_of,
};
use super::dropdown_parts::chrono_picker_parts;
use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use libero::components::{ChronoPicker, Code, DateLevel, DateRange, Flex, Text};

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
pub fn ChronoPickerPage() -> Element {
    let theme = libero::use_theme();

    rsx! {
        DocPage {
            title: "ChronoPicker",
            source: "libero/src/components/form/date/chrono_picker.rs",
            markdown: "/md/chrono_picker.md",
            properties: vec![
                props("ChronoPicker<V: DateValue>", vec![
                    prop("value", "Option<V>").doc("The picked value. Its type picks what the picker draws. Pair it with `onchange`."),
                    prop("onchange", "EventHandler<Option<V>>").doc("Called with the value to hold next."),
                    prop("level", "DateLevel").default("Day").doc("Picks a `NaiveDate` as a day, a month (its first day) or a year (its January 1). Ignored for other values."),
                    prop("min", "V::Bound").doc("The earliest value that can be picked. For a range, the earliest end."),
                    prop("max", "V::Bound").doc("The latest value that can be picked. For a range, the latest end."),
                    prop("exclude_date", "Callback<NaiveDate, bool>")
                        .doc("Days that cannot be picked, on top of `min` and `max`."),
                    prop("allow_deselect", "bool").default("false").doc("Clicking the picked day again clears it. Only for a day."),
                    prop("columns", "usize").default("1, or 2 for a range above sm").doc("Months side by side, for a day or a range of days."),
                    prop("calendar", "CalendarVariant").default(theme.chrono_picker.calendar.as_str()).doc("A month of days, or `mini`, one row of days with buttons that page it. For a day or a date-time."),
                    prop("days", "usize").default(theme.chrono_picker.days.to_string()).doc("Days in the mini calendar's row."),
                    prop("variant", "TimePickerVariant").default(theme.time_picker.variant.as_str()).doc("A digital clock, `HH:MM` with a column to turn per part, or a clock face, for values with a time."),
                    prop("with_seconds", "bool").default("false").doc("A seconds column. Digital only."),
                    prop("step", "u8").default(theme.time_picker.step.to_string()).doc("Minutes between the offered minutes."),
                    prop("twelve_hour", "bool").default("formats").doc("A 12-hour clock with AM and PM. Defaults to whether `Formats::time` is one."),
                    prop("today", "NaiveDate").doc("The day marked as today. Unset, the platform clock answers after mount on the web. Elsewhere no day is marked."),
                    prop("size", "Size").default(theme.chrono_picker.size.as_str()).doc("Cell, option and font size."),
                    prop("name", "String").doc("Posts the value as ISO 8601 in a hidden input of that name."),
                    prop("focusable", "bool").default("true").doc("`false` keeps the picker out of the tab order, for a picker inside a dropdown whose input keeps focus."),
                ])
                .parts("ChronoPickerPart", chrono_picker_parts()),
            ],
            accessibility: a11y()
                .key(["Left", "Right"], "Days: the day before or after. Months and years: the cell before or after. Mini calendar: a day, sliding the row one day past its ends.")
                .key(["Up", "Down"], "Days: a week earlier or later. Months and years: a row up or down.")
                .key(["Home", "End"], "Days: the first or last day of the week. Months, years and the mini calendar: the row's ends.")
                .key(["PageUp", "PageDown"], "Days: a month. Months: a year. Years: a decade. Mini calendar: `days` days.")
                .key(["Shift+PageUp", "Shift+PageDown"], "Days: a year.")
                .key(["Enter", "Space"], "Picks the cell.")
                .key(["Enter"], "On the month heading: climbs to the months, focus on the year heading. On the year heading: climbs to the years, focus into them.")
                .key(["Up", "Right"], "Analog clock: the hand forward by an hour or `step` minutes. Digital column (`Up`): a step forward.")
                .key(["Down", "Left"], "Analog clock: the hand back. Digital column (`Down`): a step back.")
                .key(["PageUp", "PageDown"], "Digital column: a bigger step.")
                .key(["Home", "End"], "Digital column: the first or last value.")
                .key(["Digit"], "Digital column: picks. A filled column moves on to the next.")
                .key(["Enter"], "Analog clock: from the hour to the minute. Digital column: to the next column.")
                .key(["Tab"], "Analog clock: to the next control. Digital column: to the next column.")
                .handles([
                    "Days, months and years are one tab stop each, on the picked cell, else today, else the first.",
                    "Today's day, month and year carry `aria-current=\"date\"`.",
                    "The decade heading is disabled, since there is no level above it.",
                    "Picking a month or a year below the lowest level climbs back down with focus on it.",
                    "The mini calendar's days are one tab stop. Its two buttons page the row by `days`.",
                    "The analog clock face is one tab stop. Each digital column is a spinbutton and one tab stop.",
                    "The clock keys change the value at once and skip what `min` and `max` rule out.",
                    "The wheel and a drag turn a digital column too, and a press on the value above or below picks it.",
                    "A date-time's day and time are tabs above the picker, and picking the day moves focus into the clock.",
                    "A date-time range has three tabs: the days, the start time and the end time. Each shows its value once picked, such as `12–14 Oct` or `09:00`, and moves on to the next when complete. The start and the end may be on different days.",
                ]),
            lead: rsx! {
                Text {
                    "One picker for every date and time value. The value's type picks what it draws. "
                    Code { source: "NaiveDate" }
                    " a month of days, "
                    Code { source: "NaiveTime" }
                    " a clock, "
                    Code { source: "NaiveDateTime" }
                    " the day and then the time, a "
                    Code { source: "DateRange" }
                    " of either a start and an end, and a "
                    Code { source: "TimeDelta" }
                    " a duration, one column per part. "
                    Code { source: "level" }
                    " turns a day picker into a month or a year picker, and "
                    Code { source: "calendar: \"mini\"" }
                    " into one row of days."
                }
                Text {
                    "It opens on the value's month, else today's. Names come from the "
                    "localization's "
                    Code { source: "DateLocale" }
                    ", and the first weekday and heading format from the provider's "
                    Code { source: "Formats" }
                    ". As on "
                    Code { source: "ChronoField" }
                    ", a typed handler or a turbofish names the value type. For one value type "
                    "there are "
                    Code { source: "DatePicker" }
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
            // snippet: item use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime};
            // snippet: let mut date = use_signal(|| None::<NaiveDate>);
            // snippet: let mut month = use_signal(|| None::<NaiveDate>);
            // snippet: let mut year = use_signal(|| None::<NaiveDate>);
            // snippet: let mut time = use_signal(|| None::<NaiveTime>);
            // snippet: let mut date_time = use_signal(|| None::<NaiveDateTime>);
            // snippet: let mut date_range = use_signal(|| None::<DateRange<NaiveDate>>);
            // snippet: let mut date_time_range = use_signal(|| None::<DateRange<NaiveDateTime>>);
            Demo {
                component: "ChronoPicker",
                children_text: "",
                controls: [vec![
                    Control::select("value", KINDS).default("date").code(|_, values| {
                        // A typed `value` alone does not name `V`: the
                        // `onchange` is what makes the snippet compile.
                        let (name, kind, level) = match values.str("value").as_str() {
                            "month" => ("month", "Option<NaiveDate>", Some("Month")),
                            "year" => ("year", "Option<NaiveDate>", Some("Year")),
                            "time" => ("time", "Option<NaiveTime>", None),
                            "date-time" => ("date_time", "Option<NaiveDateTime>", None),
                            "date-range" => ("date_range", "Option<DateRange<NaiveDate>>", None),
                            "date-time-range" => ("date_time_range", "Option<DateRange<NaiveDateTime>>", None),
                            _ => ("date", "Option<NaiveDate>", None),
                        };
                        let mut code = vec![
                            format!("value: {name}() /* {kind} */"),
                            format!("onchange: move |next| {name}.set(next)"),
                        ];
                        code.extend(level.map(|level| format!("level: DateLevel::{level}")));
                        code
                    }),
                    Control::sizes("size").default("md"),
                    Control::toggle("variant", ["analog", "digital"]).labels(["Analog", "Digital"]).default("analog").hidden_when(|values| !has_time(values)).code(|_, values| {
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
                    ChronoPickerDemo { values }
                },
            }
        }
    }
}

/// One signal per value type, so switching types keeps what was picked.
#[component]
fn ChronoPickerDemo(values: DemoValues) -> Element {
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
                ChronoPicker {
                    value: month(), onchange: move |next| month.set(next), level: DateLevel::Month,
                    min: min_day, max: max_day, today, size,
                }
            },
            shown(month()),
        ),
        "year" => (
            rsx! {
                ChronoPicker {
                    value: year(), onchange: move |next| year.set(next), level: DateLevel::Year,
                    min: min_day, max: max_day, today, size,
                }
            },
            shown(year()),
        ),
        "time" => (
            rsx! {
                ChronoPicker {
                    value: time(), onchange: move |next| time.set(next),
                    min: min_time, max: max_time, variant, with_seconds, step, twelve_hour, size,
                }
            },
            shown(time()),
        ),
        "date-time" => (
            rsx! {
                ChronoPicker {
                    value: date_time(), onchange: move |next| date_time.set(next),
                    min: min_moment, max: max_moment, exclude_date, today, calendar, days,
                    variant, with_seconds, step, twelve_hour, size,
                }
            },
            shown(date_time()),
        ),
        "date-range" => (
            rsx! {
                ChronoPicker {
                    value: date_range(), onchange: move |next| date_range.set(next),
                    min: min_day, max: max_day, exclude_date, columns, today, size,
                }
            },
            shown(date_range()),
        ),
        "date-time-range" => (
            rsx! {
                ChronoPicker {
                    value: date_time_range(), onchange: move |next| date_time_range.set(next),
                    min: min_moment, max: max_moment, exclude_date, today,
                    variant, with_seconds, step, twelve_hour, size,
                }
            },
            shown(date_time_range()),
        ),
        _ => (
            rsx! {
                ChronoPicker {
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
