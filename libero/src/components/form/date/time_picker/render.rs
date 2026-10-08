use dioxus::prelude::*;

use chrono::Timelike;

use crate::{
    components::{common::Part, form::ChronoPickerPart},
    hooks::{Drag, ElementHandle},
};

use super::view::{ClockView, Column, Hand, INNER_RING, OUTER_RING, TWO_DIGITS, at};
use super::{SpinAt, SpinColumn, SpinOption};

impl ClockView {
    /// A digital clock, one spinbutton per part. `pick` gets a column's picked index,
    /// `ondone` the column Enter or typing has finished.
    pub(super) fn digital_view(
        self,
        pick: Callback<(&'static str, usize)>,
        ondone: Callback<&'static str>,
    ) -> Element {
        let ClockView {
            names,
            value,
            base,
            pm,
            twelve,
            with_seconds,
            step,
            focusable,
            ..
        } = self;
        // An empty column steps from `base`; a set one shows its value.
        let place = |index: usize, exact: bool| match (value, exact) {
            (None, _) => SpinAt::Empty(index),
            (Some(_), true) => SpinAt::At(index),
            (Some(_), false) => SpinAt::Past(index),
        };
        let column = move |column: Column,
                           label: String,
                           options: Vec<SpinOption>,
                           at: SpinAt,
                           text: &'static str,
                           valuetext: Option<String>,
                           page: usize| {
            rsx! {
                SpinColumn {
                    column: column.name(),
                    label,
                    options,
                    at,
                    text: if value.is_some() { text } else { "--" },
                    valuetext,
                    // Round like a clock's digits; AM and PM just sit apart.
                    wrap: column != Column::Meridiem,
                    page,
                    focusable,
                    onpick: pick,
                    ondone,
                }
            }
        };
        let hours = (0..if twelve { 12 } else { 24 })
            .map(|index| {
                let label = self.hour_label(index);
                let hour = self.hour_of(label);
                SpinOption {
                    text: TWO_DIGITS[label as usize],
                    disabled: !self.within(at(hour, 0, 0), at(hour, 59, 59)),
                }
            })
            .collect();
        let hour_index = (base.hour() % if twelve { 12 } else { 24 }) as usize;
        let hour_text = TWO_DIGITS[self.hour_label(hour_index) as usize];
        let minutes = (0..60u32)
            .step_by(step as usize)
            .map(|minute| SpinOption {
                text: TWO_DIGITS[minute as usize],
                disabled: !self.within(at(base.hour(), minute, 0), at(base.hour(), minute, 59)),
            })
            .collect();
        let minute = base.minute();
        let step = u32::from(step);
        let minutes_at = place((minute / step) as usize, minute % step == 0);
        let seconds = with_seconds.then(|| {
            let options = (0..60u32)
                .map(|second| {
                    let time = at(base.hour(), base.minute(), second);
                    SpinOption {
                        text: TWO_DIGITS[second as usize],
                        disabled: !self.within(time, time),
                    }
                })
                .collect();
            let second = base.second() as usize;
            rsx! {
                span { "data-slot": ChronoPickerPart::Separator.slot(), "aria-hidden": "true", ":" }
                {column(Column::Seconds, names.seconds_label.to_string(), options, place(second, true), TWO_DIGITS[second], Some((names.seconds_value)(second as u32)), 15)}
            }
        });
        let meridiem = twelve.then(|| {
            let options = vec![
                SpinOption {
                    text: names.am,
                    disabled: self.half(false).is_none(),
                },
                SpinOption {
                    text: names.pm,
                    disabled: self.half(true).is_none(),
                },
            ];
            let text = if pm { names.pm } else { names.am };
            // Named by what it holds: no locale word for the half of the day.
            let label = format!("{}/{}", names.am, names.pm);
            column(
                Column::Meridiem,
                label,
                options,
                place(pm as usize, true),
                text,
                None,
                1,
            )
        });
        let minutes_page = (15 / step).max(1) as usize;
        rsx! {
            div { "data-slot": ChronoPickerPart::Columns.slot(),
                {column(Column::Hours, names.hours_label.to_string(), hours, place(hour_index, true), hour_text, None, 3)}
                span { "data-slot": ChronoPickerPart::Separator.slot(), "aria-hidden": "true", ":" }
                {column(Column::Minutes, names.minutes_label.to_string(), minutes, minutes_at, TWO_DIGITS[minute as usize], Some((names.minutes_value)(minute)), minutes_page)}
                {seconds}
                {meridiem}
            }
        }
    }

    /// The marks the analog face shows for its hand.
    fn marks(self) -> Vec<Mark> {
        let ClockView {
            value,
            base,
            twelve,
            step,
            hand,
            ..
        } = self;
        match hand() {
            Hand::Hour => {
                let outer = (0..12u32).map(|index| {
                    let label = if index == 0 { 12 } else { index };
                    let hour = self.hour_of(label);
                    Mark {
                        index,
                        inner: false,
                        label: TWO_DIGITS[label as usize].trim_start_matches('0'),
                        selected: value.is_some_and(|value| value.hour() == hour),
                        disabled: !self.within(at(hour, 0, 0), at(hour, 59, 59)),
                    }
                });
                // A 24-hour face rings 13 to 00 inside 1 to 12.
                let inner = (0..12u32).filter(|_| !twelve).map(|index| {
                    let hour = if index == 0 { 0 } else { index + 12 };
                    Mark {
                        index,
                        inner: true,
                        label: TWO_DIGITS[hour as usize],
                        selected: value.is_some_and(|value| value.hour() == hour),
                        disabled: !self.within(at(hour, 0, 0), at(hour, 59, 59)),
                    }
                });
                outer.chain(inner).collect()
            }
            Hand::Minute => (0..12u32)
                .map(|index| {
                    let minute = index * 5;
                    Mark {
                        index,
                        inner: false,
                        label: TWO_DIGITS[minute as usize],
                        selected: value.is_some_and(|value| value.minute() == minute),
                        disabled: minute % u32::from(step) != 0
                            || !self
                                .within(at(base.hour(), minute, 0), at(base.hour(), minute, 59)),
                    }
                })
                .collect(),
            Hand::Second => (0..12u32)
                .map(|index| {
                    let second = index * 5;
                    let time = at(base.hour(), base.minute(), second);
                    Mark {
                        index,
                        inner: false,
                        label: TWO_DIGITS[second as usize],
                        selected: value.is_some_and(|value| value.second() == second),
                        disabled: !self.within(time, time),
                    }
                })
                .collect(),
        }
    }

    /// `face` is the face's element, `drag` its pointer handlers.
    pub(super) fn analog_view(self, face: ElementHandle, drag: Drag) -> Element {
        let ClockView {
            names,
            value,
            pm,
            twelve,
            with_seconds,
            mut hand,
            ..
        } = self;
        let tabindex = self.tabindex();
        let marks = self.marks();
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
                Hand::Second => (f64::from(value.second()) * 6.0, 40.0),
            };
            let style = format!(
                "left: 50%; top: 50%; width: 2px; height: {radius}%; transform-origin: 50% 100%; transform: translate(-50%, -100%) rotate({degrees}deg)"
            );
            rsx! {
                div { "data-slot": ChronoPickerPart::Hand.slot(), style }
            }
        });
        let hour_text = value
            .map(|value| {
                let hour = match twelve {
                    true => (value.hour() + 11) % 12 + 1,
                    false => value.hour(),
                };
                TWO_DIGITS[hour as usize]
            })
            .unwrap_or("--");
        let minute_text = value.map_or("--", |value| TWO_DIGITS[value.minute() as usize]);
        let second_text = value.map_or("--", |value| TWO_DIGITS[value.second() as usize]);
        let seconds = with_seconds.then(|| {
            rsx! {
                span { "aria-hidden": "true", ":" }
                button {
                    r#type: "button",
                    "aria-label": "{second_text} {names.seconds_label}",
                    "data-active": (hand() == Hand::Second).then_some("true"),
                    "aria-pressed": if hand() == Hand::Second { "true" } else { "false" },
                    tabindex,
                    onclick: move |_| hand.set(Hand::Second),
                    "{second_text}"
                }
            }
        });
        // Steps finer than the marks get a tick each.
        let step = self.step_of(hand());
        let ticks = (hand() != Hand::Hour && !step.is_multiple_of(5)).then(|| {
            let style = format!("--libero-clock-tick: {}deg", step * 6);
            rsx! {
                div { "data-slot": ChronoPickerPart::Ticks.slot(), style, div {} }
            }
        });
        let halves = twelve.then(|| {
            let (am_at, pm_at) = (self.half(false), self.half(true));
            rsx! {
                button {
                    r#type: "button",
                    "data-active": (value.is_some() && !pm).then_some("true"),
                    "aria-pressed": if value.is_some() && !pm { "true" } else { "false" },
                    disabled: am_at.is_none().then_some(true),
                    tabindex,
                    onclick: move |_| {
                        if let Some(next) = am_at {
                            self.emit(next);
                        }
                    },
                    {names.am}
                }
                button {
                    r#type: "button",
                    "data-active": (value.is_some() && pm).then_some("true"),
                    "aria-pressed": if value.is_some() && pm { "true" } else { "false" },
                    disabled: pm_at.is_none().then_some(true),
                    tabindex,
                    onclick: move |_| {
                        if let Some(next) = pm_at {
                            self.emit(next);
                        }
                    },
                    {names.pm}
                }
            }
        });
        // Minutes and seconds say their unit; an hour reads as itself, plus AM/PM if twelve-hour.
        // Without a value the slider keeps its `aria-valuenow` (required), at `base`, and says so.
        let base = self.base;
        let (face_label, face_text, face_now, face_max) = match hand() {
            Hand::Hour => (
                names.hours_label,
                value.map_or(names.no_time.to_string(), |_| match twelve {
                    true => format!(
                        "{} {}",
                        hour_text.trim_start_matches('0'),
                        if pm { names.pm } else { names.am }
                    ),
                    false => hour_text.to_string(),
                }),
                value.unwrap_or(base).hour(),
                23,
            ),
            Hand::Minute => (
                names.minutes_label,
                value.map_or(names.no_time.to_string(), |value| {
                    (names.minutes_value)(value.minute())
                }),
                value.unwrap_or(base).minute(),
                59,
            ),
            Hand::Second => (
                names.seconds_label,
                value.map_or(names.no_time.to_string(), |value| {
                    (names.seconds_value)(value.second())
                }),
                value.unwrap_or(base).second(),
                59,
            ),
        };
        rsx! {
            div { "data-slot": ChronoPickerPart::Readout.slot(),
                // The shown digits lead each name, so speech input can say them (2.5.3).
                button {
                    r#type: "button",
                    "aria-label": "{hour_text} {names.hours_label}",
                    "data-active": (hand() == Hand::Hour).then_some("true"),
                    "aria-pressed": if hand() == Hand::Hour { "true" } else { "false" },
                    tabindex,
                    onclick: move |_| hand.set(Hand::Hour),
                    "{hour_text}"
                }
                span { "aria-hidden": "true", ":" }
                button {
                    r#type: "button",
                    "aria-label": "{minute_text} {names.minutes_label}",
                    "data-active": (hand() == Hand::Minute).then_some("true"),
                    "aria-pressed": if hand() == Hand::Minute { "true" } else { "false" },
                    tabindex,
                    onclick: move |_| hand.set(Hand::Minute),
                    "{minute_text}"
                }
                {seconds}
                {halves}
            }
            div {
                "data-slot": ChronoPickerPart::Face.slot(),
                role: "slider",
                tabindex,
                "aria-label": face_label,
                "aria-valuetext": face_text,
                "aria-valuenow": face_now,
                "aria-valuemin": 0,
                "aria-valuemax": face_max,
                onmounted: face.mount(),
                onkeydown: move |event| self.face_keydown(event),
                onpointerdown: drag.onpointerdown,
                onpointermove: drag.onpointermove,
                onpointerup: drag.onpointerup,
                onpointercancel: drag.onpointercancel,
                {ticks}
                {pointer}
                div {
                    "data-slot": ChronoPickerPart::Pivot.slot(),
                    style: "left: 50%; top: 50%; width: 6px; height: 6px; border-radius: 50%; transform: translate(-50%, -50%)",
                }
                ClockMarks { marks }
            }
        }
    }
}

