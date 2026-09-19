//! A span of time, `chrono::TimeDelta`, as a date field's value: shown as
//! `1 h 30 min`, posted as ISO 8601 (`PT1H30M`), picked on the digital clock's
//! columns - hours, minutes and optional seconds, each a spinbutton.

use std::sync::LazyLock;

use chrono::{NaiveDate, TimeDelta};
use dioxus::prelude::*;

use super::{
    parse::Unreadable,
    picker_field::{FieldValue, Formats},
    spin_column::{SpinAt, SpinColumn, SpinOption},
    time_picker::TIME_PICKER_SX,
};
use crate::{
    components::{
        common::{ClassList, HtmlTag, Input, States},
        layout::use_box,
    },
    hooks::{use_element, use_localization, use_theme},
    localization::{DateLocale, fill},
    platform::ElementApi,
    sx::Sx,
    theme::Size,
};

/// The top of the hours column without a `max`: 99 h 59 min 59 s.
const DEFAULT_MAX: TimeDelta = TimeDelta::seconds(99 * 3600 + 59 * 60 + 59);
/// The hours column stops here, whatever `max` says.
const MAX_HOURS: i64 = 999;

/// `00` to `999`, so a pick allocates no label.
static NUMBERS: LazyLock<Vec<String>> = LazyLock::new(|| {
    (0..=MAX_HOURS)
        .map(|number| format!("{number:02}"))
        .collect()
});

/// The columns, by their `data-column`, in order.
const COLUMNS: [&str; 3] = ["Hours", "Minutes", "Seconds"];

/// `min` and `max` with their defaults: nothing below zero, nothing above
/// 99 h 59 min 59 s.
pub(super) fn bounds(min: Option<TimeDelta>, max: Option<TimeDelta>) -> (TimeDelta, TimeDelta) {
    let min = min.unwrap_or_default().max(TimeDelta::zero());
    (min, max.unwrap_or(DEFAULT_MAX).max(min))
}

/// Whole hours, minutes and seconds of a span that is not negative.
fn parts(span: TimeDelta) -> (i64, i64, i64) {
    let seconds = span.num_seconds();
    (seconds / 3600, seconds / 60 % 60, seconds % 60)
}

fn of(hours: i64, minutes: i64, seconds: i64) -> TimeDelta {
    TimeDelta::seconds(hours * 3600 + minutes * 60 + seconds)
}

/// `1 h 30 min`: the parts that are not zero, `0 min` for none.
fn show_duration(span: TimeDelta, names: &DateLocale) -> String {
    let sign = if span < TimeDelta::zero() { "-" } else { "" };
    let (hours, minutes, seconds) = parts(span.abs());
    let mut shown = Vec::new();
    if hours > 0 {
        shown.push(format!("{hours} {}", names.hours_short));
    }
    if minutes > 0 || hours == 0 && seconds == 0 {
        shown.push(format!("{minutes} {}", names.minutes_short));
    }
    if seconds > 0 {
        shown.push(format!("{seconds} {}", names.seconds_short));
    }
    format!("{sign}{}", shown.join(" "))
}

/// Lower case, without the dots of an abbreviation (`Std.`).
fn bare(word: &str) -> String {
    word.to_lowercase().replace('.', "")
}

/// Which part a unit word names: 0 hours, 1 minutes, 2 seconds. The locale's
/// words and English's; a word of two letters or more may be cut short.
fn unit_of(word: &str, names: &DateLocale) -> Option<usize> {
    let words = [
        [
            names.hours_short,
            names.hours_label,
            "h",
            "hr",
            "hrs",
            "hour",
            "hours",
        ],
        [
            names.minutes_short,
            names.minutes_label,
            "m",
            "min",
            "mins",
            "minute",
            "minutes",
        ],
        [
            names.seconds_short,
            names.seconds_label,
            "s",
            "sec",
            "secs",
            "second",
            "seconds",
        ],
    ];
    let matching = |matches: &dyn Fn(&str) -> bool| -> Vec<usize> {
        (0..3)
            .filter(|&unit| words[unit].iter().any(|own| matches(&bare(own))))
            .collect()
    };
    let exact = matching(&|own| own == word);
    if let [unit] = exact[..] {
        return Some(unit);
    }
    match matching(&|own| word.chars().count() >= 2 && own.starts_with(word))[..] {
        [unit] => Some(unit),
        _ => None,
    }
}

