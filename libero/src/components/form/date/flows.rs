//! What a field's dropdown shows when one picker is not enough: a calendar and
//! a clock, for a day and a time, and both twice over for a range of them.

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use dioxus::prelude::*;

use super::{
    DateRange, DayPicker, TimePicker,
    calendar::{Calendar, Selection},
    parse_time::MIDNIGHT,
    picker_field::FieldValue,
};
use crate::{
    components::{
        ClassList, HtmlTag, Input, OptionLabel, Options, SegmentedControl, States, layout::use_box,
    },
    hooks::{ElementHandle, use_element, use_theme},
    platform::ElementApi,
    sx::Sx,
    theme::{Size, TimePickerVariant},
};

/// The two stacked pickers share this column.
const FLOW_STYLE: &str = "display: flex; flex-direction: column; gap: 8px;";

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Part {
    Date,
    Time,
}

impl Options for Part {
    fn options() -> &'static [Self] {
        &[Self::Date, Self::Time]
    }

    fn label(&self) -> String {
        format!("{self:?}")
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Side {
    Start,
    End,
}

impl Options for Side {
    fn options() -> &'static [Self] {
        &[Self::Start, Self::End]
    }

    fn label(&self) -> String {
        format!("{self:?}")
    }
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

/// A switch between the calendar and the clock. Calls hooks: its handlers
/// keep one identity across renders, so the control skips a re-render that
/// does not change the part.
fn part_switch(part: Signal<Part>, size: Size, focusable: bool) -> Element {
    let names = &use_theme().date;
    let mut part = part;
    let onchange = use_callback(move |next: Part| part.set(next));
    let label = use_callback(move |part: Part| {
        OptionLabel::from(match part {
            Part::Date => names.date_label,
            Part::Time => names.time_label,
        })
    });
    rsx! {
        SegmentedControl {
            value: part(),
            onchange,
            size,
            full_width: true,
            focusable,
            option_label: label,
        }
    }
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

/// Moves focus into the picker a pick switched to, so a keyboard user is not
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

/// Day, then time. Picking a day keeps the time and moves on to the clock.
#[component]
pub(super) fn DateTimeFlow(props: DateTimeFlowProps) -> Element {
    let mut part = use_signal(|| Part::Date);
    let (root, mut handoff) = use_handoff();
    let focusable = props.focusable;
    let (value, onpick, today) = (props.value, props.onpick, props.today);
    let size = props.size.copied_or(use_theme().date_picker.size);
    let date = value.map(|value| value.date());
    let time = value.map(|value| value.time());
    let (min_time, max_time) = time_limits(date, props.min, props.max);

    let picker = match part() {
        Part::Date => rsx! {
            DayPicker {
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
                        part.set(Part::Time);
                        handoff.set(focusable && root.query_selector(":focus").is_ok());
                    }
                },
            }
        },
        Part::Time => rsx! {
            TimePicker {
                value: time,
                variant: Input::Value(props.variant),
                with_seconds: props.with_seconds,
                step: props.step,
                twelve_hour: props.twelve_hour,
                min: min_time,
                max: max_time,
                size,
                focusable: props.focusable,
                onchange: move |next: Option<NaiveTime>| {
                    if let (Some(day), Some(next)) = (date.or(today), next) {
                        onpick.call(Some(NaiveDateTime::new(day, next)));
                    }
                },
            }
        },
    };

    flow_root(
        &props.class,
        &props.sx,
        &props.states,
        props.attributes,
        &root,
        rsx! {
            {part_switch(part, size, props.focusable)}
            div { "data-slot": "part", {picker} }
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

/// Start, then end - each a day, then a time. The end cannot be picked before
/// the start's day, and an end that lands before the start swaps with it.
#[component]
pub(super) fn DateTimeRangeFlow(props: DateTimeRangeFlowProps) -> Element {
    let theme = use_theme();
    let names = &theme.date;
    let size = props.size.copied_or(theme.date_picker.size);
    let mut side = use_signal(|| Side::Start);
    let mut part = use_signal(|| Part::Date);
    let (root, mut handoff) = use_handoff();
    let focusable = props.focusable;
    let (value, onpick, today) = (props.value, props.onpick, props.today);
    let start = value.map(|range| range.start);
    let end = value.and_then(|range| range.end);
    let emit = move |start: NaiveDateTime, end: Option<NaiveDateTime>| {
        onpick.call(Some(DateRange::new(start, end).ordered()));
    };
    // The end's side only means something once there is a start.
    let editing_end = side() == Side::End && start.is_some();
    let current = if editing_end { end } else { start };
    let (min_time, max_time) = time_limits(current.map(|value| value.date()), props.min, props.max);
    // One identity across renders, so the calendar's props compare equal and
    // a re-render from above skips it.
    let onpick_day = use_callback(move |day: NaiveDate| {
        match (editing_end, start) {
            (true, Some(start)) => {
                let time = end.map_or(start.time(), |end| end.time());
                emit(start, Some(NaiveDateTime::new(day, time)));
            }
            _ => {
                let time = start.map_or(MIDNIGHT, |start| start.time());
                let next = NaiveDateTime::new(day, time);
                emit(next, end.filter(|end| *end >= next));
            }
        }
        part.set(Part::Time);
        handoff.set(focusable && root.query_selector(":focus").is_ok());
    });

    let picker = match part() {
        Part::Date => rsx! {
            Calendar {
                selection: Selection::Range(value.map(|range| {
                    DateRange::new(range.start.date(), range.end.map(|end| end.date()))
                })),
                min: match editing_end {
                    true => start.map(|start| start.date()),
                    false => props.min.map(|min| min.date()),
                },
                max: props.max.map(|max| max.date()),
                exclude_date: props.exclude_date,
                today,
                size,
                focusable: props.focusable,
                onpick: onpick_day,
            }
        },
        Part::Time => rsx! {
            TimePicker {
                value: current.map(|value| value.time()),
                variant: Input::Value(props.variant),
                with_seconds: props.with_seconds,
                step: props.step,
                twelve_hour: props.twelve_hour,
                min: min_time,
                max: max_time,
                size,
                focusable: props.focusable,
                onchange: move |time: Option<NaiveTime>| {
                    let Some(time) = time else {
                        return;
                    };
                    match (editing_end, start) {
                        (true, Some(start)) => {
                            let day = end.map_or(start.date(), |end| end.date());
                            emit(start, Some(NaiveDateTime::new(day, time)));
                        }
                        _ => {
                            if let Some(day) = start.map(|start| start.date()).or(today) {
                                emit(NaiveDateTime::new(day, time), end);
                            }
                        }
                    }
                },
            }
        },
    };

    // One identity across renders, as in `part_switch`.
    let side_change = use_callback(move |next: Side| side.set(next));
    let side_label = use_callback(move |side: Side| {
        OptionLabel::from(match side {
            Side::Start => names.start_label,
            Side::End => names.end_label,
        })
    });
    let switch = part_switch(part, size, props.focusable);
    let children = rsx! {
            SegmentedControl {
                value: side(),
                onchange: side_change,
                size,
                full_width: true,
                focusable: props.focusable,
                option_label: side_label,
            }
            {switch}
            div { "data-slot": "part", {picker} }
            {hidden(props.name, value)}
    };
    flow_root(
        &props.class,
        &props.sx,
        &props.states,
        props.attributes,
        &root,
        children,
    )
}
