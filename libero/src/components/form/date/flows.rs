//! What a field's dropdown shows when one picker is not enough: a calendar and
//! a clock for a day and a time, under tabs; for a range of them, the days,
//! then the start's time, then the end's.

use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use dioxus::prelude::*;

use super::{
    DatePicker, DateRange,
    calendar::{Calendar, Selection},
    parse_time::MIDNIGHT,
    picker_field::FieldValue,
    time_picker::Clock,
    today::use_today,
};
use crate::{
    components::{
        accessibility::VisuallyHidden,
        common::{ClassList, HtmlTag, Input, States, base_color},
        layout::use_box,
        navigation::{TabSpec, TabsView, render_tabs},
    },
    hooks::{ElementHandle, use_element, use_id, use_localization, use_theme},
    localization::DateLocale,
    platform::ElementApi,
    sx::Sx,
    theme::{Size, TimePickerVariant},
};

/// The tabs and the picker under them share this column.
const FLOW_STYLE: &str = "display: flex; flex-direction: column; gap: 8px;";
/// A picker takes the width it needs, centred under the tabs.
const PART_STYLE: &str = "display: flex; justify-content: center;";

/// The steps of `DateTimeFlow`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Part {
    Date,
    Time,
}

/// The steps of `DateTimeRangeFlow`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Step {
    Dates,
    Start,
    End,
}

impl Step {
    const ALL: [Self; 3] = [Self::Dates, Self::Start, Self::End];
}

/// The time limits that apply on `date`: `min`'s time only on `min`'s day.
fn time_limits(
    date: Option<NaiveDate>,
    min: Option<NaiveDateTime>,
    max: Option<NaiveDateTime>,
) -> (Option<NaiveTime>, Option<NaiveTime>) {
    let on = |limit: Option<NaiveDateTime>| {
        limit
            .filter(|limit| Some(limit.date()) == date)
            .map(|limit| limit.time())
    };
    (on(min), on(max))
}

/// A day as a tab shows it, `12 Oct`, with the year when `year`.
fn short_day(day: NaiveDate, year: bool, names: &DateLocale) -> String {
    let month = names.months_short[day.month0() as usize];
    match year {
        true => format!("{} {month} {}", day.day(), day.year()),
        false => format!("{} {month}", day.day()),
    }
}

/// A range of days as a tab shows it: `12–14 Oct`, `30 Sep – 2 Oct`, the
/// years only when they differ, `12 Oct –` while the end is missing.
pub(super) fn short_days(range: DateRange<NaiveDate>, names: &DateLocale) -> String {
    let start = range.start;
    let Some(end) = range.end else {
        return format!("{} –", short_day(start, false, names));
    };
    match (start.year() == end.year(), start.month() == end.month()) {
        (true, true) if start == end => short_day(start, false, names),
        (true, true) => format!("{}–{}", start.day(), short_day(end, false, names)),
        (true, false) => format!(
            "{} – {}",
            short_day(start, false, names),
            short_day(end, false, names)
        ),
        (false, _) => format!(
            "{} – {}",
            short_day(start, true, names),
            short_day(end, true, names)
        ),
    }
}

/// A time as a tab shows it: `09:00`, or `9:00 AM` on a 12-hour clock.
pub(super) fn short_time(
    time: NaiveTime,
    twelve: bool,
    seconds: bool,
    names: &DateLocale,
) -> String {
    let rest = match seconds {
        true => format!("{:02}:{:02}", time.minute(), time.second()),
        false => format!("{:02}", time.minute()),
    };
    match twelve {
        true => {
            let half = if time.hour() < 12 { names.am } else { names.pm };
            format!("{}:{rest} {half}", (time.hour() + 11) % 12 + 1)
        }
        false => format!("{:02}:{rest}", time.hour()),
    }
}

/// One tab: `name` alone before there is a value, else the value it shows
/// after a hidden `name, `, so the content names it with its visible text in
/// the name (2.5.3).
fn step_tab(name: &'static str, value: Option<String>) -> TabSpec {
    let content = match value {
        Some(value) => rsx! {
            VisuallyHidden { "{name}, " }
            "{value}"
        },
        None => rsx! { "{name}" },
    };
    TabSpec {
        name: String::new(),
        content,
        disabled: false,
    }
}

