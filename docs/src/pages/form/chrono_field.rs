use super::date_common::{
    SIZES, calendar_controls, captions_of, day_limits, duration_limits, field_controls, has_days,
    has_time, is_on, is_weekend, moment_limits, shared_controls, shown, status_of, step_of,
    text_of, time_limits, today_of, twelve_hour_of,
};
use super::dropdown_parts::chrono_dropdown_parts;
use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap, a11y, prop, props};
use dioxus::prelude::*;
use libero::chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeDelta};
use libero::components::FieldPart;
use libero::components::{
    ChronoField, Code, DateLevel, DateRange, Flex, Rule, Text, Validators, not_empty,
};
use libero::{use_formats_handle, use_localization_handle};

use super::date_locales::{FORMATS, LANGUAGES, options, picked};

const KINDS: [&str; 8] = [
    "date",
    "month",
    "year",
    "time",
    "date-time",
    "date-range",
    "date-time-range",
    "duration",
];

/// The root's props for the picked language and formats, unless they are the site's.
fn root_of(values: &DemoValues) -> Option<String> {
    let (language, _) = picked(&LANGUAGES, &values.str("language"));
    let (formats, _) = picked(&FORMATS, &values.str("formats"));
    (language != LANGUAGES[0].1 || formats != FORMATS[1].1)
        .then(|| format!("localization: &Localization::{language}, formats: &Formats::{formats}"))
}

const TIME_FORMATS: [&str; 4] = ["default", "HH:mm", "h:mm A", "HH:mm:ss"];

fn rules<V: 'static>(on: bool) -> Validators<Option<V>> {
    match on {
        true => not_empty::<Option<V>>.error("Pick a value.").into(),
        false => Validators::default(),
    }
}

