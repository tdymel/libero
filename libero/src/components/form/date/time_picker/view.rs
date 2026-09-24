use dioxus::prelude::*;

use chrono::{NaiveTime, Timelike};

use crate::{
    components::common::has_shortcut_modifier,
    hooks::{Drag, DragMove, DragOptions, DragStart, ElementHandle, use_drag},
    localization::DateLocale,
    platform::ElementApi,
};

/// Which hand an analog picker is setting.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Hand {
    Hour,
    Minute,
    Second,
}

/// Where a mark's centre sits, as a share of the face's half width.
pub(super) const OUTER_RING: f64 = 0.8;
pub(super) const INNER_RING: f64 = 0.52;

/// A column of the digital variant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Column {
    Hours,
    Minutes,
    Seconds,
    Meridiem,
}

impl Column {
    pub(super) const ALL: [Self; 4] = [Self::Hours, Self::Minutes, Self::Seconds, Self::Meridiem];

    pub(super) fn named(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|column| column.name() == name)
    }

    /// Its `data-column`.
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Hours => "Hours",
            Self::Minutes => "Minutes",
            Self::Seconds => "Seconds",
            Self::Meridiem => "Meridiem",
        }
    }
}

/// `00` to `59`, so a pick allocates no label.
pub(super) static TWO_DIGITS: [&str; 60] = [
    "00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13", "14", "15",
    "16", "17", "18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30", "31",
    "32", "33", "34", "35", "36", "37", "38", "39", "40", "41", "42", "43", "44", "45", "46", "47",
    "48", "49", "50", "51", "52", "53", "54", "55", "56", "57", "58", "59",
];

pub(super) fn at(hour: u32, minute: u32, second: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(hour, minute, second).expect("in range")
}

/// What both variants read, for one render.
#[derive(Clone, Copy)]
pub(super) struct ClockView {
    pub(super) names: &'static DateLocale,
    pub(super) value: Option<NaiveTime>,
    pub(super) min: Option<NaiveTime>,
    pub(super) max: Option<NaiveTime>,
    pub(super) onchange: Option<EventHandler<Option<NaiveTime>>>,
    /// What a part picked with no value yet starts from.
    pub(super) base: NaiveTime,
    pub(super) pm: bool,
    pub(super) twelve: bool,
    pub(super) with_seconds: bool,
    pub(super) step: u8,
    pub(super) focusable: bool,
    pub(super) hand: Signal<Hand>,
    pub(super) oncomplete: Option<Callback<()>>,
}

impl ClockView {
    pub(super) fn tabindex(self) -> &'static str {
        if self.focusable { "0" } else { "-1" }
    }

    pub(super) fn within(self, from: NaiveTime, to: NaiveTime) -> bool {
        !(self.min.is_some_and(|min| to < min) || self.max.is_some_and(|max| from > max))
    }

    /// The hour a 12-hour label stands for, in the half of the day `base` is in.
    pub(super) fn hour_of(self, label: u32) -> u32 {
        match self.twelve {
            true => label % 12 + if self.pm { 12 } else { 0 },
            false => label,
        }
    }

    /// The hour label at an index of the hours column.
    pub(super) fn hour_label(self, index: usize) -> u32 {
        match (self.twelve, index) {
            (true, 0) => 12,
            (_, index) => index as u32,
        }
    }

    /// `base` moved into the morning or the afternoon and clamped to `min` and
    /// `max`; `None` when that half lies wholly outside them.
    pub(super) fn half(self, pm: bool) -> Option<NaiveTime> {
        let offset = if pm { 12 } else { 0 };
        if !self.within(at(offset, 0, 0), at(offset + 11, 59, 59)) {
            return None;
        }
        let base = self.base;
        Some(at(base.hour() % 12 + offset, base.minute(), base.second()))
    }

    pub(super) fn clamp(self, time: NaiveTime) -> NaiveTime {
        let time = self.min.map_or(time, |min| time.max(min));
        self.max.map_or(time, |max| time.min(max))
    }

    /// Emits `next` pulled inside `min` and `max`: an hour picked at 9 with
    /// `min` 09:30 keeps `base`'s minute only where it is allowed.
    pub(super) fn emit(self, next: NaiveTime) {
        if let Some(onchange) = &self.onchange {
            onchange.call(Some(self.clamp(next)));
        }
    }

    /// The hour a mark stands for, by its place on the ring.
    pub(super) fn hour_at(self, index: u32, inner: bool) -> u32 {
        match (inner, index) {
            (false, 0) => self.hour_of(12),
            (false, index) => self.hour_of(index),
            (true, 0) => 0,
            (true, index) => index + 12,
        }
    }

    /// The minutes a step of `hand` moves: an hour's are its own.
    pub(super) fn step_of(self, hand: Hand) -> u32 {
        match hand {
            Hand::Minute => u32::from(self.step),
            _ => 1,
        }
    }

    /// The time a point on the face picks: `turn` clockwise from 12 in `0..1`, `reach`
    /// the distance from the centre over the half width. `None` outside `min`/`max`.
    pub(super) fn at_point(self, shown: Hand, turn: f64, reach: f64) -> Option<NaiveTime> {
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
    pub(super) fn after(self, hand: Hand) -> Option<Hand> {
        match hand {
            Hand::Hour => Some(Hand::Minute),
            Hand::Minute if self.with_seconds => Some(Hand::Second),
            _ => None,
        }
    }

    /// On to the next hand, or complete after the last.
    pub(super) fn advance(self) {
        let mut hand = self.hand;
        match (self.after(hand()), self.oncomplete) {
            (Some(next), _) => hand.set(next),
            (None, Some(oncomplete)) => oncomplete.call(()),
            (None, None) => {}
        }
    }

    /// A digital option picked by its column and index.
    pub(super) fn pick(self, column: Column, index: usize) {
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

    /// The face is a slider over its hand: arrows step past what `min`/`max` rule out,
    /// Page keys a quarter face, Home/End the first/last open value, Enter the next hand.
    pub(super) fn face_keydown(self, event: KeyboardEvent) {
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
    pub(super) fn open(self, time: NaiveTime) -> bool {
        let (hand, hour, minute) = (self.hand, time.hour(), time.minute());
        match hand() {
            Hand::Hour => self.within(at(hour, 0, 0), at(hour, 59, 59)),
            Hand::Minute => self.within(at(hour, minute, 0), at(hour, minute, 59)),
            Hand::Second => self.within(time, time),
        }
    }

    /// One step of the hand from `from`, past what `min` and `max` rule out.
    pub(super) fn stepped(self, from: NaiveTime, delta: i64) -> Option<NaiveTime> {
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
pub(super) fn use_face_drag(clock: ClockView, face: ElementHandle) -> Drag {
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
