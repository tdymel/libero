use super::common::{SIZES, is_on, is_weekend, shown};
use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::chrono::NaiveDate;
use libero::components::{Code, DatePicker, Flex, Text};

#[component]
pub fn DatePickerPage() -> Element {
    rsx! {
        DocPage {
            title: "DatePicker",
            source: "libero/src/components/form/date/pickers.rs",
            markdown: "/md/date_picker.md",
            properties: vec![
                props("DatePicker", vec![
                    prop("value", "Option<NaiveDate>").doc("The picked day; strictly controlled. `None` picks nothing."),
                    prop("onchange", "EventHandler<Option<NaiveDate>>").doc("Called with the day the caller should hold next."),
                    prop("min", "NaiveDate").doc("The earliest day that can be picked."),
                    prop("max", "NaiveDate").doc("The latest day that can be picked."),
                    prop("exclude_date", "Callback<NaiveDate, bool>")
                        .doc("Days that cannot be picked, on top of `min` and `max`. A `Callback` always compares equal, so changing only this closure does not redraw the picker."),
                    prop("allow_deselect", "bool").default("false").doc("Clicking the picked day again clears it."),
                    prop("today", "NaiveDate").doc("The day marked as today. Unset, the platform clock answers after mount - on the web; elsewhere no day is marked."),
                    prop("size", "Size").default("md").doc("Day cell size and font size."),
                    prop("name", "String").doc("Emits a hidden input of that name, posting the day as ISO 8601."),
                    prop("focusable", "bool").default("true").doc("`false` keeps the days and buttons out of the tab order, for a picker inside a dropdown whose input keeps focus."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A month of days to pick one from. Controlled through "
                    Code { source: "value" }
                    " and "
                    Code { source: "onchange" }
                    ", and the value is "
                    Code { source: "chrono::NaiveDate" }
                    " - a calendar day with no time zone. libero re-exports the crate as "
                    Code { source: "libero::chrono" }
                    "."
                }
                Text {
                    "The month shown is the picker's own state: it opens on the value's month, else today's. "
                    "Names, the first weekday and the heading format come from the theme's "
                    Code { source: "DateDefaults" }
                    ". The grid is one tab stop; arrows move a day or a week, Home and End to the week's ends, "
                    "Page Up and Page Down a month, with Shift a year."
                }
            },
            Demo {
                component: "DatePicker",
                children_text: "",
                controls: vec![
                    Control::slider("size", SIZES).default("md"),
                    Control::switch("allow_deselect"),
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
                ],
                render: move |values: DemoValues| rsx! {
                    DatePickerDemo { values }
                },
            }
        }
    }
}

/// Its own component so the day survives a control change.
#[component]
fn DatePickerDemo(values: DemoValues) -> Element {
    let mut day = use_signal(|| NaiveDate::from_ymd_opt(2026, 9, 14));
    let limits = is_on(&values, "limits");

    rsx! {
        Flex { direction: "column", gap: "sm", align: "flex-start",
            DatePicker {
                value: day(),
                onchange: move |next| day.set(next),
                size: values.str("size"),
                allow_deselect: is_on(&values, "allow_deselect"),
                min: limits.then(|| NaiveDate::from_ymd_opt(2026, 9, 5)).flatten(),
                max: limits.then(|| NaiveDate::from_ymd_opt(2026, 10, 20)).flatten(),
                exclude_date: is_on(&values, "exclude_weekends").then(|| Callback::new(is_weekend)),
            }
            Text { size: "sm", {shown(day())} }
        }
    }
}