const DATE_FORMATS: [&str; 5] = [
    "default",
    "DD.MM.YYYY",
    "MM/DD/YYYY",
    "YYYY-MM-DD",
    "ddd, D MMM YYYY",
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
pub fn ChronoFieldPage() -> Element {
    rsx! {
        DocPage {
            title: "ChronoField",
            source: "libero/src/components/form/date/chrono_field.rs",
            markdown: "/md/chrono_field.md",
            properties: vec![
                props("ChronoField<V: DateValue>", vec![
                    prop("size", "Size").default("md").doc("Control height, font size and the dropdown's picker."),
                    prop("radius", "Size").default("sm").doc("Corner radius of the frame."),
                    prop("value", "Option<V>").doc("The value. `None` is the empty field. Its type picks the dropdown. Pair it with `onchange`."),
                    prop("onchange", "EventHandler<Option<V>>")
                        .doc("Called on every pick, and when typed text is committed on blur or Enter. Emptied text commits `None`."),
                    prop("level", "DateLevel").default("Day").doc("Types and picks a `NaiveDate` as a day, a month (its first day) or a year (its January 1). Ignored for other values."),
                    prop("format", "String")
                        .default("(Formats::date)(level)")
                        .doc("How the text shows the value, in dayjs tokens. The default is `MMMM D, YYYY` in American formats, `D. MMMM YYYY` in German, and `MMMM YYYY` or `YYYY` in both for a month or a year. Typing only has to match the order of day, month and year."),
                    prop("time_format", "String").default("Formats::time").doc("How the text shows a time. `h:mm A` in American formats, `HH:mm` in German."),
                    prop("min", "V::Bound").doc("The earliest value accepted. For a range, the earliest end. A duration's is 0 when unset."),
                    prop("max", "V::Bound").doc("The latest value accepted. For a range, the latest end. A duration's is 99 h 59 min 59 s when unset."),
                    prop("exclude_date", "Callback<NaiveDate, bool>").doc("Days that are not accepted, on top of `min` and `max`. Ignored for a time, a month and a year."),
                    prop("today", "NaiveDate").doc("The day marked as today, and the year used when typed text has none. Unset, the platform clock answers after mount."),
                    prop("variant", "TimePickerVariant").default("analog").doc("A digital clock, `HH:MM` with a column to turn per part, or a clock face, for values with a time."),
                    prop("with_seconds", "bool").default("false").doc("Seconds in the text and on the clock, or a duration's seconds column."),
                    prop("step", "u8").default("5").doc("Minutes between the offered minutes, on a clock or a duration's minutes column."),
                    prop("twelve_hour", "bool").doc("A 12-hour clock with AM and PM. Defaults to whether the time format is one."),
                    prop("calendar", "CalendarVariant").default("full").doc("A month of days, or `mini`, one row of days with buttons that page it. For a day or a date-time."),
                    prop("days", "usize").default("7").doc("Days in the mini calendar's row."),
                    prop("columns", "usize").default("1, or 2 for a range").doc("Months side by side."),
                    prop("close_on_change", "bool").default("true").doc("Picking a day, or a range's second end, closes the dropdown."),
                    prop("name", "FieldName<Option<V>>").doc("What the field posts as, the value in ISO 8601 whatever the text shows. A path also binds it to the surrounding `Form`'s value when it has no `onchange`."),
                    prop("validate", "Validators<Option<V>>").doc("Rules over the value, shown once the field loses focus or its form is submitted."),
                    prop("placeholder", "String").doc("Shown while the text is empty."),
                    prop("label", "Caption").doc("The caption above the control, and the field's name."),
                    prop("description", "Caption").doc("Between the label and the control. What to enter."),
                    prop("helper", "Caption").doc("Under the control. Formatting rules, or what the entry changes."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, under the helper. Text the field cannot accept shows its own error instead, such as `DateLocale::invalid_date`, `invalid_duration` or the bound it missed."),
                    prop("required", "bool").default("false").doc("Sets `required` on the input and marks the label."),
                    prop("disabled", "bool").default("false").doc("Disables typing and the dropdown, and dims the field."),
                    prop("readonly", "bool").default("false").doc("Focusable and posted with the form, but not editable. `disabled` drops the field from the tab order and the post instead."),
                    prop("dropdown_parts", "Parts<ChronoDropdownPart>").doc("Styles the portaled dropdown and the picker in it."),
                ])
                .parts("FieldPart", vec![
                    (FieldPart::Label, "The label above the control."),
                    (FieldPart::Required, "The required asterisk, in the label."),
                    (FieldPart::Description, "The caption between the label and the control."),
                    (FieldPart::Frame, "The bordered box around the control."),
                    (FieldPart::Control, "The element the label names."),
                    (FieldPart::Trailing, "The slot after the control: a chevron, a toggle."),
                    (FieldPart::Helper, "The caption under the control."),
                    (FieldPart::Status, "The validation message."),
                ])
                .dropdown_parts("ChronoDropdownPart", chrono_dropdown_parts()),
            ],
            accessibility: a11y()
                .key(["Down"], "Moves focus into the picker, onto the picked day or the clock, where the `ChronoPicker` keys apply.")
                .key(["Escape"], "Moves focus back to the text.")
                .handles([
                    "Focus opens the dropdown and stays in the text, so you can type at once.",
                    "A pick that closes the dropdown moves focus back to the text.",
                    "Focus leaving both the text and the dropdown closes it.",
                    "A mouse click in the dropdown leaves focus in the text.",
                ]),
            lead: rsx! {
                Text {
                    "A text field for every date and time value, with a "
                    Code { source: "ChronoPicker" }
                    " in a dropdown. The value's type picks what the dropdown shows: "
                    Code { source: "NaiveDate" }
                    " a calendar, "
                    Code { source: "NaiveTime" }
                    " a clock, "
                    Code { source: "NaiveDateTime" }
                    " both, a "
                    Code { source: "DateRange" }
                    " of either picks two, and "
                    Code { source: "TimeDelta" }
                    " is a duration. "
                    Code { source: "level" }
                    " makes a "
                    Code { source: "NaiveDate" }
                    " field a month or a year field, typed as "
                    Code { source: "September 2026" }
                    " or "
                    Code { source: "2026" }
                    ". The types are "
                    Code { source: "chrono" }
                    "'s, re-exported as "
                    Code { source: "libero::chrono" }
                    "."
                }
                Text {
                    "Typed text is read on blur or Enter, and leniently. Any separator works, "
                    "as do one-digit days, month names as a unique prefix, and a missing year. "
                    "Two-digit years are not read. Only the order of day, month and year follows "
                    Code { source: "format" }
                    ". Text the field cannot accept stays, and the error says why. The form "
                    "always gets ISO 8601."
                }
                Text {
                    "The language of names, labels and errors comes from the provider's "
                    Code { source: "Localization" }
                    ". The patterns, first weekday and 12- or 24-hour clock come from its "
                    Code { source: "Formats" }
                    ", "
                    Code { source: "Formats::AMERICAN" }
                    " by default or "
                    Code { source: "Formats::GERMAN" }
                    ". This site uses English with German formats."
                }
                Text {
                    "A typed "
                    Code { source: "value" }
                    " alone does not name the type. A handler that stores into a typed signal "
                    "does, or a turbofish such as "
                    Code { source: "ChronoField::<NaiveTime> {{ .. }}" }
                    ". For one value type there are "
                    Code { source: "DateField" }
                    ", "
                    Code { source: "TimeField" }
                    ", "
                    Code { source: "DateTimeField" }
                    ", "
                    Code { source: "DateRangeField" }
                    " and "
                    Code { source: "DateTimeRangeField" }
                    ", with only the props that type uses and no turbofish."
                }
            },
            // snippet: item use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta};
            // snippet: let mut date = use_signal(|| None::<NaiveDate>);
            // snippet: let mut month = use_signal(|| None::<NaiveDate>);
            // snippet: let mut year = use_signal(|| None::<NaiveDate>);
            // snippet: let mut time = use_signal(|| None::<NaiveTime>);
            // snippet: let mut date_time = use_signal(|| None::<NaiveDateTime>);
            // snippet: let mut date_range = use_signal(|| None::<DateRange<NaiveDate>>);
            // snippet: let mut date_time_range = use_signal(|| None::<DateRange<NaiveDateTime>>);
            // snippet: let mut duration = use_signal(|| None::<TimeDelta>);
            Demo {
                component: "ChronoField",
                children_text: "",
                wrap: Wrap(|values: &DemoValues, source: &str| match root_of(values) {
                    Some(root) => format!("// Given to the root: `LiberoProvider {{ {root}, .. }}`.\n{source}"),
                    None => source.to_string(),
                }),
                controls: {
                    let controls = vec![
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
                                "duration" => ("duration", "Option<TimeDelta>", None),
                                _ => ("date", "Option<NaiveDate>", None),
                            };
                            let mut code = vec![
                                format!("value: {name}() /* {kind} */"),
                                format!("onchange: move |next| {name}.set(next)"),
                            ];
                            code.extend(level.map(|level| format!("level: DateLevel::{level}")));
                            code
                        }),
                        Control::slider("size", SIZES).default("md"),
                        Control::slider("radius", SIZES).default("sm"),
                        // Both swap the site's own, so they add no prop.
                        Control::toggle("language", options(&LANGUAGES)).default(LANGUAGES[0].0).code(|_, _| vec![]),
                        Control::toggle("formats", options(&FORMATS)).default(FORMATS[1].0).code(|_, _| vec![]),
                        // `default` leaves it unset: the formats' own.
                        Control::select("format", DATE_FORMATS)
                            .default("default")
                            .hidden_when(|values| !matches!(values.str("value").as_str(), "date" | "date-time" | "date-range" | "date-time-range"))
                            .code(|_, values| match values.str("format").as_str() {
                                "default" => vec![],
                                format => vec![format!("format: {format:?}")],
                            }),
                        // `default` leaves it unset: the formats' own, adjusted for
                        // `with_seconds` and `twelve_hour`.
                        Control::select("time_format", TIME_FORMATS)
                            .default("default")
                            .hidden_when(|values| !has_time(values))
                            .code(|_, values| match values.str("time_format").as_str() {
                                "default" => vec![],
                                format => vec![format!("time_format: {format:?}")],
                            }),
                        Control::toggle("variant", ["analog", "digital"]).labels(["Analog", "Digital"]).default("analog").hidden_when(|values| !has_time(values)).code(|_, values| {
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
                        Control::switch("close_on_change").default("true").hidden_when(|values| matches!(values.str("value").as_str(), "time" | "duration")).code(|_, values| {
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
                    [controls, calendar_controls(), shared_controls(), field_controls()].concat()
                },
                render: move |values: DemoValues| rsx! {
                    ChronoFieldDemo { values }
                },
            }
            DocSection {
                title: "Duration",
                Text {
                    "A "
                    Code { source: "TimeDelta" }
                    " field holds a span of time rather than a moment. It shows "
                    Code { source: "1 h 30 min" }
                    ", with the units from "
                    Code { source: "DateLocale" }
                    ", and posts ISO 8601, "
                    Code { source: "PT1H30M" }
                    ". Typing reads "
                    Code { source: "1 h 30 min" }
                    ", "
                    Code { source: "1h30" }
                    ", "
                    Code { source: "1:30" }
                    " or a bare number of minutes."
                }
                Text {
                    "The dropdown has a column each for the hours, the minutes at "
                    Code { source: "step" }
                    " and, with "
                    Code { source: "with_seconds" }
                    ", the seconds. The minutes and seconds wrap round without carrying into the next column. "
                    "The value runs from "
                    Code { source: "min" }
                    ", 0 by default, to "
                    Code { source: "max" }
                    ", 99 h 59 min 59 s by default. The hours column ends at "
                    Code { source: "max" }
                    "."
                }
                Text {
                    "A duration has its own errors: "
                    Code { source: "Must be at least 15 min" }
                    " names the bound it missed, and "
                    Code { source: "Not a valid duration" }
                    " is text it cannot read. A screen reader hears each column's value with its unit, "
                    Code { source: "2 hours" }
                    "."
                }
            }
        }
    }
}

/// One signal per value type, so switching types keeps what was picked.
#[component]
fn ChronoFieldDemo(values: DemoValues) -> Element {
    let mut date = use_signal(|| day(14));
    let mut month = use_signal(|| day(1));
    let mut year = use_signal(|| NaiveDate::from_ymd_opt(2026, 1, 1));
    let mut time = use_signal(|| NaiveTime::from_hms_opt(9, 30, 0));
    let mut date_time = use_signal(|| moment(14, 9));
    let mut date_range = use_signal(|| day(14).map(|start| DateRange::new(start, day(18))));
    let mut date_time_range =
        use_signal(|| moment(14, 9).map(|start| DateRange::new(start, moment(16, 17))));
    let mut duration = use_signal(|| TimeDelta::try_minutes(90));

    let size = values.str("size");
    let radius = values.str("radius");
    let (_, language) = picked(&LANGUAGES, &values.str("language"));
    let (_, conventions) = picked(&FORMATS, &values.str("formats"));
    let localization = use_localization_handle();
    let formats = use_formats_handle();
    let site = use_hook(|| (localization.get(), formats.get()));
    use_effect(use_reactive!(|language, conventions| {
        localization.set(language);
        formats.set(conventions);
    }));
    use_drop(move || {
        localization.set(site.0);
        formats.set(site.1);
    });
    let format = Some(values.str("format")).filter(|format| format != "default");
    let variant = values.str("variant");
    let exclude_date = is_on(&values, "exclude_weekends").then(|| Callback::new(is_weekend));
    let close_on_change = is_on(&values, "close_on_change");
    let (caption, about) = captions_of(&values);
    let label = text_of(&values, "label", caption);
    let aria_label = label.is_none().then_some(caption);
    let description = text_of(&values, "description", about);
    let helper = text_of(&values, "helper", "Typing works too.");
    let placeholder = text_of(&values, "placeholder", "Pick one");
    let status = status_of(&values);
    let required = is_on(&values, "required").then_some(true);
    let disabled = is_on(&values, "disabled").then_some(true);
    let time_format = Some(values.str("time_format")).filter(|format| format != "default");
    let columns = values.str("columns").parse::<usize>().ok();
    let calendar = values.str("calendar");
    let days = values.str("days").parse::<usize>().ok();
    let step = step_of(&values);
    let with_seconds = is_on(&values, "with_seconds");
    let twelve_hour = twelve_hour_of(&values);
    let today = today_of(&values);
    let validate = is_on(&values, "validate");
    let (min_day, max_day) = day_limits(&values);
    let (min_time, max_time) = time_limits(&values);
    let (min_moment, max_moment) = moment_limits(&values);
    let (min_duration, max_duration) = duration_limits(&values);

    let (field, readout) = match values.str("value").as_str() {
        "month" => (
            rsx! {
                ChronoField {
                    value: month(), onchange: move |next| month.set(next), level: DateLevel::Month,
                    min: min_day, max: max_day, today, close_on_change,
                    validate: rules(validate),
                    size, radius, label, aria_label, description, helper, placeholder, status, required, disabled,
                }
            },
            shown(month()),
        ),
        "year" => (
            rsx! {
                ChronoField {
                    value: year(), onchange: move |next| year.set(next), level: DateLevel::Year,
                    min: min_day, max: max_day, today, close_on_change,
                    validate: rules(validate),
                    size, radius, label, aria_label, description, helper, placeholder, status, required, disabled,
                }
            },
            shown(year()),
        ),
        "time" => (
            rsx! {
                ChronoField {
                    value: time(), onchange: move |next| time.set(next),
                    min: min_time, max: max_time, time_format, variant, with_seconds, step, twelve_hour,
                    validate: rules(validate),
                    size, radius, label, aria_label, description, helper, placeholder, status, required, disabled,
                }
            },
            shown(time()),
        ),
        "date-time" => (
            rsx! {
                ChronoField {
                    value: date_time(), onchange: move |next| date_time.set(next),
                    min: min_moment, max: max_moment, format, time_format, variant, exclude_date, today,
                    with_seconds, step, twelve_hour, close_on_change, calendar, days,
                    validate: rules(validate),
                    size, radius, label, aria_label, description, helper, placeholder, status, required, disabled,
                }
            },
            shown(date_time()),
        ),
        "date-range" => (
            rsx! {
                ChronoField {
                    value: date_range(), onchange: move |next| date_range.set(next),
                    min: min_day, max: max_day, format, exclude_date, columns, today, close_on_change,
                    validate: rules(validate),
                    size, radius, label, aria_label, description, helper, placeholder, status, required, disabled,
                }
            },
            shown(date_range()),
        ),
        "date-time-range" => (
            rsx! {
                ChronoField {
                    value: date_time_range(), onchange: move |next| date_time_range.set(next),
                    min: min_moment, max: max_moment, format, time_format, variant, exclude_date, today,
                    with_seconds, step, twelve_hour, close_on_change,
                    validate: rules(validate),
                    size, radius, label, aria_label, description, helper, placeholder, status, required, disabled,
                }
            },
            shown(date_time_range()),
        ),
        "duration" => (
            rsx! {
                ChronoField {
                    value: duration(), onchange: move |next| duration.set(next),
                    min: min_duration, max: max_duration, with_seconds, step,
                    validate: rules(validate),
                    size, radius, label, aria_label, description, helper, placeholder, status, required, disabled,
                }
            },
            shown(duration()),
        ),
        _ => (
            rsx! {
                ChronoField {
                    value: date(), onchange: move |next| date.set(next),
                    min: min_day, max: max_day, format, exclude_date, today, close_on_change, calendar, days,
                    validate: rules(validate),
                    size, radius, label, aria_label, description, helper, placeholder, status, required, disabled,
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
