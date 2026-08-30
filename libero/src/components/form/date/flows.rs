//! What a field's dropdown shows when one picker is not enough: a calendar and
//! a clock, for a day and a time, and both twice over for a range of them.

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use dioxus::prelude::*;

use super::{
    DatePicker, DateRange, TimePicker,
    calendar::{Calendar, Selection},
    parse_time::MIDNIGHT,
};
use crate::{
    components::{Input, OptionLabel, Options, SegmentedControl},
    hooks::use_theme,
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

/// A switch between the calendar and the clock. Not focusable: the field's
/// text input keeps focus while the dropdown is open.
fn part_switch(part: Signal<Part>, size: Size) -> Element {
    let names = &use_theme().date;
    let mut part = part;
    rsx! {
        SegmentedControl {
            value: part(),
            onchange: move |next| part.set(next),
            size,
            full_width: true,
            focusable: false,
            label: Callback::new(move |part: Part| {
                OptionLabel::from(match part {
                    Part::Date => names.date_label,
                    Part::Time => names.time_label,
                })
            }),
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
    size: Size,
    variant: TimePickerVariant,
    with_seconds: bool,
    step: Option<u8>,
    twelve_hour: bool,
}

/// Day, then time. Picking a day keeps the time and moves on to the clock.
#[component]
pub(super) fn DateTimeFlow(props: DateTimeFlowProps) -> Element {
    let mut part = use_signal(|| Part::Date);
    let (value, onpick, today) = (props.value, props.onpick, props.today);
    let date = value.map(|value| value.date());
    let time = value.map(|value| value.time());
    let (min_time, max_time) = time_limits(date, props.min, props.max);

    let picker = match part() {
        Part::Date => rsx! {
            DatePicker {
                value: date,
                min: props.min.map(|min| min.date()),
                max: props.max.map(|max| max.date()),
                exclude_date: props.exclude_date,
                today,
                size: props.size,
                focusable: false,
                onchange: move |day: Option<NaiveDate>| {
                    if let Some(day) = day {
                        onpick.call(Some(NaiveDateTime::new(day, time.unwrap_or(MIDNIGHT))));
                        part.set(Part::Time);
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
                size: props.size,
                focusable: false,
                onchange: move |next: Option<NaiveTime>| {
                    if let (Some(day), Some(next)) = (date.or(today), next) {
                        onpick.call(Some(NaiveDateTime::new(day, next)));
                    }
                },
            }
        },
    };

    rsx! {
        div { style: FLOW_STYLE,
            {part_switch(part, props.size)}
            {picker}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub(super) struct DateTimeRangeFlowProps {
    value: Option<DateRange<NaiveDateTime>>,
    onpick: EventHandler<Option<DateRange<NaiveDateTime>>>,
    min: Option<NaiveDateTime>,
    max: Option<NaiveDateTime>,
    exclude_date: Option<Callback<NaiveDate, bool>>,
    today: Option<NaiveDate>,
    size: Size,
    variant: TimePickerVariant,
    with_seconds: bool,
    step: Option<u8>,
    twelve_hour: bool,
}

/// Start, then end - each a day, then a time. The end cannot be picked before
/// the start's day, and an end that lands before the start swaps with it.
#[component]
pub(super) fn DateTimeRangeFlow(props: DateTimeRangeFlowProps) -> Element {
    let names = &use_theme().date;
    let mut side = use_signal(|| Side::Start);
    let mut part = use_signal(|| Part::Date);
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
                size: props.size,
                focusable: false,
                onpick: move |day: NaiveDate| {
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
                },
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
                size: props.size,
                focusable: false,
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

    rsx! {
        div { style: FLOW_STYLE,
            SegmentedControl {
                value: side(),
                onchange: move |next| {
                    side.set(next);
                    part.set(Part::Date);
                },
                size: props.size,
                full_width: true,
                focusable: false,
                label: Callback::new(move |side: Side| {
                    OptionLabel::from(match side {
                        Side::Start => names.start_label,
                        Side::End => names.end_label,
                    })
                }),
            }
            {part_switch(part, props.size)}
            {picker}
        }
    }
}
