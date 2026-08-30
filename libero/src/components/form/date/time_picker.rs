use dioxus::prelude::*;

use chrono::{NaiveTime, Timelike};

use super::{
    date_value::{DateValue, PickerArgs, PickerOptions},
    format::uses_twelve_hours,
    parse_time::MIDNIGHT,
    props::date_props,
};
use crate::{
    components::{
        ClassList, HtmlTag, Input, States,
        common::{focus_ring_sx, input_from_str},
        layout::use_box,
    },
    hooks::{ElementHandle, use_element, use_theme},
    platform::ElementApi,
    sx::{StaticSx, Sx, sx},
    theme::{
        DATE_PICKER_DAY, DATE_PICKER_FONT_SIZE, DatePickerDefaults, Size, SizeCss,
        TimePickerVariant,
    },
};

input_from_str!(TimePickerVariant);

static TIME_PICKER_SX: StaticSx = StaticSx::new(|| {
    let day = DATE_PICKER_DAY.value();
    let button = sx()
        .display("flex")
        .align_items("center")
        .justify_content("center")
        .padding("0 4px")
        .margin("0")
        .border_style("none")
        .border_radius(SizeCss::RADIUS.value(Size::Sm))
        .background("transparent")
        .color("inherit")
        .font_family("inherit")
        .font_size("inherit")
        .cursor("pointer")
        .hover(sx().background("grey.1"));
    DatePickerDefaults::theme_vars()
        .display("inline-flex")
        .flex_direction("column")
        .align_items("center")
        .gap("8px")
        .font_size(DATE_PICKER_FONT_SIZE.value())
        .selector("& > [data-slot='columns']", sx().display("flex").gap("4px"))
        .selector(
            "& [data-slot='column']",
            sx().display("flex")
                .flex_direction("column")
                .gap("2px")
                .padding("2px")
                .overflow_y("auto")
                .max_height(format!("calc(7 * {day})")),
        )
        .selector(
            "& [data-slot='option']",
            button
                .clone()
                .min_width(format!("calc(1.5 * {day})"))
                .min_height(day.clone())
                .height(day.clone()),
        )
        .selector(
            "& > [data-slot='readout']",
            sx().display("flex")
                .align_items("center")
                .gap("4px")
                .font_size("1.5em"),
        )
        .selector(
            "& [data-slot='readout'] button",
            button.clone().height(format!("calc(1.25 * {day})")),
        )
        .selector(
            "& [data-slot='readout'] [data-active]",
            sx().background("primary.1").color("primary-contrast.1"),
        )
        .selector(
            "& [data-slot='face']",
            sx().position("relative")
                .width(format!("calc(7 * {day})"))
                .height(format!("calc(7 * {day})"))
                .border_radius("50%")
                .background("grey.1"),
        )
        .selector(
            "& [data-slot='mark']",
            button
                .position("absolute")
                .width(day.clone())
                .height(day)
                .border_radius("50%")
                .padding("0"),
        )
        .selector(
            "& [data-slot='hand'], & [data-slot='pivot']",
            sx().position("absolute").background("primary.6"),
        )
        .selector("& [data-selected]", {
            sx().background("primary.6")
                .color("primary-contrast.6")
                .hover(sx().background("primary.7"))
        })
        .selector(
            "& button:disabled",
            sx().opacity("0.4")
                .cursor("not-allowed")
                .hover(sx().background("transparent")),
        )
        .selector("& button:focus-visible", focus_ring_sx())
        .selector("& [data-slot='face']:focus-visible", focus_ring_sx())
});

date_props! {
    picker TimePickerProps(NaiveTime, NaiveTime): clock, limits
}

/// Which hand an analog picker is setting.
#[derive(Clone, Copy, PartialEq)]
enum Hand {
    Hour,
    Minute,
}

