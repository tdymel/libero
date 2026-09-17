use super::date_common::{
    SIZES, calendar_controls, day_limits, field_controls, has_days, has_time, is_on, is_weekend,
    moment_limits, shared_controls, shown, status_of, step_of, text_of, time_limits, today_of,
    twelve_hour_of,
};
use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap, prop, props};
use dioxus::prelude::*;
use libero::chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use libero::components::{
    Code, DateField, DateLevel, DateRange, Flex, Kbd, Rule, Text, Validators, not_empty,
};
use libero::{use_formats_handle, use_localization_handle};

use super::date_locales::{FORMATS, LANGUAGES, options, picked};

const KINDS: [&str; 7] = [
    "date",
    "month",
    "year",
    "time",
    "date-time",
    "date-range",
    "date-time-range",
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
                    prop("level", "DateLevel").default("Day").doc("Types and picks a `NaiveDate` as a day, a month (its first day) or a year (its January 1). Ignored for every other value."),
                    prop("format", "String")
                        .default("(Formats::date)(level)")
                        .doc("How the text shows the value, in dayjs tokens, at the field's level. Defaults to the provider's `(Formats::date)(level)`: `MMMM D, YYYY` in American formats, `D. MMMM YYYY` in German, and `MMMM YYYY`, `YYYY` in both. Typing is lenient either way: only the order of day, month and year has to match."),
                    prop("time_format", "String").default("Formats::time").doc("How the text shows a time: `h:mm A` in American formats, `HH:mm` in German."),
                    prop("min", "V::Bound").doc("The earliest value that can be picked or typed. For a range, the earliest end: a `NaiveDate` or `NaiveDateTime`."),
                    prop("max", "V::Bound").doc("The latest value, likewise."),
                    prop("exclude_date", "Callback<NaiveDate, bool>").doc("Days that cannot be picked or typed. Ignored for a time."),
                    prop("today", "NaiveDate").doc("The day marked as today, and the year typed text without one falls back to. Unset, the platform clock answers after mount."),
                    prop("variant", "TimePickerVariant").default("analog").doc("The clock, for values with a time."),
                    prop("with_seconds", "bool").default("false").doc("Seconds in the text and the clock, for values with a time."),
                    prop("step", "u8").default("5").doc("Minutes between the offered minutes. Defaults to the theme's `TimePickerDefaults::step`."),
                    prop("twelve_hour", "bool").doc("A 12-hour clock. Defaults to whether the time format is one."),
                    prop("calendar", "CalendarVariant").default("full").doc("A month of days, or `mini`: one row of days with buttons that page it. For a day or a date-time."),
                    prop("days", "usize").default("7").doc("Days in the mini calendar's row."),
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
                        .doc("Validation state. Text the field cannot accept shows why instead: `DateLocale::invalid_date`, or the `min`/`max` or `exclude_date` it missed."),
                    prop("required", "bool").default("false").doc("Adds `required` to the input and an asterisk to the label."),
                    prop("disabled", "bool").default("false").doc("Disables typing and the dropdown, and dims the field."),
                    prop("readonly", "bool").default("false").doc("Focusable and posted with the form, but not editable - unlike `disabled`, which drops the field from the tab order and from the post."),
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
                    " of either picks two. "
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
                    "Typed text stays as typed until the field blurs or Enter is pressed, then it is read leniently: "
                    "any separator, one-digit days and months, month names in any case or as a unique prefix, "
                    "and a missing year taken from today. Only the order of day, month and year follows "
                    Code { source: "format" }
                    ". Two-digit years are rejected. Text that is not an accepted value stays, and the field's error says why: "
                    "unreadable, or before, after or outside "
                    Code { source: "min" }
                    " and "
                    Code { source: "max" }
                    " in the field's own format, or an excluded day. "
                    "The form gets ISO 8601, whatever the text shows."
                }
                Text {
                    "Two things given to "
                    Code { source: "LiberoProvider" }
                    " shape the text, apart. The language - month and day names, labels and errors - is the "
                    Code { source: "DateLocale" }
                    " in the "
                    Code { source: "Localization" }
                    "'s "
                    Code { source: "date" }
                    ": "
                    Code { source: "Localization::ENGLISH" }
                    ", the default, or "
                    Code { source: "Localization::GERMAN" }
                    ". The formats - the date and time patterns, the first weekday, the range separator - are "
                    Code { source: "formats" }
                    ": "
                    Code { source: "Formats::AMERICAN" }
                    ", the default - Sunday first, a 12-hour clock - or "
                    Code { source: "Formats::GERMAN" }
                    " - Monday first, a 24-hour clock, "
                    Code { source: "14. September 2026" }
                    ". This site uses English with German formats. The "
                    Code { source: "language" }
                    " and "
                    Code { source: "formats" }
                    " controls switch each."
                }
                Text {
                    "A typed "
                    Code { source: "value" }
                    " alone does not name the type, because dioxus converts every prop. A handler that stores into a typed signal names it; "
                    "without one, a turbofish does: "
                    Code { source: "DateField::<NaiveTime> {{ .. }}" }
                    ". For one value type there are "
                    Code { source: "DayField" }
                    ", "
                    Code { source: "TimeField" }
                    ", "
                    Code { source: "DateTimeField" }
                    ", "
                    Code { source: "DateRangeField" }
                    " and "
                    Code { source: "DateTimeRangeField" }
                    ": only the props that type uses, no turbofish, and a value of the wrong type is a plain type mismatch."
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
                component: "DateField",
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
                    [controls, calendar_controls(), shared_controls(), field_controls()].concat()
                },
                render: move |values: DemoValues| rsx! {
                    DateFieldDemo { values }
                },
            }
            DocSection {
                title: "Accessibility",
                Text {
                    "Focus opens the dropdown and stays in the text, so typing works at once. "
                    Kbd { "↓" } " moves focus into the picker, onto the picked day or the clock, where the "
                    Code { source: "DatePicker" }
                    " keys apply. "
                    Kbd { "Escape" } " goes back to the text, and so does a pick that closes the dropdown. "
                    "Focus leaving both the text and the dropdown closes it. A mouse click in the dropdown leaves focus in the text."
                }
            }
        }
    }
}

