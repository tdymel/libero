//! Typed text back into a day. The format decides the order of day, month and
//! year; everything else is forgiven.

use chrono::NaiveDate;

use super::format::{Token, tokens};
use crate::theme::DateDefaults;

/// Typed text a field cannot read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Unreadable;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Part {
    Year,
    Month,
    Day,
}

impl Part {
    /// How many digits the part takes when the text has no separators.
    const fn compact_width(self) -> usize {
        match self {
            Self::Year => 4,
            Self::Month | Self::Day => 2,
        }
    }
}

pub(super) enum Piece<'a> {
    Number(&'a str),
    Word(&'a str),
}

/// Reads what someone typed, in the order `format` puts day, month and year,
/// with the names from `names`:
///
/// - any run of non-alphanumeric characters separates, so `1/2/2026`,
///   `01-02-2026` and `1. 2. 2026` all read alike;
/// - digits alone work when they are exactly as wide as `DDMMYYYY` in the
///   format's order: `01022026`;
/// - day and month take one or two digits, the year exactly four;
/// - a month may be a name, any case, full, short or a prefix naming only one
///   month: `1 feb 2026`, `1 Septem 2026`. A weekday name is skipped;
/// - without a year, `fallback_year` fills it in - the current year, or the
///   field's current value's.
///
/// A format with no day or no month token reads nothing.
pub(super) fn parse_date(
    text: &str,
    format: &str,
    names: &DateDefaults,
    fallback_year: Option<i32>,
) -> Result<NaiveDate, Unreadable> {
    let mut numbers = Vec::new();
    let mut named_month = None;
    for piece in pieces(text) {
        match piece {
            Piece::Number(digits) => numbers.push(digits),
            Piece::Word(word) => {
                let word = word.to_lowercase();
                if let Some(month) = month_named(&word, names) {
                    if named_month.replace(month).is_some() {
                        return Err(Unreadable);
                    }
                } else if !is_weekday(&word, names) {
                    return Err(Unreadable);
                }
            }
        }
    }

    let mut slots: Vec<Part> = order(format)
        .into_iter()
        .filter(|part| !(*part == Part::Month && named_month.is_some()))
        .collect();
    let compact_width: usize = slots.iter().map(|part| part.compact_width()).sum();
    if let [digits] = numbers[..]
        && slots.len() > 1
        && digits.len() == compact_width
    {
        numbers.clear();
        let mut start = 0;
        for part in &slots {
            numbers.push(&digits[start..start + part.compact_width()]);
            start += part.compact_width();
        }
    }
    if numbers.len() + 1 == slots.len() {
        slots.retain(|part| *part != Part::Year);
    }
    if numbers.len() != slots.len() {
        return Err(Unreadable);
    }

    let (mut year, mut month, mut day) = (None, named_month, None);
    for (part, digits) in slots.into_iter().zip(numbers) {
        match part {
            Part::Year if digits.len() == 4 => year = digits.parse().ok(),
            Part::Month if digits.len() <= 2 => month = digits.parse().ok(),
            Part::Day if digits.len() <= 2 => day = digits.parse().ok(),
            _ => return Err(Unreadable),
        }
    }
    NaiveDate::from_ymd_opt(
        year.or(fallback_year).ok_or(Unreadable)?,
        month.ok_or(Unreadable)?,
        day.ok_or(Unreadable)?,
    )
    .ok_or(Unreadable)
}

/// The parts in the order the format first names them.
fn order(format: &str) -> Vec<Part> {
    let mut order = Vec::new();
    for token in tokens(format) {
        let part = match token {
            Token::Year => Part::Year,
            Token::Month { .. } | Token::MonthName { .. } => Part::Month,
            Token::Day { .. } => Part::Day,
            _ => continue,
        };
        if !order.contains(&part) {
            order.push(part);
        }
    }
    order
}

/// Runs of ASCII digits and runs of letters; everything else only separates.
pub(super) fn pieces(text: &str) -> Vec<Piece<'_>> {
    let mut pieces = Vec::new();
    let mut chars = text.char_indices().peekable();
    while let Some((start, first)) = chars.next() {
        let digit = first.is_ascii_digit();
        if !digit && !first.is_alphabetic() {
            continue;
        }
        let same_kind = |next: char| {
            if digit {
                next.is_ascii_digit()
            } else {
                next.is_alphabetic()
            }
        };
        let mut end = start + first.len_utf8();
        while let Some(&(index, next)) = chars.peek()
            && same_kind(next)
        {
            end = index + next.len_utf8();
            chars.next();
        }
        let piece = &text[start..end];
        pieces.push(if digit {
            Piece::Number(piece)
        } else {
            Piece::Word(piece)
        });
    }
    pieces
}

