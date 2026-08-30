use super::common::{SIZES, field_controls, is_on, shown, status_of, text_of};
use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use libero::components::{
    Code, DateRange, DateRangeField, DateTimeField, DateTimeRangeField, Flex, Text, TimeField,
};

/// The props every date and time field shares, after its own.
fn shared_props(own: Vec<crate::components::PropDoc>) -> Vec<crate::components::PropDoc> {
    let mut all = own;
    all.extend([
        prop("placeholder", "String").doc("Shown while the text is empty."),
        prop("size", "Size").default("md").doc("Control height, font size and the dropdown's pickers."),
        prop("radius", "Size").default("sm").doc("Corner radius of the frame."),
        prop("label", "Caption").doc("The field's caption."),
        prop("description", "Caption").doc("Between the label and the control."),
        prop("helper", "Caption").doc("Under the control."),
        prop("status", "FieldStatus").default("Valid").doc("Validation state. Text the field cannot accept shows `DateDefaults::invalid_date` instead."),
        prop("required", "bool").default("false").doc("Adds `required` to the input and an asterisk to the label."),
        prop("disabled", "bool").default("false").doc("Disables typing and the dropdown."),
    ]);
    all
}

fn time_controls() -> Vec<Control> {
    vec![
        Control::toggle("variant", ["digital", "analog"]).default("digital"),
        Control::switch("with_seconds"),
        Control::switch("twelve_hour"),
    ]
}

fn page_controls(own: Vec<Control>) -> Vec<Control> {
    let mut controls = vec![
        Control::slider("size", SIZES).default("md"),
        Control::slider("radius", SIZES).default("sm"),
    ];
    controls.extend(own);
    controls.extend(field_controls());
    controls
}

const FIELD_WIDTH: &str = "360px";

/// A September 2026 moment for the demos.
fn moment(day: u32, hour: u32, minute: u32) -> Option<NaiveDateTime> {
    NaiveDate::from_ymd_opt(2026, 9, day)
        .zip(NaiveTime::from_hms_opt(hour, minute, 0))
        .map(|(day, time)| NaiveDateTime::new(day, time))
}

#[component]
pub fn TimeFieldPage() -> Element {
    rsx! {
        DocPage {
            title: "TimeField",
            source: "libero/src/components/form/date/fields.rs",
            markdown: "/md/time_field.md",
            properties: vec![
                props("TimeField", shared_props(vec![
                    prop("value", "Option<NaiveTime>").doc("The time in the field; strictly controlled."),
                    prop("onchange", "EventHandler<Option<NaiveTime>>").doc("On commit - blur or Enter - and on every picked part."),
                    prop("format", "String").doc("How the text shows the time. Defaults to `DateDefaults::time_format`, adjusted for `with_seconds` and `twelve_hour`."),
                    prop("variant", "TimePickerVariant").default("digital").doc("The dropdown's picker."),
                    prop("with_seconds", "bool").default("false").doc("Seconds in the text and the picker."),
                    prop("step", "u8").default("1").doc("Minutes between the offered minutes."),
                    prop("twelve_hour", "bool").doc("A 12-hour clock."),
                    prop("min", "NaiveTime").doc("The earliest time that can be picked or typed."),
                    prop("max", "NaiveTime").doc("The latest time that can be picked or typed."),
                    prop("name", "FieldName<Option<NaiveTime>>").doc("Posts `HH:MM:SS`."),
                ])),
            ],
            lead: rsx! {
                Text {
                    "A text field holding a "
                    Code { source: "NaiveTime" }
                    ", with a "
                    Code { source: "TimePicker" }
                    " in a dropdown. Typing reads 13:05, 1:05 pm, 1305 or 9 - hours, minutes and seconds in that order, any separator."
                }
            },
            Demo {
                component: "TimeField",
                children_text: "",
                controls: page_controls(time_controls()),
                render: move |values: DemoValues| rsx! {
                    TimeFieldDemo { values }
                },
            }
        }
    }
}

#[component]
fn TimeFieldDemo(values: DemoValues) -> Element {
    let mut time = use_signal(|| NaiveTime::from_hms_opt(9, 30, 0));
    rsx! {
        Flex { direction: "column", gap: "sm", sx: libero::sx::sx().width("100%").max_width(FIELD_WIDTH),
            TimeField {
                value: time(),
                onchange: move |next| time.set(next),
                size: values.str("size"),
                radius: values.str("radius"),
                variant: values.str("variant"),
                with_seconds: is_on(&values, "with_seconds"),
                twelve_hour: is_on(&values, "twelve_hour").then_some(true),
                placeholder: text_of(&values, "placeholder", "Pick one"),
                label: text_of(&values, "label", "When"),
                description: text_of(&values, "description", "Your local time."),
                helper: text_of(&values, "helper", "Typing works too."),
                status: status_of(&values),
                required: is_on(&values, "required").then_some(true),
                disabled: is_on(&values, "disabled").then_some(true),
            }
            Text { size: "sm", {shown(time())} }
        }
    }
}