/// A time to pick - scrolling columns of hours, minutes and seconds, or an
/// analog clock face that takes the hour, then the minute.
///
/// Controlled: it renders `value` and asks for a new one through `onchange`.
/// A part picked with no value yet starts from `min`, else midnight.
#[component]
pub fn TimePicker(props: TimePickerProps) -> Element {
    let theme = use_theme();
    NaiveTime::picker(PickerArgs {
        value: props.value,
        onchange: props.onchange,
        options: PickerOptions {
            min: props.min,
            max: props.max,
            variant: props.variant.copied_or(theme.time_picker.variant),
            with_seconds: props.with_seconds.unwrap_or(false),
            step: props.step,
            twelve_hour: props
                .twelve_hour
                .unwrap_or_else(|| uses_twelve_hours(theme.date.time_format)),
            ..PickerOptions::default()
        },
        today: None,
        size: props.size,
        focusable: props.focusable.unwrap_or(true),
        name: props.name,
        class: props.class,
        sx: props.sx,
        states: props.states,
        attributes: props.attributes,
    })
}

/// What `TimePicker` draws, with every option resolved. Its props are plain,
/// so `DatePicker` can hand on the caller's attributes.
#[derive(Props, Clone, PartialEq)]
pub(super) struct ClockProps {
    value: Option<NaiveTime>,
    onchange: Option<EventHandler<Option<NaiveTime>>>,
    variant: TimePickerVariant,
    with_seconds: bool,
    step: Option<u8>,
    twelve_hour: bool,
    min: Option<NaiveTime>,
    max: Option<NaiveTime>,
    size: Input<Size>,
    focusable: bool,
    name: Option<String>,
    class: Input<ClassList>,
    sx: Input<Sx>,
    states: Input<States>,
    attributes: Vec<Attribute>,
}

/// A column of the digital variant.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Column {
    Hours,
    Minutes,
    Seconds,
    Meridiem,
}

/// One option of a digital column.
struct Choice {
    key: String,
    label: String,
    selected: bool,
    disabled: bool,
    pick: NaiveTime,
}

