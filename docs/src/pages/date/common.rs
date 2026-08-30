//! What every date and time page's demo shares: the caption and state
//! controls of a field, and reading them back.

use crate::components::{Control, DemoValues};
use libero::chrono::{Datelike, NaiveDate};
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
