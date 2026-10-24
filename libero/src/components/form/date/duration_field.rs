//! `TimeDelta` as a field's text: shown as `1 h 30 min`, read back from that or `1:30`.

use std::rc::Rc;

use chrono::{NaiveDate, TimeDelta};

use super::{
    date_value::PickerOptions,
    duration::{bounds, of},
    field_value::{FieldValue, Formats, Refusal, SealedField, within},
    parse::Unreadable,
};
use crate::{
    components::form::FieldError,
    localization::{DateLocale, Localization, ZeroIn, fill},
    utils::fold_digits,
};

/// `1 h 30 min`: the parts that are not zero, `0 min` for none.
fn show_duration(span: TimeDelta, names: &DateLocale) -> String {
    let sign = if span < TimeDelta::zero() { "-" } else { "" };
    let seconds = span.abs().num_seconds().unsigned_abs();
    format!("{sign}{}", names.short_duration(seconds, ZeroIn::Minutes))
}

/// Lower case, without the dots of an abbreviation (`Std.`).
fn bare(word: &str) -> String {
    word.to_lowercase().replace('.', "")
}

/// `1.5` or `1,5`, or a `.5`: a fraction, which `read_duration` refuses instead of
/// reading `1.5 h` as 15 h.
fn has_decimal_separator(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let digit = |index: Option<usize>| {
        index
            .and_then(|index| chars.get(index))
            .is_some_and(char::is_ascii_digit)
    };
    chars.iter().enumerate().any(|(at, &c)| {
        let (before, after) = (at.checked_sub(1), Some(at + 1));
        match c {
            '.' => digit(after) && !before.is_some_and(|b| chars[b].is_alphabetic()),
            ',' => digit(before) && digit(after),
            _ => false,
        }
    })
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

/// Reads `1 h 30 min`, `1h30`, `90 min`, `1:30`, `1:30:15` or bare minutes.
/// A number after a unit counts in the next smaller one. Nothing negative.
fn read_duration(text: &str, names: &DateLocale) -> Result<TimeDelta, Unreadable> {
    let text = fold_digits(text.trim());
    if has_decimal_separator(&text) {
        return Err(Unreadable);
    }
    let text = bare(&text);
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

/// The error naming the bound missed. Never "between": a default bound was never set.
fn duration_refusal(
    span: TimeDelta,
    min: Option<TimeDelta>,
    max: Option<TimeDelta>,
    names: &DateLocale,
) -> String {
    match (min, max) {
        (Some(min), _) if span < min => fill(
            names.duration_at_least,
            &[("min", &show_duration(min, names))],
        ),
        (_, Some(max)) if span > max => fill(
            names.duration_at_most,
            &[("max", &show_duration(max, names))],
        ),
        _ => names.invalid_duration.to_string(),
    }
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

    fn dialog_label(names: &DateLocale) -> &'static str {
        names.duration_label
    }

    fn unreadable(names: &DateLocale) -> &'static str {
        names.invalid_duration
    }
}

impl SealedField for TimeDelta {
    fn limits(
        min: Option<TimeDelta>,
        max: Option<TimeDelta>,
    ) -> (Option<TimeDelta>, Option<TimeDelta>) {
        let (min, max) = bounds(min, max);
        (Some(min), Some(max))
    }

    fn accepts(self, options: &PickerOptions<TimeDelta>) -> Result<(), Refusal> {
        let (min, max) = Self::limits(options.min, options.max);
        within(self, min, max)
    }

    fn show_bound(bound: TimeDelta, _: &Formats, names: &DateLocale) -> String {
        show_duration(bound, names)
    }

    fn refusal(
        self,
        _: Refusal,
        min: Option<TimeDelta>,
        max: Option<TimeDelta>,
        _: &Formats,
    ) -> Rc<dyn FieldError> {
        Rc::new(move |locale: &Localization| duration_refusal(self, min, max, &locale.date))
    }

    fn closes(_: Option<Self>) -> bool {
        false
    }
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

    /// Todo 3803: the separator was dropped, so `1.5 h` read as 15 h.
    #[test]
    fn refuses_a_fraction_instead_of_dropping_its_separator() {
        let read = |text| read_duration(text, EN).ok();
        for text in ["1.5 h", "1,5 h", "1.5:30", "1.5", ".5 h", "0,5"] {
            assert_eq!(read(text), None, "{text:?}");
        }
        assert_eq!(read("1 h, 30 min"), Some(of(1, 30, 0)));
        assert_eq!(read_duration("1 Std. 30 Min.", DE).ok(), Some(of(1, 30, 0)));
    }

    #[test]
    fn reads_the_locales_units_and_english_ones() {
        let read = |text| read_duration(text, DE).ok();
        assert_eq!(read("1 Std. 30 Min."), Some(of(1, 30, 0)));
        assert_eq!(read("2 Stunden"), Some(of(2, 0, 0)));
        assert_eq!(read("1 Stunde 5 Sekunden"), Some(of(1, 0, 5)));
        assert_eq!(read("1 h 5 s"), Some(of(1, 0, 5)));
    }

    /// Todo 2396: another script's digits stopped the number, so the text never read.
    #[test]
    fn digits_of_another_script_read() {
        let read = |text| read_duration(text, EN).ok();
        assert_eq!(read("\u{0661} h \u{0663}\u{0660} min"), Some(of(1, 30, 0)));
        assert_eq!(read("\u{0661}:\u{0663}\u{0660}"), Some(of(1, 30, 0)));
    }

    /// Todo 855: a duration's own words, naming the bound it missed; todo 4092: worded per language.
    #[test]
    fn a_refusal_names_the_bound_it_missed() {
        let formats = Formats {
            date: String::new(),
            time: String::new(),
            names: EN,
            range_separator: " - ",
            level: super::super::DateLevel::Day,
        };
        let (min, max) = bounds(Some(of(0, 15, 0)), None);
        let refused = |span: TimeDelta, locale: &Localization| {
            span.refusal(Refusal::OutOfRange, Some(min), Some(max), &formats)
                .message(locale)
        };
        let english = &Localization::ENGLISH;
        assert_eq!(refused(of(0, 5, 0), english), "Must be at least 15 min");
        assert_eq!(
            refused(of(120, 0, 0), english),
            "Must be at most 99 h 59 min 59 s"
        );
        assert_eq!(
            refused(of(0, 5, 0), &Localization::GERMAN),
            "Mindestens 15 Min."
        );
    }
}