/// Typed text back: `1 h 30 min`, `1h30`, `90 min`, `1:30`, `1:30:15`, or a
/// bare number of minutes. A number after a unit counts in the next smaller
/// one. Nothing negative.
fn read_duration(text: &str, names: &DateLocale) -> Result<TimeDelta, Unreadable> {
    let text = bare(text.trim());
    if text.contains(':') {
        let numbers = text
            .split(':')
            .map(|part| part.trim().parse::<u32>().map(i64::from))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| Unreadable)?;
        return match numbers[..] {
            [hours, minutes] if minutes < 60 => Ok(of(hours, minutes, 0)),
            [hours, minutes, seconds] if minutes < 60 && seconds < 60 => {
                Ok(of(hours, minutes, seconds))
            }
            _ => Err(Unreadable),
        };
    }
    let mut seen: [Option<i64>; 3] = [None; 3];
    let mut last = None;
    let mut rest = text.as_str();
    loop {
        rest = rest.trim_start_matches(|c: char| c.is_whitespace() || c == ',');
        if rest.is_empty() {
            break;
        }
        let digits = rest
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(rest.len());
        let number = rest[..digits].parse::<u32>().map_err(|_| Unreadable)?;
        rest = rest[digits..].trim_start();
        let letters = rest
            .find(|c: char| !c.is_alphabetic())
            .unwrap_or(rest.len());
        let unit = match (&rest[..letters], last) {
            ("", None) => 1,
            ("", Some(unit)) if unit < 2 => unit + 1,
            ("", Some(_)) => return Err(Unreadable),
            (word, _) => unit_of(word, names).ok_or(Unreadable)?,
        };
        rest = &rest[letters..];
        if seen[unit].replace(i64::from(number)).is_some() {
            return Err(Unreadable);
        }
        last = Some(unit);
    }
    match seen {
        [None, None, None] => Err(Unreadable),
        [hours, minutes, seconds] => Ok(of(
            hours.unwrap_or(0),
            minutes.unwrap_or(0),
            seconds.unwrap_or(0),
        )),
    }
}

/// The error for a span outside `min` and `max`, naming the bound it missed:
/// a duration always has both, and "between" would name one never set.
pub(super) fn duration_refusal(
    span: TimeDelta,
    min: Option<TimeDelta>,
    max: Option<TimeDelta>,
    formats: &Formats,
) -> String {
    let names = formats.names;
    match (min, max) {
        (Some(min), _) if span < min => {
            fill(names.duration_at_least, &[("min", &min.show(formats))])
        }
        (_, Some(max)) if span > max => {
            fill(names.duration_at_most, &[("max", &max.show(formats))])
        }
        _ => names.invalid_duration.to_string(),
    }
}

/// ISO 8601: `PT1H30M`, `PT0S` for none.
fn iso_duration(span: TimeDelta) -> String {
    let sign = if span < TimeDelta::zero() { "-" } else { "" };
    let (hours, minutes, seconds) = parts(span.abs());
    if (hours, minutes, seconds) == (0, 0, 0) {
        return "PT0S".to_string();
    }
    let mut iso = format!("{sign}PT");
    for (part, unit) in [(hours, 'H'), (minutes, 'M'), (seconds, 'S')] {
        if part > 0 {
            iso.push_str(&format!("{part}{unit}"));
        }
    }
    iso
}

impl FieldValue for TimeDelta {
    fn show(self, formats: &Formats) -> String {
        show_duration(self, formats.names)
    }

    fn read(
        text: &str,
        formats: &Formats,
        _: Option<Self>,
        _: Option<NaiveDate>,
    ) -> Result<Self, Unreadable> {
        read_duration(text, formats.names)
    }

    fn iso(self) -> String {
        iso_duration(self)
    }

    fn dialog_label(names: &DateLocale) -> &'static str {
        names.duration_label
    }

    fn unreadable(names: &DateLocale) -> &'static str {
        names.invalid_duration
    }
}

/// What `ChronoPicker::<TimeDelta>` draws, with every option resolved.
#[derive(Props, Clone, PartialEq)]
pub(super) struct DurationClockProps {
    value: Option<TimeDelta>,
    onchange: Option<EventHandler<Option<TimeDelta>>>,
    with_seconds: bool,
    step: Option<u8>,
    min: Option<TimeDelta>,
    max: Option<TimeDelta>,
    size: Input<Size>,
    focusable: bool,
    name: Option<String>,
    class: Input<ClassList>,
    sx: Input<Sx>,
    states: Input<States>,
    attributes: Vec<Attribute>,
}

/// What a render reads, for the picks.
#[derive(Clone, Copy)]
struct DurationView {
    min: TimeDelta,
    max: TimeDelta,
    /// The value inside the bounds, or `min` with none yet.
    base: TimeDelta,
    step: i64,
    onchange: Option<EventHandler<Option<TimeDelta>>>,
}