/// One signal per value type, so switching types keeps what was picked.
#[component]
fn DateFieldDemo(values: DemoValues) -> Element {
    let mut date = use_signal(|| day(14));
    let mut month = use_signal(|| day(1));
    let mut year = use_signal(|| NaiveDate::from_ymd_opt(2026, 1, 1));
    let mut time = use_signal(|| NaiveTime::from_hms_opt(9, 30, 0));
    let mut date_time = use_signal(|| moment(14, 9));
    let mut date_range = use_signal(|| day(14).map(|start| DateRange::new(start, day(18))));
    let mut date_time_range =
        use_signal(|| moment(14, 9).map(|start| DateRange::new(start, moment(16, 17))));

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
    let label = text_of(&values, "label", "When");
    let aria_label = label.is_none().then_some("When");
    let description = text_of(&values, "description", "Your local time.");
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

    let (field, readout) = match values.str("value").as_str() {
        "month" => (
            rsx! {
                DateField {
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
                DateField {
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
                DateField {
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
                DateField {
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
                DateField {
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
                DateField {
                    value: date_time_range(), onchange: move |next| date_time_range.set(next),
                    min: min_moment, max: max_moment, format, time_format, variant, exclude_date, today,
                    with_seconds, step, twelve_hour, close_on_change,
                    validate: rules(validate),
                    size, radius, label, aria_label, description, helper, placeholder, status, required, disabled,
                }
            },
            shown(date_time_range()),
        ),
        _ => (
            rsx! {
                DateField {
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
