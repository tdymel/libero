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
    hooks::{ElementHandle, use_element, use_theme},
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
                .background("muted.1"),
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
        let next = at(base.hour() % 12 + offset, base.minute(), base.second());
        let next = self.min.map_or(next, |min| next.max(min));
        Some(self.max.map_or(next, |max| next.min(max)))
    }

    fn emit(self, next: NaiveTime) {
        if let Some(onchange) = &self.onchange {
            onchange.call(Some(next));
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

    /// A mark on the analog face: an hour moves the hand on to the minutes.
    fn pick_mark(self, shown: Hand, inner: bool, index: u32) {
        let mut hand = self.hand;
        match shown {
            Hand::Hour => {
                let hour = match (inner, index) {
                    (false, 0) => self.hour_of(12),
                    (false, index) => self.hour_of(index),
                    (true, 0) => 0,
                    (true, index) => index + 12,
                };
                self.emit(at(hour, self.base.minute(), 0));
                hand.set(Hand::Minute);
            }
            Hand::Minute => self.emit(at(self.base.hour(), index * 5, 0)),
        }
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
        }
    }

    /// The face is a slider over the hand it shows: the arrows step an hour,
    /// or `step` minutes, past what `min` and `max` rule out; Enter moves from
    /// the hour to the minute.
    fn face_keydown(self, event: KeyboardEvent) {
        let mut hand = self.hand;
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
        let step = i64::from(self.step);
        let mut next = self.base;
        for _ in 0..60 {
            let (candidate, open) = match hand() {
                Hand::Hour => {
                    let hour = (i64::from(next.hour()) + delta).rem_euclid(24) as u32;
                    (
                        at(hour, next.minute(), 0),
                        self.within(at(hour, 0, 0), at(hour, 59, 59)),
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
                        self.within(at(next.hour(), minute, 0), at(next.hour(), minute, 59)),
                    )
                }
            };
            next = candidate;
            if open {
                self.emit(next);
                return;
            }
        }
    }

    fn analog_view(self, pick_mark: Callback<(Hand, bool, u32)>) -> Element {
        let ClockView {
            names,
            value,
            pm,
            twelve,
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
                onkeydown: move |event| self.face_keydown(event),
                {pointer}
                div {
                    "data-slot": "pivot",
                    style: "left: 50%; top: 50%; width: 6px; height: 6px; border-radius: 50%; transform: translate(-50%, -50%)",
                }
                ClockMarks { hand: hand(), marks, onpick: pick_mark }
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
    // The same for a mark on the analog face.
    let pick_mark = use_callback(move |(shown, inner, index): (Hand, bool, u32)| {
        clock.pick_mark(shown, inner, index)
    });

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
        TimePickerVariant::Analog => clock.analog_view(pick_mark),
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
/// it.
#[derive(Props, Clone, PartialEq)]
struct ClockMarksProps {
    hand: Hand,
    marks: Vec<Mark>,
    onpick: Callback<(Hand, bool, u32)>,
}

#[component]
fn ClockMarks(props: ClockMarksProps) -> Element {
    let ClockMarksProps {
        hand,
        marks,
        onpick,
    } = props;
    let marks = marks.into_iter().map(|mark| {
        let Mark {
            index,
            inner,
            label,
            selected,
            disabled,
        } = mark;
        let radius = if inner { 26.0 } else { 40.0 };
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
                onclick: move |_| onpick.call((hand, inner, index)),
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