/// The APG tabs over the flow's steps, the picker as the panel. Calls hooks.
#[allow(clippy::too_many_arguments)]
fn step_tabs(
    id: String,
    label: &'static str,
    tabs: Vec<TabSpec>,
    selected: usize,
    onselect: Callback<usize>,
    size: Size,
    focusable: bool,
    panel: Element,
) -> Element {
    render_tabs(
        TabsView {
            tabs,
            selected: Some(selected),
            panel: rsx! {
                div { "data-slot": "part", style: PART_STYLE, {panel} }
            },
            onselect,
            color: base_color(None),
            full_width: true,
            manual: false,
            focusable,
            // The picker's first control takes focus.
            panel_stop: false,
            // Slim: never taller than the small strip.
            size: match size {
                Size::Xs => Size::Xs,
                _ => Size::Sm,
            },
            class: Input::default(),
            sx: Input::default(),
            states: Input::default(),
            attributes: vec![Attribute::new("aria-label", label, None, false)],
        },
        id,
    )
}

/// The column both flows stand in, wearing the caller's class and style.
fn flow_root(
    class: &Input<ClassList>,
    sx: &Input<Sx>,
    states: &Input<States>,
    attributes: Vec<Attribute>,
    root: &ElementHandle,
    children: Element,
) -> Element {
    use_box()
        .class(class)
        .sx(sx)
        .states(states)
        .style(Some(FLOW_STYLE.to_string()))
        .prepare()
        .element(root)
        .render(HtmlTag::Div, attributes, children)
}

/// Moves focus into the picker a step switched to, so a keyboard user is not
/// left on `body` when the day grid gives way to the clock.
fn use_handoff() -> (ElementHandle, Signal<bool>) {
    let root = use_element();
    let mut handoff = use_signal(|| false);
    use_effect(move || {
        if !handoff() {
            return;
        }
        handoff.set(false);
        let _ = root
            // The clock itself, not the readout above an analog face.
            .query_selector(
                "[data-slot='part'] [data-slot='face'][tabindex='0'], [data-slot='part'] [data-slot='columns'] [tabindex='0']",
            )
            .and_then(|element| element.focus());
    });
    (root, handoff)
}

