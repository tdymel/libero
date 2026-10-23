//! What the ChronoField and ChronoPicker demos share: the caption and state
//! controls of a field, the controls both demos have, and reading them back.

use crate::components::{Control, DemoValues};
use libero::chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta};
use libero::components::{DateLevel, DatePreset, DateRange, FieldStatus};

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

/// The label and description for the picked value kind: a duration is no moment.
pub fn captions_of(values: &DemoValues) -> (&'static str, &'static str) {
    match values.str("value").as_str() {
        "duration" => ("How long", "Hours and minutes it takes."),
        _ => ("When", "Your local time."),
    }
}

/// `label` and friends, the way every field page has them.
pub fn field_controls() -> Vec<Control> {
    vec![
        Control::toggle("status", ["valid", "warning", "error"])
            .labels(["Valid", "Warning", "Error"])
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
                true => vec![format!("label: {:?}", captions_of(values).0)],
                false => vec![format!("aria_label: {:?}", captions_of(values).0)],
            }),
        Control::switch("description").code(|_, values| match is_on(values, "description") {
            true => vec![format!("description: {:?}", captions_of(values).1)],
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

/// Whether the demoed value type has minutes to step, so `step` and
/// `with_seconds` apply: a time or a duration.
pub fn has_clock(values: &DemoValues) -> bool {
    has_time(values) || values.str("value") == "duration"
}

/// Whether the demoed value type is picked from days, so `exclude_date`
/// applies.
pub fn has_days(values: &DemoValues) -> bool {
    !matches!(
        values.str("value").as_str(),
        "time" | "month" | "year" | "duration"
    )
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
            .labels(["Full", "Mini"])
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

/// Whether the demo draws months of days, so the `grid` props apply.
pub fn has_grid(values: &DemoValues) -> bool {
    has_days(values) && !is_mini(values)
}

/// Whether the demoed value type takes `presets`: a day or a range of days.
pub fn has_presets(values: &DemoValues) -> bool {
    matches!(values.str("value").as_str(), "date" | "date-range")
}

/// The month grid's switches, `presets` and `open_to`.
pub fn grid_controls() -> Vec<Control> {
    vec![
        Control::switch("week_numbers").hidden_when(|values| !has_grid(values)),
        Control::switch("hide_weekdays").hidden_when(|values| !has_grid(values)),
        Control::switch("hide_outside_days").hidden_when(|values| !has_grid(values)),
        Control::switch("presets")
            .hidden_when(|values| !has_presets(values))
            .code(|_, values| match is_on(values, "presets") {
                true => vec![presets_code(values)],
                false => vec![],
            }),
        Control::toggle("open_to", ["auto", "month", "year"])
            .labels(["Auto", "Month", "Year"])
            .default("auto")
            .hidden_when(|values| !matches!(values.str("value").as_str(), "date" | "month"))
            .code(|_, values| match values.str("open_to").as_str() {
                "month" => vec!["open_to: DateLevel::Month".to_string()],
                "year" => vec!["open_to: DateLevel::Year".to_string()],
                _ => vec![],
            }),
    ]
}

pub fn open_to_of(values: &DemoValues) -> Option<DateLevel> {
    match values.str("open_to").as_str() {
        "month" => Some(DateLevel::Month),
        "year" => Some(DateLevel::Year),
        _ => None,
    }
}

/// Three days, or three ranges, to pick in one press.
pub fn day_presets(values: &DemoValues) -> Vec<DatePreset<NaiveDate>> {
    let day = |month, day| NaiveDate::from_ymd_opt(2026, month, day).expect("a day");
    match is_on(values, "presets") {
        true => vec![
            DatePreset::new("Start of month", day(9, 1)),
            DatePreset::new("End of month", day(9, 30)),
            DatePreset::new("Next month", day(10, 1)),
        ],
        false => Vec::new(),
    }
}

pub fn range_presets(values: &DemoValues) -> Vec<DatePreset<DateRange<NaiveDate>>> {
    let day = |month, day| NaiveDate::from_ymd_opt(2026, month, day).expect("a day");
    let range = |label, start, end| DatePreset::new(label, DateRange::new(start, Some(end)));
    match is_on(values, "presets") {
        true => vec![
            range("First week", day(9, 1), day(9, 7)),
            range("Second half", day(9, 15), day(9, 30)),
            range("October", day(10, 1), day(10, 31)),
        ],
        false => Vec::new(),
    }
}

fn presets_code(values: &DemoValues) -> String {
    let day = |month, day| format!("NaiveDate::from_ymd_opt(2026, {month}, {day}).unwrap()");
    let presets = match values.str("value").as_str() {
        "date-range" => [
            ("First week", (9, 1), (9, 7)),
            ("Second half", (9, 15), (9, 30)),
            ("October", (10, 1), (10, 31)),
        ]
        .map(|(label, start, end)| {
            format!(
                "DatePreset::new({label:?}, DateRange::new({}, Some({})))",
                day(start.0, start.1),
                day(end.0, end.1)
            )
        })
        .to_vec(),
        _ => [
            ("Start of month", (9, 1)),
            ("End of month", (9, 30)),
            ("Next month", (10, 1)),
        ]
        .map(|(label, at)| format!("DatePreset::new({label:?}, {})", day(at.0, at.1)))
        .to_vec(),
    };
    format!("presets: vec![{}]", presets.join(", "))
}

/// The controls both demos have beyond their own: the limits, the clock's
/// minute step and hour cycle, a fixed today and a posted name.
pub fn shared_controls() -> Vec<Control> {
    vec![
        Control::toggle("step", ["1", "5", "15", "30"])
            .default("5")
            .hidden_when(|values| !has_clock(values))
            .code(|_, values| match values.str("step").as_str() {
                "5" => vec![],
                step => vec![format!("step: {step}")],
            }),
        Control::toggle("hour_step", ["1", "2", "3"])
            .default("1")
            .hidden_when(|values| !has_time(values))
            .code(|_, values| match values.str("hour_step").as_str() {
                "1" => vec![],
                step => vec![format!("hour_step: {step}")],
            }),
        // A second step turns the seconds on, so it shows on its own.
        Control::toggle("second_step", ["1", "5", "15"])
            .default("1")
            .hidden_when(|values| !has_time(values))
            .code(|_, values| match values.str("second_step").as_str() {
                "1" => vec![],
                step if is_on(values, "with_seconds") => vec![format!("second_step: {step}")],
                step => vec![
                    format!("second_step: {step}"),
                    "with_seconds: true".to_string(),
                ],
            }),
        Control::switch("min_max").code(|_, values| match is_on(values, "min_max") {
            true => limits_code(values),
            false => vec![],
        }),
        Control::switch("with_seconds").hidden_when(|values| !has_clock(values)),
        Control::switch("twelve_hour").hidden_when(|values| !has_time(values)),
        // A time or a duration has no day to mark.
        Control::switch("today")
            .hidden_when(|values| matches!(values.str("value").as_str(), "time" | "duration"))
            .code(|_, values| match is_on(values, "today") {
                true => vec!["today: NaiveDate::from_ymd_opt(2026, 9, 20)".to_string()],
                false => vec![],
            }),
    ]
}

pub fn step_of(values: &DemoValues) -> Option<u8> {
    values.str("step").parse().ok()
}

pub fn hour_step_of(values: &DemoValues) -> Option<u8> {
    values.str("hour_step").parse().ok()
}

pub fn second_step_of(values: &DemoValues) -> Option<u8> {
    values.str("second_step").parse().ok()
}

/// `with_seconds`, also on for a second step.
pub fn with_seconds_of(values: &DemoValues) -> bool {
    is_on(values, "with_seconds")
        || values
            .str("second_step")
            .parse()
            .is_ok_and(|step: u8| step > 1)
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
        "duration" => ("TimeDelta::try_minutes(15)", "TimeDelta::try_hours(8)"),
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

pub fn duration_limits(values: &DemoValues) -> Limits<TimeDelta> {
    match is_on(values, "min_max") {
        true => (TimeDelta::try_minutes(15), TimeDelta::try_hours(8)),
        false => (None, None),
    }
}