#[component]
pub(super) fn Clock(props: ClockProps) -> Element {
    let theme = use_theme();
    let names = &theme.date;
    let size = props.size.copied_or(theme.time_picker.size);
    let variant = props.variant;
    let with_seconds = props.with_seconds;
    let step = props.step.unwrap_or(theme.time_picker.step).clamp(1, 30);
    let twelve = props.twelve_hour;
    let focusable = props.focusable;
    let tabindex = if focusable { "0" } else { "-1" };
    let (value, min, max, onchange) = (props.value, props.min, props.max, props.onchange);

    let base = value.or(min).unwrap_or(MIDNIGHT);
    let pm = base.hour() >= 12;
    let within = move |from: NaiveTime, to: NaiveTime| {
        !(min.is_some_and(|min| to < min) || max.is_some_and(|max| from > max))
    };
    let at = |hour: u32, minute: u32, second: u32| {
        NaiveTime::from_hms_opt(hour, minute, second).expect("in range")
    };
    // The hour a 12-hour label stands for, in the half of the day `base` is in.
    let hour_of = move |label: u32| match twelve {
        true => label % 12 + if pm { 12 } else { 0 },
        false => label,
    };
    let emit = move |next: NaiveTime| {
        if let Some(onchange) = &onchange {
            onchange.call(Some(next));
        }
    };

    let mut hand = use_signal(|| Hand::Hour);
    let root = use_element();
    let hours_column = use_element();
    let minutes_column = use_element();
    let seconds_column = use_element();
    let meridiem_column = use_element();
    // Once each column mounts, its picked option scrolls to the top.
    use_effect(move || {
        for column in [hours_column, minutes_column, seconds_column] {
            scroll_picked_into_view(column);
        }
    });
    // The digital option the keyboard is on, and a selector to focus after the
    // next render.
    let mut active = use_signal(|| None::<(Column, usize)>);
    let mut focus_request = use_signal(|| None::<String>);
    use_effect(move || {
        let Some(selector) = focus_request() else {
            return;
        };
        focus_request.set(None);
        let _ = root
            .query_selector(&selector)
            .and_then(|element| element.focus());
    });

    // One tab stop per column: the option the keyboard is on, else the picked
    // one, else the first enabled. Up and Down, Home and End move within it.
    let column = move |column: Column,
                       label: Option<&'static str>,
                       handle: ElementHandle,
                       choices: Vec<Choice>| {
        let enabled: Vec<usize> = choices
            .iter()
            .enumerate()
            .filter(|(_, choice)| !choice.disabled)
            .map(|(index, _)| index)
            .collect();
        let stop = active()
            .filter(|(active, index)| *active == column && enabled.contains(index))
            .map(|(_, index)| index)
            .or_else(|| {
                choices
                    .iter()
                    .position(|choice| choice.selected && !choice.disabled)
            })
            .or_else(|| enabled.first().copied());
        let onkeydown = move |event: KeyboardEvent| {
            let Some(at) = stop.and_then(|stop| enabled.iter().position(|index| *index == stop))
            else {
                return;
            };
            let next = match event.key() {
                Key::ArrowDown => enabled[(at + 1).min(enabled.len() - 1)],
                Key::ArrowUp => enabled[at.saturating_sub(1)],
                Key::Home => enabled[0],
                Key::End => enabled[enabled.len() - 1],
                _ => return,
            };
            event.prevent_default();
            active.set(Some((column, next)));
            focus_request.set(Some(format!(
                "[data-column='{column:?}'] [data-index='{next}']"
            )));
        };
        let options = choices.into_iter().enumerate().map(|(index, choice)| {
            let pick = choice.pick;
            rsx! {
                button {
                    key: "{choice.key}",
                    r#type: "button",
                    "data-slot": "option",
                    "data-index": "{index}",
                    "data-selected": choice.selected.then_some("true"),
                    disabled: choice.disabled,
                    tabindex: if focusable && stop == Some(index) { "0" } else { "-1" },
                    onfocus: move |_| active.set(Some((column, index))),
                    onclick: move |_| emit(pick),
                    "{choice.label}"
                }
            }
        });
        rsx! {
            div {
                "data-slot": "column",
                "data-column": "{column:?}",
                "aria-label": label,
                onmounted: handle.mount(),
                onkeydown,
                {options}
            }
        }
    };

    let body = match variant {
        TimePickerVariant::Digital => {
            let hour_labels: Vec<u32> = match twelve {
                true => std::iter::once(12).chain(1..12).collect(),
                false => (0..24).collect(),
            };
            let hours = hour_labels
                .into_iter()
                .map(|label| {
                    let hour = hour_of(label);
                    Choice {
                        key: label.to_string(),
                        label: format!("{label:02}"),
                        selected: value.is_some_and(|value| value.hour() == hour),
                        disabled: !within(at(hour, 0, 0), at(hour, 59, 59)),
                        pick: at(hour, base.minute(), base.second()),
                    }
                })
                .collect();
            let minutes = (0..60u32)
                .step_by(step as usize)
                .map(|minute| Choice {
                    key: minute.to_string(),
                    label: format!("{minute:02}"),
                    selected: value.is_some_and(|value| value.minute() == minute),
                    disabled: !within(at(base.hour(), minute, 0), at(base.hour(), minute, 59)),
                    pick: at(base.hour(), minute, base.second()),
                })
                .collect();
            let seconds = with_seconds.then(|| {
                let choices = (0..60u32)
                    .map(|second| {
                        let time = at(base.hour(), base.minute(), second);
                        Choice {
                            key: second.to_string(),
                            label: format!("{second:02}"),
                            selected: value.is_some_and(|value| value.second() == second),
                            disabled: !within(time, time),
                            pick: time,
                        }
                    })
                    .collect();
                column(
                    Column::Seconds,
                    Some(names.seconds_label),
                    seconds_column,
                    choices,
                )
            });
            let meridiem = twelve.then(|| {
                let choices = vec![
                    Choice {
                        key: "am".into(),
                        label: names.am.into(),
                        selected: value.is_some() && !pm,
                        disabled: false,
                        pick: at(base.hour() % 12, base.minute(), base.second()),
                    },
                    Choice {
                        key: "pm".into(),
                        label: names.pm.into(),
                        selected: value.is_some() && pm,
                        disabled: false,
                        pick: at(base.hour() % 12 + 12, base.minute(), base.second()),
                    },
                ];
                column(Column::Meridiem, None, meridiem_column, choices)
            });
            rsx! {
                div { "data-slot": "columns",
                    {column(Column::Hours, Some(names.hours_label), hours_column, hours)}
                    {column(Column::Minutes, Some(names.minutes_label), minutes_column, minutes)}
                    {seconds}
                    {meridiem}
                }
            }
        }
        TimePickerVariant::Analog => {
            let mark = move |index: u32,
                             radius: f64,
                             label: String,
                             selected: bool,
                             disabled: bool,
                             pick: NaiveTime,
                             next: Option<Hand>| {
                let angle = f64::from(index) * std::f64::consts::PI / 6.0;
                let style = format!(
                    "left: {:.3}%; top: {:.3}%; transform: translate(-50%, -50%)",
                    50.0 + radius * angle.sin(),
                    50.0 - radius * angle.cos()
                );
                rsx! {
                    button {
                        key: "{label}",
                        r#type: "button",
                        "data-slot": "mark",
                        "data-selected": selected.then_some("true"),
                        disabled,
                        // The face is the tab stop; a click must not move focus
                        // onto a mark the hand change is about to replace.
                        tabindex: "-1",
                        onmousedown: move |event| event.prevent_default(),
                        style,
                        onclick: move |_| {
                            emit(pick);
                            if let Some(next) = next {
                                hand.set(next);
                            }
                        },
                        "{label}"
                    }
                }
            };
            let marks: Vec<Element> = match hand() {
                Hand::Hour => {
                    let mut marks: Vec<Element> = (0..12u32)
                        .map(|index| {
                            let label = if index == 0 { 12 } else { index };
                            let hour = hour_of(label);
                            mark(
                                index,
                                40.0,
                                label.to_string(),
                                value.is_some_and(|value| value.hour() == hour),
                                !within(at(hour, 0, 0), at(hour, 59, 59)),
                                at(hour, base.minute(), 0),
                                Some(Hand::Minute),
                            )
                        })
                        .collect();
                    // A 24-hour face rings 13 to 00 inside 1 to 12.
                    if !twelve {
                        marks.extend((0..12u32).map(|index| {
                            let hour = if index == 0 { 0 } else { index + 12 };
                            mark(
                                index,
                                26.0,
                                format!("{hour:02}"),
                                value.is_some_and(|value| value.hour() == hour),
                                !within(at(hour, 0, 0), at(hour, 59, 59)),
                                at(hour, base.minute(), 0),
                                Some(Hand::Minute),
                            )
                        }));
                    }
                    marks
                }
                Hand::Minute => (0..12u32)
                    .map(|index| {
                        let minute = index * 5;
                        mark(
                            index,
                            40.0,
                            format!("{minute:02}"),
                            value.is_some_and(|value| value.minute() == minute),
                            minute % u32::from(step) != 0
                                || !within(at(base.hour(), minute, 0), at(base.hour(), minute, 59)),
                            at(base.hour(), minute, 0),
                            None,
                        )
                    })
                    .collect(),
            };
            let pointer = value.map(|value| {
                let (degrees, radius) = match hand() {
                    Hand::Hour => {
                        let inner = !twelve && !(1..=12).contains(&value.hour());
                        (
                            f64::from(value.hour() % 12) * 30.0,
                            if inner { 26.0 } else { 40.0 },
                        )
                    }
                    Hand::Minute => (f64::from(value.minute()) * 6.0, 40.0),
                };
                let style = format!(
                    "left: 50%; top: 50%; width: 2px; height: {radius}%; transform-origin: 50% 100%; transform: translate(-50%, -100%) rotate({degrees}deg)"
                );
                rsx! {
                    div { "data-slot": "hand", style }
                }
            });
            let hour_text = value
                .map(|value| {
                    let hour = match twelve {
                        true => (value.hour() + 11) % 12 + 1,
                        false => value.hour(),
                    };
                    format!("{hour:02}")
                })
                .unwrap_or_else(|| "--".into());
            let minute_text = value
                .map(|value| format!("{:02}", value.minute()))
                .unwrap_or_else(|| "--".into());
            let halves = twelve.then(|| {
                rsx! {
                    button {
                        r#type: "button",
                        "data-active": (value.is_some() && !pm).then_some("true"),
                        tabindex,
                        onclick: move |_| emit(at(base.hour() % 12, base.minute(), base.second())),
                        {names.am}
                    }
                    button {
                        r#type: "button",
                        "data-active": (value.is_some() && pm).then_some("true"),
                        tabindex,
                        onclick: move |_| emit(at(base.hour() % 12 + 12, base.minute(), base.second())),
                        {names.pm}
                    }
                }
            });
            // The face is a slider over the hand it shows: the arrows step an
            // hour, or `step` minutes, past what `min` and `max` rule out; Enter
            // moves from the hour to the minute.
            let face_keydown = move |event: KeyboardEvent| {
                let delta: i64 = match event.key() {
                    Key::ArrowUp | Key::ArrowRight => 1,
                    Key::ArrowDown | Key::ArrowLeft => -1,
                    Key::Enter => {
                        event.prevent_default();
                        if hand() == Hand::Hour {
                            hand.set(Hand::Minute);
                        }
                        return;
                    }
                    _ => return,
                };
                event.prevent_default();
                let step = i64::from(step);
                let mut next = base;
                for _ in 0..60 {
                    let (candidate, open) = match hand() {
                        Hand::Hour => {
                            let hour = (i64::from(next.hour()) + delta).rem_euclid(24) as u32;
                            (
                                at(hour, next.minute(), 0),
                                within(at(hour, 0, 0), at(hour, 59, 59)),
                            )
                        }
                        Hand::Minute => {
                            let minute = i64::from(next.minute());
                            // Off the step, the first press lands on it.
                            let snapped = match delta > 0 {
                                true => (minute / step + 1) * step,
                                false => (minute + step - 1) / step * step - step,
                            };
                            let minute = snapped.rem_euclid(60) as u32;
                            (
                                at(next.hour(), minute, 0),
                                within(at(next.hour(), minute, 0), at(next.hour(), minute, 59)),
                            )
                        }
                    };
                    next = candidate;
                    if open {
                        emit(next);
                        return;
                    }
                }
            };
            let (face_label, face_text, face_now, face_max) = match hand() {
                Hand::Hour => (
                    names.hours_label,
                    hour_text.clone(),
                    value.map(|value| value.hour()),
                    23,
                ),
                Hand::Minute => (
                    names.minutes_label,
                    minute_text.clone(),
                    value.map(|value| value.minute()),
                    59,
                ),
            };
            rsx! {
                div { "data-slot": "readout",
                    button {
                        r#type: "button",
                        "aria-label": names.hours_label,
                        "data-active": (hand() == Hand::Hour).then_some("true"),
                        tabindex,
                        onclick: move |_| hand.set(Hand::Hour),
                        "{hour_text}"
                    }
                    span { ":" }
                    button {
                        r#type: "button",
                        "aria-label": names.minutes_label,
                        "data-active": (hand() == Hand::Minute).then_some("true"),
                        tabindex,
                        onclick: move |_| hand.set(Hand::Minute),
                        "{minute_text}"
                    }
                    {halves}
                }
                div {
                    "data-slot": "face",
                    role: "slider",
                    tabindex,
                    "aria-label": face_label,
                    "aria-valuetext": face_text,
                    "aria-valuenow": face_now,
                    "aria-valuemin": 0,
                    "aria-valuemax": face_max,
                    onkeydown: face_keydown,
                    {pointer}
                    div {
                        "data-slot": "pivot",
                        style: "left: 50%; top: 50%; width: 6px; height: 6px; border-radius: 50%; transform: translate(-50%, -50%)",
                    }
                    {marks.into_iter()}
                }
            }
        }
    };

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .into();
    let root_box = use_box()
        .framework_sx(&TIME_PICKER_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare();
    let hidden = props.name.map(|name| {
        rsx! {
            input {
                r#type: "hidden",
                name,
                value: value.map(|value| value.to_string()).unwrap_or_default(),
            }
        }
    });
    root_box.element(&root).render(
        HtmlTag::Div,
        props.attributes,
        rsx! {
            {body}
            {hidden}
        },
    )
}

/// Scrolls a column so its picked option sits at the top. The reads start
/// here, outside the `spawn`: under Blitz a read resolves where it is called.
fn scroll_picked_into_view(column: ElementHandle) {
    if !column.is_mounted() {
        return;
    }
    let Ok(picked) = column.query_selector("[data-selected]") else {
        return;
    };
    let (column_at, picked_at, scrolled) = (
        column.client_offset(),
        picked.client_offset(),
        column.scroll_offset(),
    );
    spawn(async move {
        let (Ok((_, column_y)), Ok((_, picked_y)), Ok((_, scrolled_y))) =
            (column_at.await, picked_at.await, scrolled.await)
        else {
            return;
        };
        let _ = column.scroll_to(0.0, scrolled_y + picked_y - column_y);
    });
}