/// A hidden input posting the value as ISO 8601, when the flow has a name.
fn hidden(name: Option<String>, value: Option<impl FieldValue>) -> Element {
    let value = value.map(FieldValue::iso).unwrap_or_default();
    rsx! {
        if let Some(name) = name {
            input { r#type: "hidden", name, value }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub(super) struct DateTimeFlowProps {
    value: Option<NaiveDateTime>,
    onpick: EventHandler<Option<NaiveDateTime>>,
    min: Option<NaiveDateTime>,
    max: Option<NaiveDateTime>,
    exclude_date: Option<Callback<NaiveDate, bool>>,
    today: Option<NaiveDate>,
    size: Input<Size>,
    variant: TimePickerVariant,
    with_seconds: bool,
    step: Option<u8>,
    twelve_hour: bool,
    calendar: crate::theme::CalendarVariant,
    days: usize,
    focusable: bool,
    #[props(default)]
    name: Option<String>,
    #[props(default)]
    class: Input<ClassList>,
    #[props(default)]
    sx: Input<Sx>,
    #[props(default)]
    states: Input<States>,
    #[props(default)]
    attributes: Vec<Attribute>,
}

/// Day, then time, under two tabs. Picking a day keeps the time and moves on
/// to the clock.
#[component]
pub(super) fn DateTimeFlow(props: DateTimeFlowProps) -> Element {
    let names = &use_localization().date;
    let mut part = use_signal(|| Part::Date);
    let (root, mut handoff) = use_handoff();
    let tabs_id = use_id();
    let focusable = props.focusable;
    let (value, onpick) = (props.value, props.onpick);
    let today = use_today(props.today);
    // A time picked with no day and no clock waits here for the day pick.
    let mut pending = use_signal(|| None::<NaiveTime>);
    let size = props.size.copied_or(use_theme().chrono_picker.size);
    let date = value.map(|value| value.date());
    let time = value.map(|value| value.time()).or(pending());
    let (min_time, max_time) = time_limits(date, props.min, props.max);

    let picker = match part() {
        Part::Date => rsx! {
            DatePicker {
                value: date,
                min: props.min.map(|min| min.date()),
                max: props.max.map(|max| max.date()),
                exclude_date: props.exclude_date,
                calendar: Input::Value(props.calendar),
                days: props.days,
                today,
                size,
                focusable: props.focusable,
                onchange: move |day: Option<NaiveDate>| {
                    if let Some(day) = day {
                        onpick.call(Some(NaiveDateTime::new(day, time.unwrap_or(MIDNIGHT))));
                        pending.set(None);
                        part.set(Part::Time);
                        handoff.set(focusable && root.query_selector(":focus").is_ok());
                    }
                },
            }
        },
        Part::Time => rsx! {
            Clock {
                value: time,
                onchange: move |next: Option<NaiveTime>| {
                    match (date.or(today), next) {
                        (Some(day), Some(next)) => onpick.call(Some(NaiveDateTime::new(day, next))),
                        (None, next) => pending.set(next),
                        _ => {}
                    }
                },
                variant: props.variant,
                with_seconds: props.with_seconds,
                step: props.step,
                twelve_hour: props.twelve_hour,
                min: min_time,
                max: max_time,
                size: Input::Value(size),
                focusable: props.focusable,
                name: None,
                class: Input::default(),
                sx: Input::default(),
                states: Input::default(),
                attributes: Vec::new(),
            }
        },
    };

    let onselect = use_callback(move |index: usize| {
        part.set(if index == 0 { Part::Date } else { Part::Time });
    });
    let tabs = vec![
        step_tab(names.date_label, None),
        step_tab(names.time_label, None),
    ];
    let strip = step_tabs(
        tabs_id(),
        names.part_switch_label,
        tabs,
        (part() == Part::Time) as usize,
        onselect,
        size,
        props.focusable,
        picker,
    );
    flow_root(
        &props.class,
        &props.sx,
        &props.states,
        props.attributes,
        &root,
        rsx! {
            {strip}
            {hidden(props.name, value)}
        },
    )
}

#[derive(Props, Clone, PartialEq)]
pub(super) struct DateTimeRangeFlowProps {
    value: Option<DateRange<NaiveDateTime>>,
    onpick: EventHandler<Option<DateRange<NaiveDateTime>>>,
    min: Option<NaiveDateTime>,
    max: Option<NaiveDateTime>,
    exclude_date: Option<Callback<NaiveDate, bool>>,
    today: Option<NaiveDate>,
    size: Input<Size>,
    variant: TimePickerVariant,
    with_seconds: bool,
    step: Option<u8>,
    twelve_hour: bool,
    focusable: bool,
    #[props(default)]
    name: Option<String>,
    #[props(default)]
    class: Input<ClassList>,
    #[props(default)]
    sx: Input<Sx>,
    #[props(default)]
    states: Input<States>,
    #[props(default)]
    attributes: Vec<Attribute>,
}

/// Three steps under three tabs: the days, the start's time, the end's time.
/// Each step moves on to the next once it is complete. The end never comes
/// before the start: on the start's day its clock starts at the start's time.
#[component]
pub(super) fn DateTimeRangeFlow(props: DateTimeRangeFlowProps) -> Element {
    let theme = use_theme();
    let names = &use_localization().date;
    let size = props.size.copied_or(theme.chrono_picker.size);
    let mut step = use_signal(|| Step::Dates);
    let (root, mut handoff) = use_handoff();
    let tabs_id = use_id();
    let focusable = props.focusable;
    let (value, onpick) = (props.value, props.onpick);
    let today = use_today(props.today);
    // Times picked before their day, as in `DateTimeFlow`.
    let mut pending_start = use_signal(|| None::<NaiveTime>);
    let mut pending_end = use_signal(|| None::<NaiveTime>);
    let start = value.map(|range| range.start);
    let end = value.and_then(|range| range.end);
    let start_time = start.map(|start| start.time()).or(pending_start());
    let end_time = end.map(|end| end.time()).or(pending_end());
    let emit = move |start: NaiveDateTime, end: Option<NaiveDateTime>| {
        onpick.call(Some(DateRange::new(start, end.map(|end| end.max(start)))));
    };
    let mut next_step = move |to: Step| {
        step.set(to);
        handoff.set(focusable && root.query_selector(":focus").is_ok());
    };
    // One identity across renders, so the calendar's props compare equal and
    // a re-render from above skips it.
    let onpick_day = use_callback(move |day: NaiveDate| {
        let days =
            value.map(|range| DateRange::new(range.start.date(), range.end.map(|end| end.date())));
        let days = DateRange::pick(days, day);
        let from = start_time.unwrap_or(MIDNIGHT);
        let start = NaiveDateTime::new(days.start, from);
        let end = days
            .end
            .map(|day| NaiveDateTime::new(day, end_time.unwrap_or(from)));
        pending_start.set(None);
        pending_end.set(None);
        emit(start, end);
        if end.is_some() {
            next_step(Step::Start);
        }
    });

    // Each step keyed apart: a fresh end clock starts on the hour.
    let shown = step();
    let picker = match shown {
        Step::Dates => rsx! {
            Calendar {
                key: "{shown:?}",
                selection: Selection::Range(value.map(|range| {
                    DateRange::new(range.start.date(), range.end.map(|end| end.date()))
                })),
                min: props.min.map(|min| min.date()),
                max: props.max.map(|max| max.date()),
                exclude_date: props.exclude_date,
                today,
                size,
                focusable: props.focusable,
                onpick: onpick_day,
            }
        },
        Step::Start => {
            let (min, max) = time_limits(start.map(|start| start.date()), props.min, props.max);
            let onchange = EventHandler::new(move |time: Option<NaiveTime>| {
                let Some(time) = time else {
                    return;
                };
                match start.map(|start| start.date()).or(today) {
                    Some(day) => emit(NaiveDateTime::new(day, time), end),
                    None => pending_start.set(Some(time)),
                }
            });
            rsx! {
                Clock {
                    key: "{shown:?}",
                    value: start_time,
                    onchange,
                    oncomplete: move |()| next_step(Step::End),
                    variant: props.variant,
                    with_seconds: props.with_seconds,
                    step: props.step,
                    twelve_hour: props.twelve_hour,
                    min,
                    max,
                    size: Input::Value(size),
                    focusable: props.focusable,
                    name: None,
                    class: Input::default(),
                    sx: Input::default(),
                    states: Input::default(),
                    attributes: Vec::new(),
                }
            }
        }
        Step::End => {
            let end_day = end.or(start).map(|moment| moment.date());
            let (mut min, max) = time_limits(end_day, props.min, props.max);
            // On the start's day the end cannot come before the start.
            if let Some(start) = start.filter(|start| Some(start.date()) == end_day) {
                min = Some(min.map_or(start.time(), |min| min.max(start.time())));
            }
            let onchange = EventHandler::new(move |time: Option<NaiveTime>| {
                let Some(time) = time else {
                    return;
                };
                match (start, end_day) {
                    (Some(start), Some(day)) => emit(start, Some(NaiveDateTime::new(day, time))),
                    _ => pending_end.set(Some(time)),
                }
            });
            rsx! {
                Clock {
                    key: "{shown:?}",
                    value: end_time,
                    onchange,
                    variant: props.variant,
                    with_seconds: props.with_seconds,
                    step: props.step,
                    twelve_hour: props.twelve_hour,
                    min,
                    max,
                    size: Input::Value(size),
                    focusable: props.focusable,
                    name: None,
                    class: Input::default(),
                    sx: Input::default(),
                    states: Input::default(),
                    attributes: Vec::new(),
                }
            }
        }
    };

    let onselect = use_callback(move |index: usize| step.set(Step::ALL[index]));
    let (twelve, seconds) = (props.twelve_hour, props.with_seconds);
    let days =
        value.map(|range| DateRange::new(range.start.date(), range.end.map(|end| end.date())));
    let tabs = vec![
        step_tab(names.dates_label, days.map(|days| short_days(days, names))),
        step_tab(
            names.start_time_label,
            start_time.map(|time| short_time(time, twelve, seconds, names)),
        ),
        step_tab(
            names.end_time_label,
            end_time.map(|time| short_time(time, twelve, seconds, names)),
        ),
    ];
    let selected = Step::ALL.iter().position(|at| *at == step()).unwrap_or(0);
    let strip = step_tabs(
        tabs_id(),
        names.range_steps_label,
        tabs,
        selected,
        onselect,
        size,
        props.focusable,
        picker,
    );
    flow_root(
        &props.class,
        &props.sx,
        &props.states,
        props.attributes,
        &root,
        rsx! {
            {strip}
            {hidden(props.name, value)}
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, month, day).expect("a real day")
    }

    #[test]
    fn a_range_of_days_shows_short() {
        let names = &DateLocale::ENGLISH;
        let range = |start, end| short_days(DateRange::new(start, end), names);
        assert_eq!(range(day(10, 12), Some(day(10, 14))), "12–14 Oct");
        assert_eq!(range(day(9, 30), Some(day(10, 2))), "30 Sep – 2 Oct");
        assert_eq!(range(day(10, 12), Some(day(10, 12))), "12 Oct");
        assert_eq!(range(day(10, 12), None), "12 Oct –");
        let next_year = NaiveDate::from_ymd_opt(2027, 1, 2).expect("a real day");
        assert_eq!(
            range(day(12, 30), Some(next_year)),
            "30 Dec 2026 – 2 Jan 2027"
        );
    }

    #[test]
    fn a_time_shows_short() {
        let names = &DateLocale::ENGLISH;
        let time = NaiveTime::from_hms_opt(17, 30, 5).expect("a real time");
        assert_eq!(short_time(time, false, false, names), "17:30");
        assert_eq!(short_time(time, true, false, names), "5:30 PM");
        assert_eq!(short_time(time, false, true, names), "17:30:05");
        assert_eq!(short_time(MIDNIGHT, true, false, names), "12:00 AM");
    }
}