impl DurationView {
    fn within(self, from: TimeDelta, to: TimeDelta) -> bool {
        !(to < self.min || from > self.max)
    }

    /// One column's option picked: the other parts stay, nothing carries.
    fn pick(self, column: &str, index: usize) {
        let (hours, minutes, seconds) = parts(self.base);
        let index = index as i64;
        let next = match column {
            "Hours" => of(index, minutes, seconds),
            "Minutes" => of(hours, index * self.step, seconds),
            _ => of(hours, minutes, index),
        };
        if let Some(onchange) = &self.onchange {
            onchange.call(Some(next.clamp(self.min, self.max)));
        }
    }
}

#[component]
pub(super) fn DurationClock(props: DurationClockProps) -> Element {
    let theme = use_theme();
    let names = &use_localization().date;
    let size = props.size.copied_or(theme.time_picker.size);
    let (value, focusable) = (props.value, props.focusable);
    let (min, max) = bounds(props.min, props.max);
    let base = value.map_or(min, |value| value.clamp(min, max));
    let view = DurationView {
        min,
        max,
        base,
        step: i64::from(props.step.unwrap_or(theme.time_picker.step).clamp(1, 30)),
        onchange: props.onchange,
    };
    let root = use_element();

    // One identity across renders, so a pick in one column skips the others.
    let pick = use_callback(move |(column, index): (&'static str, usize)| view.pick(column, index));
    // Enter, or typing that fills a column, moves on to the next.
    let ondone = use_callback(move |column: &'static str| {
        let next = COLUMNS
            .into_iter()
            .skip_while(|name| *name != column)
            .skip(1)
            .find_map(|name| root.query_selector(&format!("[data-column='{name}']")).ok());
        if let Some(next) = next {
            let _ = next.focus();
        }
    });

    let (hours, minutes, seconds) = parts(base);
    let step = view.step;
    let place = |index: i64, exact: bool| match (value, exact) {
        (None, _) => SpinAt::Empty(index as usize),
        (Some(_), true) => SpinAt::At(index as usize),
        (Some(_), false) => SpinAt::Past(index as usize),
    };
    let column = |name: &'static str,
                  label: &str,
                  unit: &str,
                  spoken: fn(u32) -> String,
                  options: Vec<SpinOption>,
                  at: SpinAt,
                  shown: i64,
                  wrap: bool,
                  page: usize| {
        rsx! {
            SpinColumn {
                column: name,
                label: label.to_string(),
                options,
                at,
                text: if value.is_some() { NUMBERS[shown as usize].as_str() } else { "--" },
                valuetext: spoken(shown as u32),
                wrap,
                page,
                focusable,
                onpick: pick,
                ondone,
            }
            span { "data-slot": "unit", "aria-hidden": "true", "{unit}" }
        }
    };
    let hour_options = (0..=parts(max).0.min(MAX_HOURS))
        .map(|hour| SpinOption {
            text: NUMBERS[hour as usize].as_str(),
            disabled: !view.within(of(hour, 0, 0), of(hour, 59, 59)),
        })
        .collect();
    let minute_options = (0..60)
        .step_by(step as usize)
        .map(|minute| SpinOption {
            text: NUMBERS[minute as usize].as_str(),
            disabled: !view.within(of(hours, minute, 0), of(hours, minute, 59)),
        })
        .collect();
    let seconds_column = props.with_seconds.then(|| {
        let options = (0..60)
            .map(|second| SpinOption {
                text: NUMBERS[second as usize].as_str(),
                disabled: !view.within(of(hours, minutes, second), of(hours, minutes, second)),
            })
            .collect();
        column(
            COLUMNS[2],
            names.seconds_label,
            names.seconds_short,
            names.seconds_value,
            options,
            place(seconds, true),
            seconds,
            true,
            15,
        )
    });
    // A duration has no cycle to round its hours; minutes and seconds wrap
    // without carrying into the hour.
    let body = rsx! {
        div { "data-slot": "columns",
            {column(COLUMNS[0], names.hours_label, names.hours_short, names.hours_value, hour_options, place(hours, true), hours, false, 10)}
            {column(COLUMNS[1], names.minutes_label, names.minutes_short, names.minutes_value, minute_options, place(minutes / step, minutes % step == 0), minutes, true, (15 / step).max(1) as usize)}
            {seconds_column}
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
                value: value.map(iso_duration).unwrap_or_default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    const EN: &DateLocale = &DateLocale::ENGLISH;
    const DE: &DateLocale = &DateLocale::GERMAN;

    #[test]
    fn shows_the_parts_that_are_not_zero() {
        assert_eq!(show_duration(of(1, 30, 0), EN), "1 h 30 min");
        assert_eq!(show_duration(of(2, 0, 0), EN), "2 h");
        assert_eq!(show_duration(of(0, 0, 45), EN), "45 s");
        assert_eq!(show_duration(of(1, 0, 5), EN), "1 h 5 s");
        assert_eq!(show_duration(TimeDelta::zero(), EN), "0 min");
        assert_eq!(show_duration(of(1, 30, 0), DE), "1 Std. 30 Min.");
    }

    #[test]
    fn reads_units_clock_text_and_bare_minutes() {
        let read = |text| read_duration(text, EN).ok();
        assert_eq!(read("1 h 30 min"), Some(of(1, 30, 0)));
        assert_eq!(read("1h30"), Some(of(1, 30, 0)));
        assert_eq!(read("1 hour, 30 minutes"), Some(of(1, 30, 0)));
        assert_eq!(read("90 min"), Some(of(1, 30, 0)));
        assert_eq!(read("90"), Some(of(1, 30, 0)));
        assert_eq!(read("1:30"), Some(of(1, 30, 0)));
        assert_eq!(read("1:30:15"), Some(of(1, 30, 15)));
        assert_eq!(read("2m 5s"), Some(of(0, 2, 5)));
        assert_eq!(read("45 SEC"), Some(of(0, 0, 45)));
    }

    #[test]
    fn refuses_what_is_not_a_duration() {
        let read = |text| read_duration(text, EN).ok();
        for text in [
            "", "-5", "1:75", "1 h 2 h", "5 days", "1:2:3:4", "30 s 5", "h",
        ] {
            assert_eq!(read(text), None, "{text:?}");
        }
    }

    #[test]
    fn reads_the_locales_units_and_english_ones() {
        let read = |text| read_duration(text, DE).ok();
        assert_eq!(read("1 Std. 30 Min."), Some(of(1, 30, 0)));
        assert_eq!(read("2 Stunden"), Some(of(2, 0, 0)));
        assert_eq!(read("1 Stunde 5 Sekunden"), Some(of(1, 0, 5)));
        assert_eq!(read("1 h 5 s"), Some(of(1, 0, 5)));
    }

    #[test]
    fn posts_iso_8601() {
        assert_eq!(iso_duration(of(1, 30, 0)), "PT1H30M");
        assert_eq!(iso_duration(of(0, 0, 15)), "PT15S");
        assert_eq!(iso_duration(of(100, 0, 1)), "PT100H1S");
        assert_eq!(iso_duration(TimeDelta::zero()), "PT0S");
    }

    /// Todo 855: a duration's own words, naming the bound it missed.
    #[test]
    fn a_refusal_names_the_bound_it_missed() {
        let formats = |names| Formats {
            date: String::new(),
            time: String::new(),
            names,
            range_separator: " - ",
            level: super::super::DateLevel::Day,
        };
        let (min, max) = bounds(Some(of(0, 15, 0)), None);
        let refused = |span, names| duration_refusal(span, Some(min), Some(max), &formats(names));
        assert_eq!(refused(of(0, 5, 0), EN), "Must be at least 15 min");
        assert_eq!(
            refused(of(120, 0, 0), EN),
            "Must be at most 99 h 59 min 59 s"
        );
        assert_eq!(refused(of(0, 5, 0), DE), "Mindestens 15 Min.");
    }

    /// Todo 856: each spinbutton says its value with the unit.
    #[test]
    fn the_columns_say_their_unit() {
        let html = dioxus_ssr::render_element(rsx! {
            crate::LiberoProvider {
                super::super::ChronoPicker::<TimeDelta> { value: of(1, 30, 0), with_seconds: true }
            }
        });
        for valuetext in ["1 hour", "30 minutes", "0 seconds"] {
            assert!(
                html.contains(&format!(r#"aria-valuetext="{valuetext}""#)),
                "{valuetext}: {html}"
            );
        }
    }

    #[test]
    fn bounds_default_to_zero_and_99_hours() {
        assert_eq!(bounds(None, None), (TimeDelta::zero(), DEFAULT_MAX));
        assert_eq!(bounds(Some(of(0, -5, 0)), None).0, TimeDelta::zero());
        assert_eq!(
            bounds(Some(of(2, 0, 0)), Some(of(1, 0, 0))),
            (of(2, 0, 0), of(2, 0, 0))
        );
    }
}