/// One mark on the analog face.
#[derive(Clone, Copy, PartialEq)]
struct Mark {
    /// Its place on the ring, 0 at the top.
    index: u32,
    /// On the inner ring of a 24-hour face.
    inner: bool,
    label: &'static str,
    selected: bool,
    disabled: bool,
}

/// The analog face's marks, own scope so a change that moves no mark skips it.
/// Labels only: the face picks by the pointer's angle.
#[component]
fn ClockMarks(marks: Vec<Mark>) -> Element {
    let marks = marks.into_iter().map(|mark| {
        let Mark {
            index,
            inner,
            label,
            selected,
            disabled,
        } = mark;
        let radius = 50.0 * if inner { INNER_RING } else { OUTER_RING };
        let angle = f64::from(index) * std::f64::consts::PI / 6.0;
        let style = format!(
            "left: {:.3}%; top: {:.3}%; transform: translate(-50%, -50%)",
            50.0 + radius * angle.sin(),
            50.0 - radius * angle.cos()
        );
        rsx! {
            span {
                key: "{label}",
                "data-slot": ChronoPickerPart::Mark.slot(),
                "data-selected": selected.then_some("true"),
                "data-disabled": disabled.then_some("true"),
                style,
                "{label}"
            }
        }
    });
    rsx! {
        {marks}
    }
}
