use dioxus::prelude::*;

use chrono::{NaiveTime, Timelike};

use super::{
    date_value::{PickerArgs, PickerOptions, Sealed},
    format::uses_twelve_hours,
    parse_time::MIDNIGHT,
    props::date_props,
    spin_column::{SpinAt, SpinColumn, SpinOption},
};
use crate::{
    components::{
        ClassList, HtmlTag, Input, States,
        common::{
            borderless_on_state_sx, disabled_look_sx, focus_ring_sx, has_shortcut_modifier,
            input_from_str, inset_focus_ring_sx,
        },
        layout::use_box,
    },
    hooks::{
        Drag, DragMove, DragOptions, DragStart, ElementHandle, use_drag, use_element, use_formats,
        use_localization, use_theme,
    },
    localization::DateLocale,
    platform::ElementApi,
    sx::{FORCED_COLORS, StaticSx, Sx, sx},
    theme::{
        DATE_PICKER_DAY, DATE_PICKER_FONT_SIZE, DatePickerDefaults, Size, SizeCss,
        TimePickerVariant,
    },
};

input_from_str!(TimePickerVariant);

pub(super) static TIME_PICKER_SX: StaticSx = StaticSx::new(|| {
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
        // A digital clock: `HH:MM`, each number between its faded neighbours.
        .selector(
            "& > [data-slot='columns']",
            sx().display("flex")
                .align_items("center")
                .justify_content("center")
                // Room for the focus ring, clear of the separators and an edge.
                .gap("6px")
                .padding("4px")
                .with("font-variant-numeric", "tabular-nums")
                .white_space("nowrap"),
        )
        .selector(
            "& [data-slot='spin']",
            sx().display("flex")
                .flex_direction("column")
                .align_items("center")
                .min_width(format!("calc(1.25 * {day})"))
                .padding("2px 6px")
                .border_radius(SizeCss::RADIUS.value(Size::Sm))
                .line_height("1.25")
                .cursor("ns-resize")
                // A touch drags the column rather than scrolling the page.
                .touch_action("none")
                .user_select("none")
                .hover(sx().background("muted.1")),
        )
        .selector(
            "& [data-slot='spin'] > [data-slot='value']",
            sx().font_size("1.75em").font_weight("500"),
        )
        .selector(
            "& [data-slot='spin'] > [data-slot='neighbour']",
            sx().min_height("1.25em").color("text-dimmed").cursor("pointer"),
        )
        .selector("& [data-slot='spin']:focus-visible", focus_ring_sx())
        .selector(
            "& [data-slot='spin']:focus-visible > [data-slot='value']",
            sx().color("primary.6"),
        )
        .selector(
            "& [data-slot='separator']",
            sx().font_size("1.75em").font_weight("500").color("text-dimmed"),
        )
        // A duration's unit after each column: `h`, `min`.
        .selector(
            "& [data-slot='unit']",
            sx().color("text-dimmed"),
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
        // The house on-state ring marks the hand being set (todo 744).
        .selector(
            "& [data-slot='readout'] [data-active]",
            borderless_on_state_sx().focus_visible(focus_ring_sx()),
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
            disabled_look_sx("not-allowed").hover(sx().background("transparent")),
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
            disabled_look_sx("not-allowed").hover(sx().background("transparent")),
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
        // Forced colours paint every fill `Canvas`: the picks and the hand
        // itself would vanish, as in `Calendar`.
        .media(
            FORCED_COLORS,
            sx().selector(
                "& [data-selected]",
                sx().background("Highlight")
                    .color("HighlightText")
                    .hover(sx().background("Highlight")),
            )
            .selector(
                "& [data-slot='hand'], & [data-slot='pivot']",
                sx().background("CanvasText"),
            ),
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
    let time_format = use_formats().time;
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
                .unwrap_or_else(|| uses_twelve_hours(time_format)),
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
    /// Called once the last part is set: the last hand picked, or Enter on
    /// the last column. A date-time range moves on to its next step.
    #[props(default)]
    oncomplete: Option<Callback<()>>,
}

/// A column of the digital variant.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Column {
    Hours,
    Minutes,
    Seconds,
    Meridiem,
}

impl Column {
    const ALL: [Self; 4] = [Self::Hours, Self::Minutes, Self::Seconds, Self::Meridiem];

    fn named(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|column| column.name() == name)
    }

    /// Its `data-column`.
    fn name(self) -> &'static str {
        match self {
            Self::Hours => "Hours",
            Self::Minutes => "Minutes",
            Self::Seconds => "Seconds",
            Self::Meridiem => "Meridiem",
        }
    }
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
    names: &'static DateLocale,
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
    oncomplete: Option<Callback<()>>,
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

    /// On to the next hand, or complete after the last.
    fn advance(self) {
        let mut hand = self.hand;
        match (self.after(hand()), self.oncomplete) {
            (Some(next), _) => hand.set(next),
            (None, Some(oncomplete)) => oncomplete.call(()),
            (None, None) => {}
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

    /// A digital clock: a spinbutton each for the hours, the minutes, the
    /// seconds and AM/PM. `pick` gets a column's picked index, `ondone` the
    /// column Enter or typing has finished.
    fn digital_view(
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
                           page: usize| {
            rsx! {
                SpinColumn {
                    column: column.name(),
                    label,
                    options,
                    at,
                    text: if value.is_some() { text } else { "--" },
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
                span { "data-slot": "separator", "aria-hidden": "true", ":" }
                {column(Column::Seconds, names.seconds_label.to_string(), options, place(second, true), TWO_DIGITS[second], 15)}
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
                1,
            )
        });
        let minutes_page = (15 / step).max(1) as usize;
        rsx! {
            div { "data-slot": "columns",
                {column(Column::Hours, names.hours_label.to_string(), hours, place(hour_index, true), hour_text, 3)}
                span { "data-slot": "separator", "aria-hidden": "true", ":" }
                {column(Column::Minutes, names.minutes_label.to_string(), minutes, minutes_at, TWO_DIGITS[minute as usize], minutes_page)}
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
    /// `step` minutes or a second, past what `min` and `max` rule out; Page
    /// Up/Down a quarter of the face, Home/End its first and last open value;
    /// Enter moves on to the next hand.
    fn face_keydown(self, event: KeyboardEvent) {
        // Ctrl/Alt/Meta chords are the browser's.
        if has_shortcut_modifier(&event) {
            return;
        }
        let hand = self.hand;
        let step = self.step_of(hand());
        let page = match hand() {
            Hand::Hour => 3,
            _ => (15 / step).max(1),
        };
        let next = match event.key() {
            Key::ArrowUp | Key::ArrowRight => self.stepped(self.base, 1),
            Key::ArrowDown | Key::ArrowLeft => self.stepped(self.base, -1),
            Key::PageUp => (0..page).try_fold(self.base, |from, _| self.stepped(from, 1)),
            Key::PageDown => (0..page).try_fold(self.base, |from, _| self.stepped(from, -1)),
            Key::Home | Key::End => {
                let last = event.key() == Key::End;
                let (hour, minute, second) =
                    (self.base.hour(), self.base.minute(), self.base.second());
                let edge = match (hand(), last) {
                    (Hand::Hour, false) => at(0, minute, second),
                    (Hand::Hour, true) => at(23, minute, second),
                    (Hand::Minute, false) => at(hour, 0, second),
                    (Hand::Minute, true) => at(hour, 59 / step * step, second),
                    (Hand::Second, false) => at(hour, minute, 0),
                    (Hand::Second, true) => at(hour, minute, 59),
                };
                match self.open(edge) {
                    true => Some(edge),
                    false => self.stepped(edge, if last { -1 } else { 1 }),
                }
            }
            Key::Enter => {
                event.prevent_default();
                self.advance();
                return;
            }
            _ => return,
        };
        event.prevent_default();
        if let Some(next) = next {
            self.emit(next);
        }
    }

    /// Whether the hand's value at `time` is one `min` and `max` allow.
    fn open(self, time: NaiveTime) -> bool {
        let (hand, hour, minute) = (self.hand, time.hour(), time.minute());
        match hand() {
            Hand::Hour => self.within(at(hour, 0, 0), at(hour, 59, 59)),
            Hand::Minute => self.within(at(hour, minute, 0), at(hour, minute, 59)),
            Hand::Second => self.within(time, time),
        }
    }

    /// One step of the hand from `from`, past what `min` and `max` rule out.
    fn stepped(self, from: NaiveTime, delta: i64) -> Option<NaiveTime> {
        let hand = self.hand;
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
        let mut next = from;
        for _ in 0..60 {
            let (hour, minute, second) = (next.hour(), next.minute(), next.second());
            next = match hand() {
                Hand::Hour => at(
                    (i64::from(hour) + delta).rem_euclid(24) as u32,
                    minute,
                    second,
                ),
                Hand::Minute => at(hour, snap(minute), second),
                Hand::Second => at(hour, minute, snap(second)),
            };
            if self.open(next) {
                return Some(next);
            }
        }
        None
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
                span { ":" }
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
        names: &use_localization().date,
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
        oncomplete: props.oncomplete,
    };
    let root = use_element();

    // One identity across renders, so the columns' props compare equal and a
    // pick in one column skips the others.
    let pick = use_callback(move |(name, index): (&'static str, usize)| {
        if let Some(column) = Column::named(name) {
            clock.pick(column, index);
        }
    });
    // Enter, or typing that fills a column, moves on to the next, as a native
    // time input; after the last the time is complete.
    let oncomplete = props.oncomplete;
    let ondone = use_callback(move |name: &'static str| {
        let later = Column::ALL
            .into_iter()
            .skip_while(|column| column.name() != name);
        let next = later.skip(1).find_map(|column| {
            root.query_selector(&format!("[data-column='{}']", column.name()))
                .ok()
        });
        match (next, oncomplete) {
            (Some(next), _) => {
                let _ = next.focus();
            }
            (None, Some(oncomplete)) => oncomplete.call(()),
            (None, None) => {}
        }
    });
    let face = use_element();
    let drag = use_face_drag(clock, face);

    let body = match props.variant {
        TimePickerVariant::Digital => clock.digital_view(pick, ondone),
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
        if picked {
            clock.advance();
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
