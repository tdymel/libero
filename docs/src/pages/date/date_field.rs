use super::common::{SIZES, is_on, is_weekend, shown};
use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::chrono::NaiveDate;
use libero::components::{Code, DateField, FieldStatus, Flex, Text};

const FORMATS: [&str; 5] = [
    "MMMM D, YYYY",
    "DD.MM.YYYY",
    "MM/DD/YYYY",
    "YYYY-MM-DD",
    "ddd, D MMM YYYY",
];

#[component]
pub fn DateFieldPage() -> Element {
    rsx! {
        DocPage {
            title: "DateField",
            source: "libero/src/components/form/date/fields.rs",
            markdown: "/md/date_field.md",
            properties: vec![
                props("DateField", vec![
                    prop("value", "Option<NaiveDate>").doc("The day in the field; strictly controlled. `None` is the empty field."),
                    prop("onchange", "EventHandler<Option<NaiveDate>>")
                        .doc("Called when typed text is committed - on blur or Enter - and when a day is picked. Emptied text commits `None`."),
                    prop("format", "String")
                        .default("DateDefaults::format")
                        .doc("How the text shows the day, in dayjs tokens. Typing is lenient either way: only the order of day, month and year has to match."),
                    prop("min", "NaiveDate").doc("The earliest day that can be picked or typed."),
                    prop("max", "NaiveDate").doc("The latest day that can be picked or typed."),
                    prop("exclude_date", "Callback<NaiveDate, bool>").doc("Days that cannot be picked or typed."),
                    prop("today", "NaiveDate").doc("The day marked as today, and the year typed text without one falls back to. Unset, the platform clock answers after mount."),
                    prop("close_on_change", "bool").default("true").doc("Picking a day closes the dropdown."),
                    prop("name", "FieldName<Option<NaiveDate>>").doc("What the field posts as - the day in ISO 8601, whatever `format` shows. A path also binds it to the surrounding `Form`."),
                    prop("placeholder", "String").doc("Shown while the text is empty."),
                    prop("size", "Size").default("md").doc("Control height, font size and the dropdown's calendar."),
                    prop("radius", "Size").default("sm").doc("Corner radius of the frame."),
                    prop("label", "Caption").doc("The field's caption."),
                    prop("description", "Caption").doc("Between the label and the control."),
                    prop("helper", "Caption").doc("Under the control."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, rendered under the helper. Text the field cannot accept shows `DateDefaults::invalid_date` instead."),
                    prop("required", "bool").default("false").doc("Adds `required` to the input and an asterisk to the label."),
                    prop("disabled", "bool").default("false").doc("Disables typing and the dropdown, and dims the field."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A text field holding a "
                    Code { source: "NaiveDate" }
                    ", with a "
                    Code { source: "DatePicker" }
                    " in a dropdown. Controlled through "
                    Code { source: "value" }
                    " and "
                    Code { source: "onchange" }
                    "."
                }
                Text {
                    "Typed text stays as typed until the field blurs or Enter is pressed, then it is read leniently: "
                    "any separator, one-digit days and months, month names in any case or as a unique prefix, "
                    "and a missing year taken from today. Only the order of day, month and year follows "
                    Code { source: "format" }
                    ". Two-digit years are rejected. Text that is not an accepted day stays, and the field shows an error."
                }
            },
            Demo {
                component: "DateField",
                children_text: "",
                controls: vec![
                    Control::slider("size", SIZES).default("md"),
                    Control::slider("radius", SIZES).default("sm"),
                    Control::select("format", FORMATS)
                        .default("MMMM D, YYYY")
                        .code(|_, values| match values.str("format").as_str() {
                            "MMMM D, YYYY" => vec![],
                            format => vec![format!("format: {format:?}")],
                        }),
                    Control::toggle("status", ["valid", "warning", "error"])
                        .default("valid")
                        .code(|_, values| match values.str("status").as_str() {
                            "warning" => vec![
                                r#"status: FieldStatus::Warning("That is a Sunday.".into())"#.to_string(),
                            ],
                            "error" => vec![r#"status: "Pick an arrival day.""#.to_string()],
                            _ => vec![],
                        }),
                    Control::switch("limits").code(|_, values| match is_on(values, "limits") {
                        true => vec![
                            "min: NaiveDate::from_ymd_opt(2026, 9, 5)".to_string(),
                            "max: NaiveDate::from_ymd_opt(2026, 10, 20)".to_string(),
                        ],
                        false => vec![],
                    }),
                    Control::switch("exclude_weekends").code(|_, values| {
                        match is_on(values, "exclude_weekends") {
                            true => vec!["exclude_date: |day: NaiveDate| day.weekday().num_days_from_monday() >= 5".to_string()],
                            false => vec![],
                        }
                    }),
                    Control::switch("close_on_change").default("true").code(|_, values| {
                        match is_on(values, "close_on_change") {
                            true => vec![],
                            false => vec!["close_on_change: false".to_string()],
                        }
                    }),
                    Control::switch("placeholder").code(|_, values| {
                        match is_on(values, "placeholder") {
                            true => vec![r#"placeholder: "Pick a day""#.to_string()],
                            false => vec![],
                        }
                    }),
                    Control::switch("label").default("true").code(|_, values| {
                        match is_on(values, "label") {
                            true => vec![r#"label: "Arrival""#.to_string()],
                            false => vec![],
                        }
                    }),
                    Control::switch("description").code(|_, values| {
                        match is_on(values, "description") {
                            true => vec![r#"description: "The day you check in.""#.to_string()],
                            false => vec![],
                        }
                    }),
                    Control::switch("helper").code(|_, values| match is_on(values, "helper") {
                        true => vec![r#"helper: "Try typing 3/10 or 3 oct.""#.to_string()],
                        false => vec![],
                    }),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    DateFieldDemo { values }
                },
            }
        }
    }
}

/// Its own component so the day survives a control change.
#[component]
fn DateFieldDemo(values: DemoValues) -> Element {
    let mut day = use_signal(|| NaiveDate::from_ymd_opt(2026, 9, 14));
    let limits = is_on(&values, "limits");

    rsx! {
        Flex { direction: "column", gap: "sm", sx: libero::sx::sx().width("100%").max_width("320px"),
            DateField {
                value: day(),
                onchange: move |next| day.set(next),
                size: values.str("size"),
                radius: values.str("radius"),
                format: values.str("format"),
                min: limits.then(|| NaiveDate::from_ymd_opt(2026, 9, 5)).flatten(),
                max: limits.then(|| NaiveDate::from_ymd_opt(2026, 10, 20)).flatten(),
                exclude_date: is_on(&values, "exclude_weekends").then(|| Callback::new(is_weekend)),
                close_on_change: is_on(&values, "close_on_change"),
                placeholder: is_on(&values, "placeholder").then(|| "Pick a day".to_string()),
                label: is_on(&values, "label").then(|| "Arrival".to_string()),
                description: is_on(&values, "description")
                    .then(|| "The day you check in.".to_string()),
                helper: is_on(&values, "helper").then(|| "Try typing 3/10 or 3 oct.".to_string()),
                status: match values.str("status").as_str() {
                    "warning" => FieldStatus::Warning("That is a Sunday.".to_string()),
                    "error" => FieldStatus::Error("Pick an arrival day.".to_string()),
                    _ => FieldStatus::Valid,
                },
                required: is_on(&values, "required").then_some(true),
                disabled: is_on(&values, "disabled").then_some(true),
            }
            Text { size: "sm", {shown(day())} }
        }
    }
}
