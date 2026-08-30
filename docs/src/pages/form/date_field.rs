use super::date_common::{
    SIZES, day_limits, field_controls, has_days, has_time, is_on, is_weekend, moment_limits,
    shared_controls, shown, status_of, step_of, switches_last, text_of, time_limits, today_of,
    twelve_hour_of,
};
use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use libero::components::{
    Code, CodeBlock, DateField, DateRange, Flex, Kbd, List, ListItem, Rule, Text, Validators,
    not_empty,
};

const KINDS: [&str; 5] = ["date", "time", "date-time", "date-range", "date-time-range"];

const TIME_FORMATS: [&str; 4] = ["default", "HH:mm", "h:mm A", "HH:mm:ss"];

fn rules<V: 'static>(on: bool) -> Validators<Option<V>> {
    match on {
        true => not_empty::<Option<V>>.error("Pick a value.").into(),
        false => Validators::default(),
    }
}

const FORMATS: [&str; 5] = [
    "MMMM D, YYYY",
    "DD.MM.YYYY",
    "MM/DD/YYYY",
    "YYYY-MM-DD",
    "ddd, D MMM YYYY",
];

const NAMING_V: &str = r#"// A typed signal names `V` through the handler.
let mut day = use_signal(|| None::<NaiveDate>);
DateField { value: day(), onchange: move |next| day.set(next) }

// Without a typed handler, a turbofish does.
DateField::<NaiveTime> { name: "alarm" }"#;

fn day(day: u32) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(2026, 9, day)
}

fn moment(day: u32, hour: u32) -> Option<NaiveDateTime> {
    NaiveDate::from_ymd_opt(2026, 9, day)
        .zip(NaiveTime::from_hms_opt(hour, 0, 0))
        .map(|(day, time)| NaiveDateTime::new(day, time))
}

