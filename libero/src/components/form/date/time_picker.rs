use dioxus::prelude::*;

use chrono::{NaiveTime, Timelike};

use super::{format::uses_twelve_hours, parse_time::MIDNIGHT};
use crate::{
    components::{
        HtmlTag, Input, States,
        common::{base_props, focus_ring_sx, input_from_str},
        layout::use_box,
    },
    hooks::{ElementHandle, use_element, use_theme},
    platform::ElementApi,
    sx::{StaticSx, sx},
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
});

base_props! {
    pub struct TimePickerProps {
        /// The picked time; strictly controlled. `None` picks nothing.
        #[props(default)]
        value: Option<NaiveTime>,
        /// Called with the time the caller should hold next. A part picked
        /// with no value yet starts from `min`, else midnight.
        #[props(default)]
        onchange: Option<EventHandler<Option<NaiveTime>>>,
        /// Columns of numbers, or a clock face. Digital by default.
        #[props(default, into)]
        variant: Input<TimePickerVariant>,
        /// A seconds column. Digital only.
        #[props(default)]
        with_seconds: Option<bool>,
        /// Minutes between the offered minutes. `1` by default.
        #[props(default)]
        step: Option<u8>,
        /// A 12-hour clock with AM and PM. Defaults to whether the theme's
        /// `DateDefaults::time_format` is one.
        #[props(default)]
        twelve_hour: Option<bool>,
        /// The earliest time that can be picked.
        #[props(default)]
        min: Option<NaiveTime>,
        /// The latest time that can be picked.
        #[props(default)]
        max: Option<NaiveTime>,
        #[props(default, into)]
        size: Input<Size>,
        /// Emits a hidden input of that name, posting the time as `HH:MM:SS`.
        #[props(default, into)]
        name: Option<String>,
        /// `false` keeps the buttons out of the tab order. On by default.
        #[props(default)]
        focusable: Option<bool>,
    }
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
#[component]
pub fn TimePicker(props: TimePickerProps) -> Element {
    let theme = use_theme();
    let names = &theme.date;
    let size = props.size.copied_or(theme.time_picker.size);
    let variant = props.variant.copied_or(theme.time_picker.variant);
    let with_seconds = props.with_seconds.unwrap_or(false);
    let step = props.step.unwrap_or(1).clamp(1, 30);
    let twelve = props
        .twelve_hour
        .unwrap_or_else(|| uses_twelve_hours(names.time_format));
    let focusable = props.focusable.unwrap_or(true);
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
    let hours_column = use_element();
    let minutes_column = use_element();
    let seconds_column = use_element();
    // Once each column mounts, its picked option scrolls to the top.
    use_effect(move || {
        for column in [hours_column, minutes_column, seconds_column] {
            scroll_picked_into_view(column);
        }
    });

    let option =
        move |key: String, label: String, selected: bool, disabled: bool, pick: NaiveTime| {
            rsx! {
                button {
                    key: "{key}",
                    r#type: "button",
                    "data-slot": "option",
                    "data-selected": selected.then_some("true"),
                    disabled,
                    tabindex,
                    onclick: move |_| emit(pick),
                    "{label}"
                }
            }
        };
    let meridiem = twelve.then(|| {
        rsx! {
            div { "data-slot": "column",
                {option("am".into(), names.am.into(), value.is_some() && !pm, false, at(base.hour() % 12, base.minute(), base.second()))}
                {option("pm".into(), names.pm.into(), value.is_some() && pm, false, at(base.hour() % 12 + 12, base.minute(), base.second()))}
            }
        }
    });

    let body = match variant {
        TimePickerVariant::Digital => {
            let hour_labels: Vec<u32> = match twelve {
                true => std::iter::once(12).chain(1..12).collect(),
                false => (0..24).collect(),
            };
            let hours = hour_labels.into_iter().map(|label| {
                let hour = hour_of(label);
                option(
                    label.to_string(),
                    format!("{label:02}"),
                    value.is_some_and(|value| value.hour() == hour),
                    !within(at(hour, 0, 0), at(hour, 59, 59)),
                    at(hour, base.minute(), base.second()),
                )
            });
            let minutes = (0..60u32).step_by(step as usize).map(|minute| {
                option(
                    minute.to_string(),
                    format!("{minute:02}"),
                    value.is_some_and(|value| value.minute() == minute),
                    !within(at(base.hour(), minute, 0), at(base.hour(), minute, 59)),
                    at(base.hour(), minute, base.second()),
                )
            });
            let seconds = with_seconds.then(|| {
                let options = (0..60u32).map(|second| {
                    option(
                        second.to_string(),
                        format!("{second:02}"),
                        value.is_some_and(|value| value.second() == second),
                        !within(
                            at(base.hour(), base.minute(), second),
                            at(base.hour(), base.minute(), second),
                        ),
                        at(base.hour(), base.minute(), second),
                    )
                });
                rsx! {
                    div {
                        "data-slot": "column",
                        "aria-label": names.seconds_label,
                        onmounted: seconds_column.mount(),
                        {options}
                    }
                }
            });
            rsx! {
                div { "data-slot": "columns",
                    div {
                        "data-slot": "column",
                        "aria-label": names.hours_label,
                        onmounted: hours_column.mount(),
                        {hours}
                    }
                    div {
                        "data-slot": "column",
                        "aria-label": names.minutes_label,
                        onmounted: minutes_column.mount(),
                        {minutes}
                    }
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
                        tabindex,
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
                div { "data-slot": "face",
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
    let root = use_box()
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
    root.render(
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