#[component]
pub fn DateTimeFieldPage() -> Element {
    rsx! {
        DocPage {
            title: "DateTimeField",
            source: "libero/src/components/form/date/fields.rs",
            markdown: "/md/date_time_field.md",
            properties: vec![
                props("DateTimeField", shared_props(vec![
                    prop("value", "Option<NaiveDateTime>").doc("The day and time in the field; strictly controlled."),
                    prop("onchange", "EventHandler<Option<NaiveDateTime>>").doc("On commit and on every pick."),
                    prop("format", "String").default("DateDefaults::format").doc("How the text shows the day."),
                    prop("time_format", "String").default("DateDefaults::time_format").doc("How the text shows the time, after the day and a space."),
                    prop("min", "NaiveDateTime").doc("The earliest moment that can be picked or typed."),
                    prop("max", "NaiveDateTime").doc("The latest moment that can be picked or typed."),
                    prop("exclude_date", "Callback<NaiveDate, bool>").doc("Days that cannot be picked or typed."),
                    prop("variant", "TimePickerVariant").default("digital").doc("The dropdown's clock."),
                    prop("with_seconds", "bool").default("false").doc("Seconds in the text and the clock."),
                    prop("step", "u8").default("1").doc("Minutes between the offered minutes."),
                    prop("twelve_hour", "bool").doc("A 12-hour clock."),
                    prop("today", "NaiveDate").doc("The day marked as today, and the year a yearless text takes."),
                    prop("name", "FieldName<Option<NaiveDateTime>>").doc("Posts `2026-09-14T13:05:00`."),
                ])),
            ],
            lead: rsx! {
                Text {
                    "A text field holding a "
                    Code { source: "NaiveDateTime" }
                    ". The dropdown picks the day, then moves on to the time; its segmented control goes back. "
                    "Typing reads the day as a DateField does, and the time from the number before the first colon."
                }
            },
            Demo {
                component: "DateTimeField",
                children_text: "",
                controls: page_controls(time_controls()),
                render: move |values: DemoValues| rsx! {
                    DateTimeFieldDemo { values }
                },
            }
        }
    }
}

#[component]
fn DateTimeFieldDemo(values: DemoValues) -> Element {
    let mut value = use_signal(|| moment(14, 9, 30));
    rsx! {
        Flex { direction: "column", gap: "sm", sx: libero::sx::sx().width("100%").max_width(FIELD_WIDTH),
            DateTimeField {
                value: value(),
                onchange: move |next| value.set(next),
                size: values.str("size"),
                radius: values.str("radius"),
                variant: values.str("variant"),
                with_seconds: is_on(&values, "with_seconds"),
                twelve_hour: is_on(&values, "twelve_hour").then_some(true),
                placeholder: text_of(&values, "placeholder", "Pick one"),
                label: text_of(&values, "label", "When"),
                description: text_of(&values, "description", "Your local time."),
                helper: text_of(&values, "helper", "Typing works too."),
                status: status_of(&values),
                required: is_on(&values, "required").then_some(true),
                disabled: is_on(&values, "disabled").then_some(true),
            }
            Text { size: "sm", {shown(value())} }
        }
    }
}

#[component]
pub fn DateRangeFieldPage() -> Element {
    rsx! {
        DocPage {
            title: "DateRangeField",
            source: "libero/src/components/form/date/fields.rs",
            markdown: "/md/date_range_field.md",
            properties: vec![
                props("DateRangeField", shared_props(vec![
                    prop("value", "Option<DateRange<NaiveDate>>").doc("The range; strictly controlled. An `end` of `None` is still being picked."),
                    prop("onchange", "EventHandler<Option<DateRange<NaiveDate>>>").doc("On commit and on every pick."),
                    prop("format", "String").default("DateDefaults::format").doc("How the text shows each day; `DateDefaults::range_separator` joins them. Typing takes `–`, ` - ` or ` to `."),
                    prop("columns", "usize").default("2").doc("Months side by side in the dropdown."),
                    prop("min", "NaiveDate").doc("The earliest day."),
                    prop("max", "NaiveDate").doc("The latest day."),
                    prop("exclude_date", "Callback<NaiveDate, bool>").doc("Days that cannot be picked or typed."),
                    prop("close_on_change", "bool").default("true").doc("Picking the end closes the dropdown."),
                    prop("today", "NaiveDate").doc("The day marked as today."),
                    prop("name", "FieldName<Option<DateRange<NaiveDate>>>").doc("Posts an ISO 8601 interval."),
                ])),
            ],
            lead: rsx! {
                Text {
                    "A text field holding a "
                    Code { source: "DateRange<NaiveDate>" }
                    ", with two months in a dropdown. The first pick starts the range, the second ends it, and a hovered day previews the end."
                }
            },
            Demo {
                component: "DateRangeField",
                children_text: "",
                controls: page_controls(vec![
                    Control::slider("columns", ["1", "2"]).default("2").code(|_, values| {
                        match values.str("columns").as_str() {
                            "2" => vec![],
                            columns => vec![format!("columns: {columns}usize")],
                        }
                    }),
                ]),
                render: move |values: DemoValues| rsx! {
                    DateRangeFieldDemo { values }
                },
            }
        }
    }
}

