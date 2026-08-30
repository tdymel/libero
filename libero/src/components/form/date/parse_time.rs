//! Typed text back into a time, or a day and a time. As forgiving as the date
//! reader: hours, minutes and seconds come in that order, and whatever sits
//! between them only separates.

use chrono::{NaiveDateTime, NaiveTime};

use super::parse::{Piece, Unreadable, parse_date, pieces};
use crate::theme::DateDefaults;

pub(super) const MIDNIGHT: NaiveTime = match NaiveTime::from_hms_opt(0, 0, 0) {
    Some(midnight) => midnight,
    None => unreachable!(),
};

/// Reads what someone typed:
///
/// - hours, then minutes, then seconds, separated by anything: `13:05`,
///   `13.05.30`, `9 30`;
/// - or run together: `930`, `0930`, `93015`;
/// - an `am`/`pm` word from `names`, any case or a prefix (`p`), makes it a
///   12-hour time: `1:05 pm`, `12am`.
pub(super) fn parse_time(text: &str, names: &DateDefaults) -> Result<NaiveTime, Unreadable> {
    let mut numbers = Vec::new();
    let mut afternoon = None;
    for piece in pieces(text) {
        match piece {
            Piece::Number(digits) => numbers.push(digits),
            Piece::Word(word) => {
                let word = word.to_lowercase();
                let am = names.am.to_lowercase().starts_with(&word);
                let pm = names.pm.to_lowercase().starts_with(&word);
                if am == pm || afternoon.replace(pm).is_some() {
                    return Err(Unreadable);
                }
            }
        }
    }

    let parts = match numbers[..] {
        [digits] if digits.len() > 2 => {
            let width = digits.len();
            match width {
                3 | 4 => vec![&digits[..width - 2], &digits[width - 2..]],
                5 | 6 => vec![
                    &digits[..width - 4],
                    &digits[width - 4..width - 2],
                    &digits[width - 2..],
                ],
                _ => return Err(Unreadable),
            }
        }
        _ => numbers,
    };
    if parts.is_empty() || parts.len() > 3 || parts.iter().any(|part| part.len() > 2) {
        return Err(Unreadable);
    }

    let number = |index: usize| {
        parts
            .get(index)
            .map_or(Ok(0), |part| part.parse::<u32>().map_err(|_| Unreadable))
    };
    let (mut hour, minute, second) = (number(0)?, number(1)?, number(2)?);
    if let Some(afternoon) = afternoon {
        if !(1..=12).contains(&hour) {
            return Err(Unreadable);
        }
        hour = hour % 12 + if afternoon { 12 } else { 0 };
    }
    NaiveTime::from_hms_opt(hour, minute, second).ok_or(Unreadable)
}

/// Reads a typed day and time: the time starts at the number before the first
/// `:`, and everything before it is the day, read as [`parse_date`] reads it.
/// Without a `:` the whole text is the day and the time is `fallback_time`,
/// else midnight.
pub(super) fn parse_date_time(
    text: &str,
    format: &str,
    names: &DateDefaults,
    fallback_year: Option<i32>,
    fallback_time: Option<NaiveTime>,
) -> Result<NaiveDateTime, Unreadable> {
    let (date_text, time_text) = match text.find(':') {
        Some(colon) => {
            let start = text[..colon]
                .trim_end_matches(|character: char| character.is_ascii_digit())
                .len();
            // An ISO `T` between the two is a separator, not a word.
            let date_text = text[..start].trim_end_matches(['T', 't']);
            (date_text, Some(&text[start..]))
        }
        None => (text, None),
    };
    let date = parse_date(date_text, format, names, fallback_year)?;
    let time = match time_text {
        Some(time_text) => parse_time(time_text, names)?,
        None => fallback_time.unwrap_or(MIDNIGHT),
    };
    Ok(NaiveDateTime::new(date, time))
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    fn time(text: &str) -> Result<NaiveTime, Unreadable> {
        parse_time(text, &DateDefaults::ENGLISH)
    }

    fn at(hour: u32, minute: u32, second: u32) -> Result<NaiveTime, Unreadable> {
        NaiveTime::from_hms_opt(hour, minute, second).ok_or(Unreadable)
    }

    #[test]
    fn times_read_with_any_separator_or_none() {
        assert_eq!(time("13:05"), at(13, 5, 0));
        assert_eq!(time("13.05.30"), at(13, 5, 30));
        assert_eq!(time("9 30"), at(9, 30, 0));
        assert_eq!(time("9"), at(9, 0, 0));
        assert_eq!(time("930"), at(9, 30, 0));
        assert_eq!(time("0930"), at(9, 30, 0));
        assert_eq!(time("093015"), at(9, 30, 15));
    }

    #[test]
    fn am_and_pm_make_a_twelve_hour_time() {
        assert_eq!(time("1:05 pm"), at(13, 5, 0));
        assert_eq!(time("1:05 PM"), at(13, 5, 0));
        assert_eq!(time("1:05p"), at(13, 5, 0));
        assert_eq!(time("12am"), at(0, 0, 0));
        assert_eq!(time("12 pm"), at(12, 0, 0));
        assert_eq!(time("13 pm"), Err(Unreadable));
    }

    #[test]
    fn impossible_times_are_rejected() {
        for text in [
            "24:00", "12:60", "1:2:3:4", "", "1:05 xm", "1234567", "123:4",
        ] {
            assert_eq!(time(text), Err(Unreadable), "{text}");
        }
    }

    #[test]
    fn a_date_time_reads_the_date_then_the_time() {
        let names = &DateDefaults::ENGLISH;
        let expected = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2026, 2, 1).expect("a real day"),
            NaiveTime::from_hms_opt(13, 5, 0).expect("a real time"),
        );
        let read = |text: &str, format: &str, fallback_time: Option<NaiveTime>| {
            parse_date_time(text, format, names, None, fallback_time)
        };
        assert_eq!(read("1.2.2026 13:05", "DD.MM.YYYY", None), Ok(expected));
        assert_eq!(
            read("feb 1 2026, 1:05 pm", "MMMM D, YYYY", None),
            Ok(expected)
        );
        assert_eq!(read("2026-02-01T13:05", "YYYY-MM-DD", None), Ok(expected));
        assert_eq!(
            read("1.2.2026", "DD.MM.YYYY", Some(expected.time())),
            Ok(expected)
        );
    }
}
