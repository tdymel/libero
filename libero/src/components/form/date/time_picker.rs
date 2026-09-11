use dioxus::prelude::*;

use chrono::{NaiveTime, Timelike};

use super::{
    calendar::use_focus_after_render,
    date_value::{PickerArgs, PickerOptions, Sealed},
    format::uses_twelve_hours,
    parse_time::MIDNIGHT,
    props::date_props,
};
use crate::{
    components::{
        ClassList, HtmlTag, Input, States,
        common::{focus_ring_sx, input_from_str, inset_focus_ring_sx},
        layout::use_box,
    },
    hooks::{
        Drag, DragMove, DragOptions, DragStart, ElementHandle, use_drag, use_element, use_theme,
    },
    platform::ElementApi,
    sx::{StaticSx, Sx, sx},
    theme::{
        DATE_PICKER_DAY, DATE_PICKER_FONT_SIZE, DateDefaults, DatePickerDefaults, Size, SizeCss,
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
        .hover(sx().background("muted.1"));
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
                .background("muted.1")
                .cursor("pointer")
                // A touch drags the hand rather than scrolling the page.
                .touch_action("none")
                .user_select("none"),
        )
        .selector(
            "& [data-slot='mark']",
            button
                .clone()
                .position("absolute")
                .width(day.clone())
                .height(day)
                .border_radius("50%")
                .padding("0"),
        )
        .selector(
            "& [data-slot='mark'][data-disabled]",
            sx().opacity("0.4")
                .cursor("not-allowed")
                .hover(sx().background("transparent")),
        )
        // One tick per step where the marks are coarser than the step.
        .selector(
            "& [data-slot='ticks']",
            sx().position("absolute")
                .inset("4%")
                .border_radius("50%")
                .background(
                    "repeating-conic-gradient(from -0.5deg, color-mix(in srgb, currentColor 35%, transparent) 0 1deg, transparent 1deg var(--libero-clock-tick))",
                ),
        )
        // The face's own fill over the middle leaves the ticks a thin ring.
        .selector(
            "& [data-slot='ticks'] > div",
            sx().position("absolute")
                .inset("4px")
                .border_radius("50%")
                .background("muted.1"),
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
        // As in `Calendar`: a picked option's fill sets the ring's contrast
        // colour for its inside, so the ring is drawn there. After the ring
        // above, and more specific, so it wins.
        .selector(
            "& [data-selected]:focus-visible",
            inset_focus_ring_sx("-4px"),
        )
});

date_props! {
    picker TimePickerProps(NaiveTime, NaiveTime): clock, limits
}

/// Which hand an analog picker is setting.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Hand {
    Hour,
    Minute,
    Second,
}

/// Where a mark's centre sits, as a share of the face's half width.
const OUTER_RING: f64 = 0.8;
const INNER_RING: f64 = 0.52;

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

/// One option of a digital column. Plain values only: what a click picks is
/// worked out from the column and the index, so the options do not change
/// when another column's value does.
#[derive(Clone, Copy, PartialEq)]
struct Choice {
    /// Also its key: labels are unique within a column.
    label: &'static str,
    selected: bool,
    disabled: bool,
}

/// `00` to `59`, so a pick allocates no label.
static TWO_DIGITS: [&str; 60] = [
    "00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13", "14", "15",
    "16", "17", "18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30", "31",
    "32", "33", "34", "35", "36", "37", "38", "39", "40", "41", "42", "43", "44", "45", "46", "47",
    "48", "49", "50", "51", "52", "53", "54", "55", "56", "57", "58", "59",
];

fn at(hour: u32, minute: u32, second: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(hour, minute, second).expect("in range")
}

/// What both variants read, for one render.
#[derive(Clone, Copy)]
struct ClockView {
    names: &'static DateDefaults,
    value: Option<NaiveTime>,
    min: Option<NaiveTime>,
    max: Option<NaiveTime>,
    onchange: Option<EventHandler<Option<NaiveTime>>>,
    /// What a part picked with no value yet starts from.
    base: NaiveTime,
    pm: bool,
    twelve: bool,
    with_seconds: bool,
    step: u8,
    focusable: bool,
    hand: Signal<Hand>,
}

