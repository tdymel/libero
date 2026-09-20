//! Typed text back into a time, or a day and a time. Hours, minutes, seconds in order;
//! whatever sits between them only separates.

use chrono::{NaiveDateTime, NaiveTime};

use super::parse::{Piece, Unreadable, literal_words, parse_date, pieces};
use crate::localization::DateLocale;

pub(super) const MIDNIGHT: NaiveTime = match NaiveTime::from_hms_opt(0, 0, 0) {
    Some(midnight) => midnight,
    None => unreachable!(),
};

/// Reads `13:05`, `13.05.30`, `9 30`, `930`, `93015`; an am/pm word or prefix makes it 12-hour.
/// A word `format` writes as literal text is skipped: `h` in `HH[ h ]mm`.
pub(super) fn parse_time(
    text: &str,
    format: &str,
    names: &DateLocale,
) -> Result<NaiveTime, Unreadable> {
    let literals = literal_words(format);
    let mut numbers = Vec::new();
    let mut afternoon = None;
    for piece in pieces(text) {
        match piece {
            Piece::Number(digits) => numbers.push(digits),
            Piece::Word(word) => {
                let word = word.to_lowercase();
                if literals.contains(&word) {
                    continue;
                }
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

/// Reads a day and time: the time starts at the number (or am/pm word) before the first `:`
/// or `time_format` literal. Without one, the time is `fallback_time`, else midnight.
pub(super) fn parse_date_time(
    text: &str,
    format: &str,
    time_format: &str,
    names: &DateLocale,
    fallback_year: Option<i32>,
    fallback_time: Option<NaiveTime>,
) -> Result<NaiveDateTime, Unreadable> {
    let (date_text, time_text) = match time_marker(text, format, time_format) {
        Some(marker) => {
            let mut start = text[..marker]
                .trim_end()
                .trim_end_matches(|character: char| character.is_ascii_digit())
                .len();
            let before = text[..start].trim_end();
            if let Some(Piece::Word(word)) = pieces(before).pop()
                && before.ends_with(word)
                && is_meridiem(word, names)
            {
                start = before.len() - word.len();
            }
            // An ISO `T` between the two is a separator, not a word.
            let date_text = text[..start].trim_end_matches(['T', 't']);
            (date_text, Some(&text[start..]))
        }
        None => (text, None),
    };
    let date = parse_date(date_text, format, names, fallback_year)?;
    let time = match time_text {
        Some(time_text) => parse_time(time_text, time_format, names)?,
        None => fallback_time.unwrap_or(MIDNIGHT),
    };
    Ok(NaiveDateTime::new(date, time))
}

/// Where the time starts: the first `:`, or the first literal word only `time_format` writes.
fn time_marker(text: &str, format: &str, time_format: &str) -> Option<usize> {
    let date_words = literal_words(format);
    let time_words: Vec<String> = literal_words(time_format)
        .into_iter()
        .filter(|word| !date_words.contains(word))
        .collect();
    let word = pieces(text).into_iter().find_map(|piece| match piece {
        Piece::Word(word) if time_words.contains(&word.to_lowercase()) => {
            Some(word.as_ptr().addr() - text.as_ptr().addr())
        }
        _ => None,
    });
    text.find(':').into_iter().chain(word).min()
}

fn is_meridiem(word: &str, names: &DateLocale) -> bool {
    let word = word.to_lowercase();
    word == names.am.to_lowercase() || word == names.pm.to_lowercase()
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    fn time(text: &str) -> Result<NaiveTime, Unreadable> {
        parse_time(text, "HH:mm", &DateLocale::ENGLISH)
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
    fn a_word_the_time_format_writes_is_skipped() {
        let names = &DateLocale::ENGLISH;
        assert_eq!(parse_time("14 h 05", "HH[ h ]mm", names), at(14, 5, 0));
        assert_eq!(parse_time("14 h 05", "HH:mm", names), Err(Unreadable));
        let read = |text: &str| {
            parse_date_time(text, "D.M.YYYY", "HH[ h ]mm", names, None, None)
                .map(|moment| moment.time())
        };
        assert_eq!(read("4.3.2026 14 h 05"), at(14, 5, 0));
        assert_eq!(read("4.3.2026 14:05"), at(14, 5, 0));
    }

    #[test]
    fn an_am_pm_word_before_the_hour_goes_with_the_time() {
        let read = parse_date_time(
            "1.2.2026 PM 1:05",
            "D.M.YYYY",
            "A h:mm",
            &DateLocale::ENGLISH,
            None,
            None,
        );
        assert_eq!(read.map(|moment| moment.time()), at(13, 5, 0));
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
        let names = &DateLocale::ENGLISH;
        let expected = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2026, 2, 1).expect("a real day"),
            NaiveTime::from_hms_opt(13, 5, 0).expect("a real time"),
        );
        let read = |text: &str, format: &str, fallback_time: Option<NaiveTime>| {
            parse_date_time(text, format, "HH:mm", names, None, fallback_time)
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