/// `1` for January. An exact full or short name wins; otherwise a prefix has
/// to name exactly one month.
fn month_named(word: &str, names: &DateDefaults) -> Option<u32> {
    let months = || (0..12).map(|index| (index, names.months[index], names.months_short[index]));
    if let Some((index, ..)) = months()
        .find(|(_, long, short)| long.to_lowercase() == word || short.to_lowercase() == word)
    {
        return Some(index as u32 + 1);
    }
    let mut prefixed = months().filter(|(_, long, short)| {
        long.to_lowercase().starts_with(word) || short.to_lowercase().starts_with(word)
    });
    let (index, ..) = prefixed.next()?;
    prefixed.next().is_none().then_some(index as u32 + 1)
}

fn is_weekday(word: &str, names: &DateDefaults) -> bool {
    [names.weekdays, names.weekdays_short, names.weekdays_min]
        .iter()
        .flatten()
        .any(|name| name.to_lowercase().starts_with(word))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DMY: &str = "DD.MM.YYYY";

    fn parse(text: &str, format: &str) -> Result<NaiveDate, Unreadable> {
        parse_date(text, format, &DateDefaults::ENGLISH, None)
    }

    fn date(year: i32, month: u32, day: u32) -> Result<NaiveDate, Unreadable> {
        NaiveDate::from_ymd_opt(year, month, day).ok_or(Unreadable)
    }

    #[test]
    fn any_separator_reads() {
        for text in [
            "01.02.2026",
            "1/2/2026",
            "01-02-2026",
            "1. 2. 2026",
            " 1.2.2026 ",
        ] {
            assert_eq!(parse(text, DMY), date(2026, 2, 1), "{text}");
        }
    }

    #[test]
    fn the_format_decides_the_order() {
        assert_eq!(parse("1/2/2026", "MM/DD/YYYY"), date(2026, 1, 2));
        assert_eq!(parse("2026-2-1", "YYYY-MM-DD"), date(2026, 2, 1));
        assert_eq!(parse("2.1.2026", "MMMM D, YYYY"), date(2026, 2, 1));
    }

    #[test]
    fn digits_alone_read_at_the_exact_width() {
        assert_eq!(parse("01022026", DMY), date(2026, 2, 1));
        assert_eq!(parse("20260201", "YYYY-MM-DD"), date(2026, 2, 1));
        assert_eq!(parse("1022026", DMY), Err(Unreadable));
    }

    #[test]
    fn month_names_read_in_any_case_and_as_unique_prefixes() {
        for text in [
            "1 feb 2026",
            "1 February 2026",
            "1 FEBR 2026",
            "feb 1 2026",
            "1.Feb.2026",
        ] {
            assert_eq!(parse(text, DMY), date(2026, 2, 1), "{text}");
        }
        assert_eq!(
            parse("September 14, 2026", "MMMM D, YYYY"),
            date(2026, 9, 14)
        );
        assert_eq!(parse("May 3 2026", "MMMM D, YYYY"), date(2026, 5, 3));
        // March and May.
        assert_eq!(parse("1 ma 2026", DMY), Err(Unreadable));
        assert_eq!(parse("1 feb march 2026", DMY), Err(Unreadable));
    }

    #[test]
    fn a_weekday_name_is_skipped() {
        assert_eq!(
            parse("Mon, 14 Sep 2026", "ddd, D MMM YYYY"),
            date(2026, 9, 14)
        );
        assert_eq!(parse("1 foo 2026", DMY), Err(Unreadable));
    }

    #[test]
    fn a_missing_year_takes_the_fallback() {
        let names = &DateDefaults::ENGLISH;
        assert_eq!(parse_date("1.2", DMY, names, Some(2026)), date(2026, 2, 1));
        assert_eq!(
            parse_date("2/1", "MM/DD/YYYY", names, Some(2026)),
            date(2026, 2, 1)
        );
        assert_eq!(
            parse_date("feb 1", "MMMM D, YYYY", names, Some(2026)),
            date(2026, 2, 1)
        );
        assert_eq!(parse_date("1.2", DMY, names, None), Err(Unreadable));
    }

    #[test]
    fn short_years_and_impossible_days_are_rejected() {
        for text in [
            "1.2.26",
            "1.2.026",
            "31.2.2026",
            "1.13.2026",
            "001.2.2026",
            "",
            "1.2.3.2026",
        ] {
            assert_eq!(parse(text, DMY), Err(Unreadable), "{text}");
        }
        assert_eq!(parse("2026.2", DMY), Err(Unreadable));
    }

    #[test]
    fn a_format_without_a_day_reads_nothing() {
        assert_eq!(parse("2 2026", "MM YYYY"), Err(Unreadable));
    }
}