#[component]
pub fn DateFieldPage() -> Element {
    rsx! {
        DocPage {
            title: "DateField",
            source: "libero/src/components/form/date/date_field.rs",
            markdown: "/md/date_field.md",
            properties: vec![
                props("DateField<V: DateValue>", vec![
                    prop("value", "Option<V>").doc("The value; strictly controlled. `None` is the empty field. Its type picks the dropdown."),
                    prop("onchange", "EventHandler<Option<V>>")
                        .doc("Called when typed text is committed - on blur or Enter - and on every pick. Emptied text commits `None`."),
                    prop("format", "String")
                        .default("DateDefaults::format")
                        .doc("How the text shows a day, in dayjs tokens. Typing is lenient either way: only the order of day, month and year has to match."),
                    prop("time_format", "String").default("DateDefaults::time_format").doc("How the text shows a time."),
                    prop("min", "V::Bound").doc("The earliest value that can be picked or typed. For a range, the earliest end: a `NaiveDate` or `NaiveDateTime`."),
                    prop("max", "V::Bound").doc("The latest value, likewise."),
                    prop("exclude_date", "Callback<NaiveDate, bool>").doc("Days that cannot be picked or typed. Ignored for a time."),
                    prop("today", "NaiveDate").doc("The day marked as today, and the year typed text without one falls back to. Unset, the platform clock answers after mount."),
                    prop("variant", "TimePickerVariant").default("analog").doc("The clock, for values with a time."),
                    prop("with_seconds", "bool").default("false").doc("Seconds in the text and the clock, for values with a time."),
                    prop("step", "u8").default("1").doc("Minutes between the offered minutes."),
                    prop("twelve_hour", "bool").doc("A 12-hour clock. Defaults to whether the time format is one."),
                    prop("columns", "usize").default("2").doc("Months side by side, for a range of days."),
                    prop("close_on_change", "bool").default("true").doc("Picking a day, or a range's end, closes the dropdown."),
                    prop("name", "FieldName<Option<V>>").doc("What the field posts as - the value in ISO 8601, whatever the text shows. A path also binds it to the surrounding `Form`."),
                    prop("validate", "Validators<Option<V>>").doc("Rules over the value."),
                    prop("placeholder", "String").doc("Shown while the text is empty."),
                    prop("size", "Size").default("md").doc("Control height, font size and the dropdown's picker."),
                    prop("radius", "Size").default("sm").doc("Corner radius of the frame."),
                    prop("label", "Caption").doc("The field's caption."),
                    prop("description", "Caption").doc("Between the label and the control."),
                    prop("helper", "Caption").doc("Under the control."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state. Text the field cannot accept shows `DateDefaults::invalid_date` instead."),
                    prop("required", "bool").default("false").doc("Adds `required` to the input and an asterisk to the label."),
                    prop("disabled", "bool").default("false").doc("Disables typing and the dropdown, and dims the field."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A text field for every date and time value, with a "
                    Code { source: "DatePicker" }
                    " in a dropdown. The value's type picks what the dropdown shows: "
                    Code { source: "NaiveDate" }
                    " a calendar, "
                    Code { source: "NaiveTime" }
                    " a clock, "
                    Code { source: "NaiveDateTime" }
                    " both, and a "
                    Code { source: "DateRange" }
                    " of either picks two. The types are "
                    Code { source: "chrono" }
                    "'s, re-exported as "
                    Code { source: "libero::chrono" }
                    "."
                }
                Text {
                    "Typed text stays as typed until the field blurs or Enter is pressed, then it is read leniently: "
                    "any separator, one-digit days and months, month names in any case or as a unique prefix, "
                    "and a missing year taken from today. Only the order of day, month and year follows "
                    Code { source: "format" }
                    ". Two-digit years are rejected. Text that is not an accepted value stays, and the field shows an error. "
                    "The form gets ISO 8601, whatever the text shows."
                }
            },
            Demo {
                component: "DateField",
                children_text: "",
                controls: {
                    let controls = vec![
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
                        Control::slider("radius", SIZES).default("sm"),
                        Control::select("format", FORMATS)
                            .default("MMMM D, YYYY")
                            .hidden_when(|values| values.str("value") == "time")
                            .code(|_, values| match values.str("format").as_str() {
                                "MMMM D, YYYY" => vec![],
                                format => vec![format!("format: {format:?}")],
                            }),
                        // `default` leaves it unset: the theme's, adjusted for
                        // `with_seconds` and `twelve_hour`.
                        Control::select("time_format", TIME_FORMATS)
                            .default("default")
                            .hidden_when(|values| !has_time(values))
                            .code(|_, values| match values.str("time_format").as_str() {
                                "default" => vec![],
                                format => vec![format!("time_format: {format:?}")],
                            }),
                        Control::toggle("variant", ["analog", "digital"]).default("analog").hidden_when(|values| !has_time(values)).code(|_, values| {
                            match values.str("variant").as_str() {
                                "digital" => vec![r#"variant: "digital""#.to_string()],
                                _ => vec![],
                            }
                        }),
                        Control::toggle("columns", ["1", "2", "3"])
                            .default("2")
                            .hidden_when(|values| values.str("value") != "date-range")
                            .code(|_, values| match values.str("columns").as_str() {
                                "2" => vec![],
                                columns => vec![format!("columns: {columns}")],
                            }),
                        Control::switch("exclude_weekends").hidden_when(|values| !has_days(values)).code(|_, values| {
                            match is_on(values, "exclude_weekends") {
                                true => vec!["exclude_date: |day: NaiveDate| day.weekday().num_days_from_monday() >= 5".to_string()],
                                false => vec![],
                            }
                        }),
                        Control::switch("close_on_change").default("true").hidden_when(|values| values.str("value") == "time").code(|_, values| {
                            match is_on(values, "close_on_change") {
                                true => vec![],
                                false => vec!["close_on_change: false".to_string()],
                            }
                        }),
                        Control::switch("validate").code(|_, values| match is_on(values, "validate") {
                            true => vec![r#"validate: not_empty.error("Pick a value.")"#.to_string()],
                            false => vec![],
                        }),
                    ];
                    switches_last([controls, shared_controls(), field_controls()].concat())
                },
                render: move |values: DemoValues| rsx! {
                    DateFieldDemo { values }
                },
            }
            DocSection {
                title: "Keyboard",
                Text {
                    "Focus opens the dropdown and stays in the text, so typing works at once. "
                    Kbd { "↓" } " moves focus into the picker, onto the picked day or the clock, where the "
                    Code { source: "DatePicker" }
                    " keys apply. "
                    Kbd { "Escape" } " goes back to the text, and so does a pick that closes the dropdown. "
                    "Focus leaving both the text and the dropdown closes it. A mouse click in the dropdown leaves focus in the text."
                }
            }
            DocSection {
                title: "Naming the value type",
                Text {
                    "A typed "
                    Code { source: "value" }
                    " alone does not tell the compiler which "
                    Code { source: "V" }
                    " the field holds: dioxus converts every prop, so "
                    Code { source: "Some(day)" }
                    " could become more than one type. A handler that stores into a typed signal names it, and so does a turbofish."
                }
                CodeBlock { source: NAMING_V, language: "rust" }
            }
            DocSection {
                title: "Alternatives",
                Text {
                    "The same field for one value type each, with only the props that type uses. "
                    "They need no turbofish, and a value of the wrong type is a plain type mismatch."
                }
                List {
                    ListItem { Code { source: "DayField" } " - a " Code { source: "NaiveDate" } ", with " Code { source: "DayPicker" } " in the dropdown." }
                    ListItem { Code { source: "TimeField" } " - a " Code { source: "NaiveTime" } ", with " Code { source: "TimePicker" } "." }
                    ListItem { Code { source: "DateTimeField" } " - a " Code { source: "NaiveDateTime" } ": the day, then the time." }
                    ListItem { Code { source: "DateRangeField" } " - a " Code { source: "DateRange<NaiveDate>" } ", with " Code { source: "DateRangePicker" } "." }
                    ListItem { Code { source: "DateTimeRangeField" } " - a " Code { source: "DateRange<NaiveDateTime>" } ": the start, then the end." }
                }
            }
        }
    }
}

/// One signal per value type, so switching types keeps what was picked.
#[component]
fn DateFieldDemo(values: DemoValues) -> Element {
    let mut date = use_signal(|| day(14));
    let mut time = use_signal(|| NaiveTime::from_hms_opt(9, 30, 0));
    let mut date_time = use_signal(|| moment(14, 9));
    let mut date_range = use_signal(|| day(14).map(|start| DateRange::new(start, day(18))));
    let mut date_time_range =
        use_signal(|| moment(14, 9).map(|start| DateRange::new(start, moment(16, 17))));

    let size = values.str("size");
    let radius = values.str("radius");
    let format = values.str("format");
    let variant = values.str("variant");
    let exclude_date = is_on(&values, "exclude_weekends").then(|| Callback::new(is_weekend));
    let close_on_change = is_on(&values, "close_on_change");
    let label = text_of(&values, "label", "When");
    let description = text_of(&values, "description", "Your local time.");
    let helper = text_of(&values, "helper", "Typing works too.");
    let placeholder = text_of(&values, "placeholder", "Pick one");
    let status = status_of(&values);
    let required = is_on(&values, "required").then_some(true);
    let disabled = is_on(&values, "disabled").then_some(true);
    let time_format = Some(values.str("time_format")).filter(|format| format != "default");
    let columns = values.str("columns").parse::<usize>().ok();
    let step = step_of(&values);
    let with_seconds = is_on(&values, "with_seconds");
    let twelve_hour = twelve_hour_of(&values);
    let today = today_of(&values);
    let validate = is_on(&values, "validate");
    let (min_day, max_day) = day_limits(&values);
    let (min_time, max_time) = time_limits(&values);
    let (min_moment, max_moment) = moment_limits(&values);

    let (field, readout) = match values.str("value").as_str() {
        "time" => (
            rsx! {
                DateField {
                    value: time(), onchange: move |next| time.set(next),
                    min: min_time, max: max_time, time_format, variant, with_seconds, step, twelve_hour,
                    validate: rules(validate),
                    size, radius, label, description, helper, placeholder, status, required, disabled,
                }
            },
            shown(time()),
        ),
        "date-time" => (
            rsx! {
                DateField {
                    value: date_time(), onchange: move |next| date_time.set(next),
                    min: min_moment, max: max_moment, format, time_format, variant, exclude_date, today,
                    with_seconds, step, twelve_hour, close_on_change,
                    validate: rules(validate),
                    size, radius, label, description, helper, placeholder, status, required, disabled,
                }
            },
            shown(date_time()),
        ),
        "date-range" => (
            rsx! {
                DateField {
                    value: date_range(), onchange: move |next| date_range.set(next),
                    min: min_day, max: max_day, format, exclude_date, columns, today, close_on_change,
                    validate: rules(validate),
                    size, radius, label, description, helper, placeholder, status, required, disabled,
                }
            },
            shown(date_range()),
        ),
        "date-time-range" => (
            rsx! {
                DateField {
                    value: date_time_range(), onchange: move |next| date_time_range.set(next),
                    min: min_moment, max: max_moment, format, time_format, variant, exclude_date, today,
                    with_seconds, step, twelve_hour, close_on_change,
                    validate: rules(validate),
                    size, radius, label, description, helper, placeholder, status, required, disabled,
                }
            },
            shown(date_time_range()),
        ),
        _ => (
            rsx! {
                DateField {
                    value: date(), onchange: move |next| date.set(next),
                    min: min_day, max: max_day, format, exclude_date, today, close_on_change,
                    validate: rules(validate),
                    size, radius, label, description, helper, placeholder, status, required, disabled,
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