#[component]
fn DateRangeFieldDemo(values: DemoValues) -> Element {
    let mut range = use_signal(|| {
        NaiveDate::from_ymd_opt(2026, 9, 14)
            .map(|start| DateRange::new(start, NaiveDate::from_ymd_opt(2026, 9, 18)))
    });
    rsx! {
        Flex { direction: "column", gap: "sm", sx: libero::sx::sx().width("100%").max_width(FIELD_WIDTH),
            DateRangeField {
                value: range(),
                onchange: move |next| range.set(next),
                size: values.str("size"),
                radius: values.str("radius"),
                columns: values.str("columns").parse::<usize>().ok(),
                placeholder: text_of(&values, "placeholder", "Pick one"),
                label: text_of(&values, "label", "When"),
                description: text_of(&values, "description", "Your local time."),
                helper: text_of(&values, "helper", "Typing works too."),
                status: status_of(&values),
                required: is_on(&values, "required").then_some(true),
                disabled: is_on(&values, "disabled").then_some(true),
            }
            Text { size: "sm", {shown(range())} }
        }
    }
}

#[component]
pub fn DateTimeRangeFieldPage() -> Element {
    rsx! {
        DocPage {
            title: "DateTimeRangeField",
            source: "libero/src/components/form/date/fields.rs",
            markdown: "/md/date_time_range_field.md",
            properties: vec![
                props("DateTimeRangeField", shared_props(vec![
                    prop("value", "Option<DateRange<NaiveDateTime>>").doc("The range; strictly controlled."),
                    prop("onchange", "EventHandler<Option<DateRange<NaiveDateTime>>>").doc("On commit and on every pick."),
                    prop("format", "String").default("DateDefaults::format").doc("How the text shows each day."),
                    prop("time_format", "String").default("DateDefaults::time_format").doc("How the text shows each time."),
                    prop("min", "NaiveDateTime").doc("The earliest moment."),
                    prop("max", "NaiveDateTime").doc("The latest moment."),
                    prop("exclude_date", "Callback<NaiveDate, bool>").doc("Days that cannot be picked or typed."),
                    prop("variant", "TimePickerVariant").default("digital").doc("The dropdown's clock."),
                    prop("with_seconds", "bool").default("false").doc("Seconds in the text and the clock."),
                    prop("step", "u8").default("1").doc("Minutes between the offered minutes."),
                    prop("twelve_hour", "bool").doc("A 12-hour clock."),
                    prop("today", "NaiveDate").doc("The day marked as today."),
                    prop("name", "FieldName<Option<DateRange<NaiveDateTime>>>").doc("Posts an ISO 8601 interval of two date-times."),
                ])),
            ],
            lead: rsx! {
                Text {
                    "A text field holding a "
                    Code { source: "DateRange<NaiveDateTime>" }
                    ". The dropdown picks the start - a day, then a time - before the end. "
                    "One segmented control switches between start and end, the other between day and time; "
                    "the end cannot be picked before the start's day, and an end that lands earlier swaps in."
                }
            },
            Demo {
                component: "DateTimeRangeField",
                children_text: "",
                controls: page_controls(time_controls()),
                render: move |values: DemoValues| rsx! {
                    DateTimeRangeFieldDemo { values }
                },
            }
        }
    }
}

#[component]
fn DateTimeRangeFieldDemo(values: DemoValues) -> Element {
    let mut range =
        use_signal(|| moment(14, 9, 0).map(|start| DateRange::new(start, moment(16, 17, 0))));
    rsx! {
        Flex { direction: "column", gap: "sm", sx: libero::sx::sx().width("100%").max_width("440px"),
            DateTimeRangeField {
                value: range(),
                onchange: move |next| range.set(next),
                size: values.str("size"),
                radius: values.str("radius"),
                variant: values.str("variant"),
                with_seconds: is_on(&values, "with_seconds"),
                twelve_hour: is_on(&values, "twelve_hour").then_some(true),
                placeholder: text_of(&values, "placeholder", "Pick one"),
                label: text_of(&values, "label", "When"),
                description: text_of(&values, "description", "Your local time."),
                helper: text_of(&values, "helper", "Typing works too."),
                status: status_of(&values),
                required: is_on(&values, "required").then_some(true),
                disabled: is_on(&values, "disabled").then_some(true),
            }
            Text { size: "sm", {shown(range())} }
        }
    }
}
