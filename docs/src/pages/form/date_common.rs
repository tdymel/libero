//! What the DateField and DatePicker demos share: the caption and state
//! controls of a field, the controls both demos have, and reading them back.

use crate::components::{Control, DemoValues};
use libero::chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime};
use libero::components::FieldStatus;

pub const SIZES: [&str; 6] = ["xs", "sm", "md", "lg", "xl", "xxl"];

pub fn is_on(values: &DemoValues, name: &str) -> bool {
    values.str(name) == "true"
}

pub fn is_weekend(day: NaiveDate) -> bool {
    day.weekday().num_days_from_monday() >= 5
}

pub fn status_of(values: &DemoValues) -> FieldStatus {
    match values.str("status").as_str() {
        "warning" => FieldStatus::Warning("Double-check this.".to_string()),
        "error" => FieldStatus::Error("This is required.".to_string()),
        _ => FieldStatus::Valid,
    }
}

/// `label` and friends, the way every field page has them.
pub fn field_controls() -> Vec<Control> {
    vec![
        Control::toggle("status", ["valid", "warning", "error"])
            .default("valid")
            .code(|_, values| match values.str("status").as_str() {
                "warning" => {
                    vec![r#"status: FieldStatus::Warning("Double-check this.".into())"#.to_string()]
                }
                "error" => vec![r#"status: "This is required.""#.to_string()],
                _ => vec![],
            }),
        Control::switch("placeholder").code(|_, values| match is_on(values, "placeholder") {
            true => vec![r#"placeholder: "Pick one""#.to_string()],
            false => vec![],
        }),
        Control::switch("label")
            .default("true")
            .code(|_, values| match is_on(values, "label") {
                true => vec![r#"label: "When""#.to_string()],
                false => vec![],
            }),
        Control::switch("description").code(|_, values| match is_on(values, "description") {
            true => vec![r#"description: "Your local time.""#.to_string()],
            false => vec![],
        }),
        Control::switch("helper").code(|_, values| match is_on(values, "helper") {
            true => vec![r#"helper: "Typing works too.""#.to_string()],
            false => vec![],
        }),
        Control::switch("required"),
        Control::switch("disabled"),
    ]
}

pub fn text_of(values: &DemoValues, name: &str, text: &str) -> Option<String> {
    is_on(values, name).then(|| text.to_string())
}

/// `value: ...` under a demo, `None` spelled out.
pub fn shown(value: Option<impl std::fmt::Display>) -> String {
    match value {
        Some(value) => format!("value: {value}"),
        None => "value: None".to_string(),
    }
}

/// Whether the demoed value type has a time, so the clock props apply.
pub fn has_time(values: &DemoValues) -> bool {
    matches!(
        values.str("value").as_str(),
        "time" | "date-time" | "date-time-range"
    )
}

/// Whether the demoed value type is picked from days, so `exclude_date`
/// applies.
pub fn has_days(values: &DemoValues) -> bool {
    !matches!(values.str("value").as_str(), "time" | "month" | "year")
}

/// Whether the demoed value type is picked from a calendar that can be mini.
pub fn has_calendar(values: &DemoValues) -> bool {
    matches!(values.str("value").as_str(), "date" | "date-time")
}

/// Whether the demo shows the mini calendar.
pub fn is_mini(values: &DemoValues) -> bool {
    has_calendar(values) && values.str("calendar") == "mini"
}

/// `calendar` and `days`, for the value types that draw days in one piece.
pub fn calendar_controls() -> Vec<Control> {
    vec![
        Control::toggle("calendar", ["full", "mini"])
            .default("full")
            .hidden_when(|values| !has_calendar(values))
            .code(|_, values| match values.str("calendar").as_str() {
                "mini" => vec![r#"calendar: "mini""#.to_string()],
                _ => vec![],
            }),
        Control::toggle("days", ["5", "7", "10"])
            .default("7")
            .hidden_when(|values| !is_mini(values))
            .code(|_, values| match values.str("days").as_str() {
                "7" => vec![],
                days => vec![format!("days: {days}")],
            }),
    ]
}

/// The controls both demos have beyond their own: the limits, the clock's
/// minute step and hour cycle, a fixed today and a posted name.
pub fn shared_controls() -> Vec<Control> {
    vec![
        Control::toggle("step", ["1", "5", "15", "30"])
            .default("5")
            .hidden_when(|values| !has_time(values))
            .code(|_, values| match values.str("step").as_str() {
                "5" => vec![],
                step => vec![format!("step: {step}")],
            }),
        Control::switch("min_max").code(|_, values| match is_on(values, "min_max") {
            true => limits_code(values),
            false => vec![],
        }),
        Control::switch("with_seconds").hidden_when(|values| !has_time(values)),
        Control::switch("twelve_hour").hidden_when(|values| !has_time(values)),
        Control::switch("today").code(|_, values| match is_on(values, "today") {
            true => vec!["today: NaiveDate::from_ymd_opt(2026, 9, 20)".to_string()],
            false => vec![],
        }),
    ]
}

pub fn step_of(values: &DemoValues) -> Option<u8> {
    values.str("step").parse().ok()
}

/// `None` leaves the hour cycle to the theme or the time format.
pub fn twelve_hour_of(values: &DemoValues) -> Option<bool> {
    is_on(values, "twelve_hour").then_some(true)
}

pub fn today_of(values: &DemoValues) -> Option<NaiveDate> {
    is_on(values, "today")
        .then(|| NaiveDate::from_ymd_opt(2026, 9, 20))
        .flatten()
}

fn limits_code(values: &DemoValues) -> Vec<String> {
    let (min, max) = match values.str("value").as_str() {
        "month" => (
            "NaiveDate::from_ymd_opt(2026, 3, 1)",
            "NaiveDate::from_ymd_opt(2026, 10, 1)",
        ),
        "year" => (
            "NaiveDate::from_ymd_opt(2022, 1, 1)",
            "NaiveDate::from_ymd_opt(2028, 1, 1)",
        ),
        "time" => (
            "NaiveTime::from_hms_opt(8, 0, 0)",
            "NaiveTime::from_hms_opt(18, 0, 0)",
        ),
        "date-time" | "date-time-range" => (
            "NaiveDate::from_ymd_opt(2026, 9, 5).and_then(|day| day.and_hms_opt(8, 0, 0))",
            "NaiveDate::from_ymd_opt(2026, 9, 25).and_then(|day| day.and_hms_opt(18, 0, 0))",
        ),
        _ => (
            "NaiveDate::from_ymd_opt(2026, 9, 5)",
            "NaiveDate::from_ymd_opt(2026, 9, 25)",
        ),
    };
    vec![format!("min: {min}"), format!("max: {max}")]
}

type Limits<T> = (Option<T>, Option<T>);

/// The days `min_max` limits a day, a range of days, a month or a year to.
pub fn day_limits(values: &DemoValues) -> Limits<NaiveDate> {
    if !is_on(values, "min_max") {
        return (None, None);
    }
    match values.str("value").as_str() {
        "month" => (
            NaiveDate::from_ymd_opt(2026, 3, 1),
            NaiveDate::from_ymd_opt(2026, 10, 1),
        ),
        "year" => (
            NaiveDate::from_ymd_opt(2022, 1, 1),
            NaiveDate::from_ymd_opt(2028, 1, 1),
        ),
        _ => (
            NaiveDate::from_ymd_opt(2026, 9, 5),
            NaiveDate::from_ymd_opt(2026, 9, 25),
        ),
    }
}

pub fn time_limits(values: &DemoValues) -> Limits<NaiveTime> {
    match is_on(values, "min_max") {
        true => (
            NaiveTime::from_hms_opt(8, 0, 0),
            NaiveTime::from_hms_opt(18, 0, 0),
        ),
        false => (None, None),
    }
}

pub fn moment_limits(values: &DemoValues) -> Limits<NaiveDateTime> {
    match is_on(values, "min_max") {
        true => (
            NaiveDate::from_ymd_opt(2026, 9, 5).and_then(|day| day.and_hms_opt(8, 0, 0)),
            NaiveDate::from_ymd_opt(2026, 9, 25).and_then(|day| day.and_hms_opt(18, 0, 0)),
        ),
        false => (None, None),
    }
}