impl ClockView {
    fn tabindex(self) -> &'static str {
        if self.focusable { "0" } else { "-1" }
    }

    fn within(self, from: NaiveTime, to: NaiveTime) -> bool {
        !(self.min.is_some_and(|min| to < min) || self.max.is_some_and(|max| from > max))
    }

    /// The hour a 12-hour label stands for, in the half of the day `base` is in.
    fn hour_of(self, label: u32) -> u32 {
        match self.twelve {
            true => label % 12 + if self.pm { 12 } else { 0 },
            false => label,
        }
    }

    /// The hour label at an index of the hours column.
    fn hour_label(self, index: usize) -> u32 {
        match (self.twelve, index) {
            (true, 0) => 12,
            (_, index) => index as u32,
        }
    }

    /// `base` moved into the morning or the afternoon and clamped to `min` and
    /// `max`; `None` when that half lies wholly outside them.
    fn half(self, pm: bool) -> Option<NaiveTime> {
        let offset = if pm { 12 } else { 0 };
        if !self.within(at(offset, 0, 0), at(offset + 11, 59, 59)) {
            return None;
        }
        let base = self.base;
        Some(at(base.hour() % 12 + offset, base.minute(), base.second()))
    }

    fn clamp(self, time: NaiveTime) -> NaiveTime {
        let time = self.min.map_or(time, |min| time.max(min));
        self.max.map_or(time, |max| time.min(max))
    }

    /// Emits `next` pulled inside `min` and `max`: an hour picked at 9 with
    /// `min` 09:30 keeps `base`'s minute only where it is allowed.
    fn emit(self, next: NaiveTime) {
        if let Some(onchange) = &self.onchange {
            onchange.call(Some(self.clamp(next)));
        }
    }

    /// The hour a mark stands for, by its place on the ring.
    fn hour_at(self, index: u32, inner: bool) -> u32 {
        match (inner, index) {
            (false, 0) => self.hour_of(12),
            (false, index) => self.hour_of(index),
            (true, 0) => 0,
            (true, index) => index + 12,
        }
    }

    /// The minutes a step of `hand` moves: an hour's are its own.
    fn step_of(self, hand: Hand) -> u32 {
        match hand {
            Hand::Minute => u32::from(self.step),
            _ => 1,
        }
    }

    /// The time a point on the face picks for `shown`: `turn` is clockwise
    /// from 12 in `0..1`, `reach` the distance from the centre over the half
    /// width. `None` where `min` or `max` rule it out.
    fn at_point(self, shown: Hand, turn: f64, reach: f64) -> Option<NaiveTime> {
        let base = self.base;
        let (hour, minute, second) = (base.hour(), base.minute(), base.second());
        match shown {
            Hand::Hour => {
                let inner = !self.twelve && reach < (OUTER_RING + INNER_RING) / 2.0;
                let hour = self.hour_at((turn * 12.0).round() as u32 % 12, inner);
                self.within(at(hour, 0, 0), at(hour, 59, 59))
                    .then(|| at(hour, minute, second))
            }
            Hand::Minute | Hand::Second => {
                let step = f64::from(self.step_of(shown));
                // The nearest step, or 0 from the other side of 12 when a
                // step does not divide the hour.
                let minutes = turn * 60.0;
                let near = (minutes / step).round() * step;
                let snapped = match near >= 60.0 || 60.0 - minutes < (minutes - near).abs() {
                    true => 0,
                    false => near as u32,
                };
                let (from, to) = match shown {
                    Hand::Minute => (at(hour, snapped, 0), at(hour, snapped, 59)),
                    _ => (at(hour, minute, snapped), at(hour, minute, snapped)),
                };
                self.within(from, to).then_some(match shown {
                    Hand::Minute => at(hour, snapped, second),
                    _ => to,
                })
            }
        }
    }

    /// The hand an analog pick moves on to, if any.
    fn after(self, hand: Hand) -> Option<Hand> {
        match hand {
            Hand::Hour => Some(Hand::Minute),
            Hand::Minute if self.with_seconds => Some(Hand::Second),
            _ => None,
        }
    }

    /// A digital option picked by its column and index.
    fn pick(self, column: Column, index: usize) {
        let base = self.base;
        let (hour, minute, second) = (base.hour(), base.minute(), base.second());
        let next = match column {
            Column::Hours => at(self.hour_of(self.hour_label(index)), minute, second),
            Column::Minutes => at(hour, index as u32 * u32::from(self.step), second),
            Column::Seconds => at(hour, minute, index as u32),
            Column::Meridiem => match self.half(index == 1) {
                Some(next) => next,
                None => return,
            },
        };
        self.emit(next);
    }

    /// `handles` are the hours, minutes, seconds and meridiem columns.
    fn digital_view(
        self,
        handles: [ElementHandle; 4],
        active: Signal<Option<(Column, usize)>>,
        focus_request: Signal<Option<String>>,
        pick: Callback<(Column, usize)>,
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
        let [
            hours_column,
            minutes_column,
            seconds_column,
            meridiem_column,
        ] = handles;
        let column = move |column: Column,
                           label: Option<&'static str>,
                           handle: ElementHandle,
                           choices: Vec<Choice>| {
            rsx! {
                ClockColumn {
                    column,
                    label,
                    handle,
                    choices,
                    focusable,
                    active,
                    focus_request,
                    onpick: pick,
                }
            }
        };
        let hours = (0..if twelve { 12 } else { 24 })
            .map(|index| {
                let label = self.hour_label(index);
                let hour = self.hour_of(label);
                Choice {
                    label: TWO_DIGITS[label as usize],
                    selected: value.is_some_and(|value| value.hour() == hour),
                    disabled: !self.within(at(hour, 0, 0), at(hour, 59, 59)),
                }
            })
            .collect();
        let minutes = (0..60u32)
            .step_by(step as usize)
            .map(|minute| Choice {
                label: TWO_DIGITS[minute as usize],
                selected: value.is_some_and(|value| value.minute() == minute),
                disabled: !self.within(at(base.hour(), minute, 0), at(base.hour(), minute, 59)),
            })
            .collect();
        let seconds = with_seconds.then(|| {
            let choices = (0..60u32)
                .map(|second| {
                    let time = at(base.hour(), base.minute(), second);
                    Choice {
                        label: TWO_DIGITS[second as usize],
                        selected: value.is_some_and(|value| value.second() == second),
                        disabled: !self.within(time, time),
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
                    label: names.am,
                    selected: value.is_some() && !pm,
                    disabled: self.half(false).is_none(),
                },
                Choice {
                    label: names.pm,
                    selected: value.is_some() && pm,
                    disabled: self.half(true).is_none(),
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

    /// The face is a slider over the hand it shows: the arrows step an hour,
    /// `step` minutes or a second, past what `min` and `max` rule out; Enter
    /// moves on to the next hand.
    fn face_keydown(self, event: KeyboardEvent) {
        let mut hand = self.hand;
        let delta: i64 = match event.key() {
            Key::ArrowUp | Key::ArrowRight => 1,
            Key::ArrowDown | Key::ArrowLeft => -1,
            Key::Enter => {
                event.prevent_default();
                if let Some(next) = self.after(hand()) {
                    hand.set(next);
                }
                return;
            }
            _ => return,
        };
        event.prevent_default();
        let step = i64::from(self.step_of(hand()));
        // Off the step, the first press lands on it.
        let snap = |value: u32| {
            let value = i64::from(value);
            let snapped = match delta > 0 {
                true => (value / step + 1) * step,
                false => (value + step - 1) / step * step - step,
            };
            // Round the hour onto a step, also one that does not divide it.
            match snapped {
                60.. => 0,
                ..0 => (59 / step * step) as u32,
                _ => snapped as u32,
            }
        };
        let mut next = self.base;
        for _ in 0..60 {
            let (hour, minute, second) = (next.hour(), next.minute(), next.second());
            let (candidate, open) = match hand() {
                Hand::Hour => {
                    let hour = (i64::from(hour) + delta).rem_euclid(24) as u32;
                    (
                        at(hour, minute, second),
                        self.within(at(hour, 0, 0), at(hour, 59, 59)),
                    )
                }
                Hand::Minute => {
                    let minute = snap(minute);
                    (
                        at(hour, minute, second),
                        self.within(at(hour, minute, 0), at(hour, minute, 59)),
                    )
                }
                Hand::Second => {
                    let time = at(hour, minute, snap(second));
                    (time, self.within(time, time))
                }
            };
            next = candidate;
            if open {
                self.emit(next);
                return;
            }
        }
    }

    /// `face` is the face's element, `drag` its pointer handlers.
    fn analog_view(self, face: ElementHandle, drag: Drag) -> Element {
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
                div { "data-slot": "hand", style }
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
                span { ":" }
                button {
                    r#type: "button",
                    "aria-label": names.seconds_label,
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
                div { "data-slot": "ticks", style, div {} }
            }
        });
        let halves = twelve.then(|| {
            let (am_at, pm_at) = (self.half(false), self.half(true));
            rsx! {
                button {
                    r#type: "button",
                    "data-active": (value.is_some() && !pm).then_some("true"),
                    "aria-pressed": if value.is_some() && !pm { "true" } else { "false" },
                    disabled: am_at.is_none(),
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
                    disabled: pm_at.is_none(),
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
        let (face_label, face_text, face_now, face_max) = match hand() {
            Hand::Hour => (
                names.hours_label,
                hour_text,
                value.map(|value| value.hour()),
                23,
            ),
            Hand::Minute => (
                names.minutes_label,
                minute_text,
                value.map(|value| value.minute()),
                59,
            ),
            Hand::Second => (
                names.seconds_label,
                second_text,
                value.map(|value| value.second()),
                59,
            ),
        };
        rsx! {
            div { "data-slot": "readout",
                button {
                    r#type: "button",
                    "aria-label": names.hours_label,
                    "data-active": (hand() == Hand::Hour).then_some("true"),
                    "aria-pressed": if hand() == Hand::Hour { "true" } else { "false" },
                    tabindex,
                    onclick: move |_| hand.set(Hand::Hour),
                    "{hour_text}"
                }
                span { ":" }
                button {
                    r#type: "button",
                    "aria-label": names.minutes_label,
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
                "data-slot": "face",
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
                    "data-slot": "pivot",
                    style: "left: 50%; top: 50%; width: 6px; height: 6px; border-radius: 50%; transform: translate(-50%, -50%)",
                }
                ClockMarks { marks }
            }
        }
    }
}

#[component]
pub(super) fn Clock(props: ClockProps) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.time_picker.size);
    let value = props.value;
    let base = value.or(props.min).unwrap_or(MIDNIGHT);

    let hand = use_signal(|| Hand::Hour);
    let clock = ClockView {
        names: &theme.date,
        value,
        min: props.min,
        max: props.max,
        onchange: props.onchange,
        base,
        pm: base.hour() >= 12,
        twelve: props.twelve_hour,
        with_seconds: props.with_seconds,
        step: props.step.unwrap_or(theme.time_picker.step).clamp(1, 30),
        focusable: props.focusable,
        hand,
    };
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
    // The digital option the keyboard is on.
    let active = use_signal(|| None::<(Column, usize)>);
    let focus_request = use_focus_after_render(root);

    // One identity across renders, so the columns' props compare equal and a
    // pick in one column skips the others.
    let pick = use_callback(move |(column, index): (Column, usize)| clock.pick(column, index));
    let face = use_element();
    let drag = use_face_drag(clock, face);

    let body = match props.variant {
        TimePickerVariant::Digital => clock.digital_view(
            [
                hours_column,
                minutes_column,
                seconds_column,
                meridiem_column,
            ],
            active,
            focus_request,
            pick,
        ),
        TimePickerVariant::Analog => clock.analog_view(face, drag),
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

/// One press on the analog face.
#[derive(Clone, Copy)]
struct Press {
    /// The face's centre and half width, once measured.
    centre: Option<(f64, f64, f64)>,
    picked: bool,
    released: bool,
}

/// Pointer picks on the analog face: a press or a drag sets the hand shown to
/// the pointer's angle, and the release moves on to the next hand.
fn use_face_drag(clock: ClockView, face: ElementHandle) -> Drag {
    let mut press = use_hook(|| CopyValue::new(None::<Press>));
    let pick = move |clock: ClockView, x: f64, y: f64| {
        let Some(Press {
            centre: Some((cx, cy, half)),
            ..
        }) = press.cloned()
        else {
            return;
        };
        let (dx, dy) = (x - cx, y - cy);
        let turn = (dx.atan2(-dy) / std::f64::consts::TAU).rem_euclid(1.0);
        let Some(next) = clock.at_point((clock.hand)(), turn, dx.hypot(dy) / half) else {
            return;
        };
        if let Some(press) = { press }.write().as_mut() {
            press.picked = true;
        }
        if clock.value != Some(clock.clamp(next)) {
            clock.emit(next);
        }
    };
    let finish = move |clock: ClockView| {
        let picked = press.cloned().is_some_and(|press| press.picked);
        { press }.set(None);
        let mut hand = clock.hand;
        if let Some(next) = clock.after(hand()).filter(|_| picked) {
            hand.set(next);
        }
    };
    let onstart = use_callback(move |start: DragStart| {
        // Started here, awaited in the task: see `platform::Read`.
        let (offset, size) = (face.client_offset(), face.dimensions());
        press.set(Some(Press {
            centre: None,
            picked: false,
            released: false,
        }));
        spawn(async move {
            let (Ok((left, top)), Ok(size)) = (offset.await, size.await) else {
                press.set(None);
                start.cancel.call(());
                return;
            };
            let half = size.width / 2.0;
            let Some(released) = press.write().as_mut().map(|press| {
                press.centre = Some((left + half, top + size.height / 2.0, half));
                press.released
            }) else {
                return;
            };
            pick(clock, start.client.x, start.client.y);
            if released {
                finish(clock);
            }
        });
    });
    let onmove = use_callback(move |step: DragMove| pick(clock, step.client.x, step.client.y));
    let onend = use_callback(move |()| {
        let measured = press.cloned().is_some_and(|press| press.centre.is_some());
        match measured {
            true => finish(clock),
            false => {
                if let Some(press) = press.write().as_mut() {
                    press.released = true;
                }
            }
        }
    });
    use_drag(DragOptions {
        capture: face,
        onstart,
        onmove,
        onend,
    })
}

/// A column of the digital variant. Its own scope: a pick in another column
/// leaves its props equal, so it skips the re-render.
#[derive(Props, Clone, PartialEq)]
struct ClockColumnProps {
    column: Column,
    #[props(!optional)]
    label: Option<&'static str>,
    handle: ElementHandle,
    choices: Vec<Choice>,
    focusable: bool,
    /// The option the keyboard is on, in whichever column.
    active: Signal<Option<(Column, usize)>>,
    /// A selector the clock focuses after the next render.
    focus_request: Signal<Option<String>>,
    onpick: Callback<(Column, usize)>,
}

/// One tab stop: the option the keyboard is on, else the picked one, else the
/// first enabled. Up and Down, Home and End move within the column.
#[component]
fn ClockColumn(props: ClockColumnProps) -> Element {
    let ClockColumnProps {
        column,
        label,
        handle,
        choices,
        focusable,
        mut active,
        mut focus_request,
        onpick,
    } = props;
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
        let Some(at) = stop.and_then(|stop| enabled.iter().position(|index| *index == stop)) else {
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
    rsx! {
        div {
            "data-slot": "column",
            "data-column": "{column:?}",
            "aria-label": label,
            onmounted: handle.mount(),
            onkeydown,
            for (index, choice) in choices.into_iter().enumerate() {
                button {
                    key: "{choice.label}",
                    r#type: "button",
                    "data-slot": "option",
                    "data-index": index as i64,
                    "data-selected": choice.selected.then_some("true"),
                    "aria-pressed": if choice.selected { "true" } else { "false" },
                    disabled: choice.disabled,
                    tabindex: if focusable && stop == Some(index) { "0" } else { "-1" },
                    onfocus: move |_| active.set(Some((column, index))),
                    onclick: move |_| onpick.call((column, index)),
                    "{choice.label}"
                }
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

/// The marks on the analog face. Its own scope with plain values, so a value
/// change that moves no mark - a minute, while the face shows hours - skips
/// it. Labels only: the face picks by the pointer's angle.
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
                "data-slot": "mark",
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
