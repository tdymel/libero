//! dayjs' format tokens - the ones a date and a time need - read once and
//! shared by the formatters and the parsers. `chrono`'s own `format` takes
//! strftime and English names; these take the theme's.

use chrono::{Datelike, NaiveDate, NaiveTime, Timelike};

use crate::theme::DateDefaults;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Token<'a> {
    Literal(&'a str),
    /// `YYYY`
    Year,
    /// `M`, `MM`
    Month {
        padded: bool,
    },
    /// `MMM`, `MMMM`
    MonthName {
        short: bool,
    },
    /// `D`, `DD`
    Day {
        padded: bool,
    },
    /// `dd`, `ddd`, `dddd`
    WeekdayName(WeekdayWidth),
    /// `H`, `HH` for `0..24`; `h`, `hh` for `1..=12`
    Hour {
        padded: bool,
        twelve: bool,
    },
    /// `m`, `mm`
    Minute {
        padded: bool,
    },
    /// `s`, `ss`
    Second {
        padded: bool,
    },
    /// `A` for `AM`, `a` for `am`
    Meridiem {
        upper: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WeekdayWidth {
    Min,
    Short,
    Long,
}

/// Splits a format into tokens. A run of token letters with no meaning
/// (`YY`, `DDD`) is literal text, and so is everything inside `[brackets]`.
pub(super) fn tokens(format: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut rest = format;
    while let Some(first) = rest.chars().next() {
        if first == '[' {
            let end = rest.find(']').unwrap_or(rest.len());
            tokens.push(Token::Literal(&rest[1..end]));
            rest = &rest[(end + 1).min(rest.len())..];
            continue;
        }
        let run = rest.len() - rest.trim_start_matches(first).len();
        tokens.push(match (first, run) {
            ('Y', 4) => Token::Year,
            ('M', 1 | 2) => Token::Month { padded: run == 2 },
            ('M', 3 | 4) => Token::MonthName { short: run == 3 },
            ('D', 1 | 2) => Token::Day { padded: run == 2 },
            ('d', 2) => Token::WeekdayName(WeekdayWidth::Min),
            ('d', 3) => Token::WeekdayName(WeekdayWidth::Short),
            ('d', 4) => Token::WeekdayName(WeekdayWidth::Long),
            ('H', 1 | 2) => Token::Hour {
                padded: run == 2,
                twelve: false,
            },
            ('h', 1 | 2) => Token::Hour {
                padded: run == 2,
                twelve: true,
            },
            ('m', 1 | 2) => Token::Minute { padded: run == 2 },
            ('s', 1 | 2) => Token::Second { padded: run == 2 },
            ('A', 1) => Token::Meridiem { upper: true },
            ('a', 1) => Token::Meridiem { upper: false },
            _ => Token::Literal(&rest[..run]),
        });
        rest = &rest[run..];
    }
    tokens
}

/// Whether `format` shows a 12-hour clock.
pub(super) fn uses_twelve_hours(format: &str) -> bool {
    tokens(format).iter().any(|token| {
        matches!(
            token,
            Token::Hour { twelve: true, .. } | Token::Meridiem { .. }
        )
    })
}

/// The day as `format` shows it, with the names from `names`.
pub(super) fn format_date(date: NaiveDate, format: &str, names: &DateDefaults) -> String {
    write(format, Some(date), None, names)
}

/// The time as `format` shows it.
pub(super) fn format_time(time: NaiveTime, format: &str, names: &DateDefaults) -> String {
    write(format, None, Some(time), names)
}

/// `format` written out for a day, a time, or both. A token for the part that
/// is missing writes nothing.
fn write(
    format: &str,
    date: Option<NaiveDate>,
    time: Option<NaiveTime>,
    names: &DateDefaults,
) -> String {
    let mut text = String::new();
    for token in tokens(format) {
        match (token, date, time) {
            (Token::Literal(literal), _, _) => text.push_str(literal),
            (Token::Year, Some(date), _) => text.push_str(&year_text(date.year())),
            (Token::Month { padded }, Some(date), _) => {
                text.push_str(&number(date.month(), padded))
            }
            (Token::MonthName { short }, Some(date), _) => {
                let months = match short {
                    true => &names.months_short,
                    false => &names.months,
                };
                text.push_str(months[date.month0() as usize]);
            }
            (Token::Day { padded }, Some(date), _) => text.push_str(&number(date.day(), padded)),
            (Token::WeekdayName(width), Some(date), _) => {
                let weekdays = match width {
                    WeekdayWidth::Min => &names.weekdays_min,
                    WeekdayWidth::Short => &names.weekdays_short,
                    WeekdayWidth::Long => &names.weekdays,
                };
                text.push_str(weekdays[date.weekday().num_days_from_sunday() as usize]);
            }
            (Token::Hour { padded, twelve }, _, Some(time)) => {
                let hour = match twelve {
                    true => time.hour12().1,
                    false => time.hour(),
                };
                text.push_str(&number(hour, padded));
            }
            (Token::Minute { padded }, _, Some(time)) => {
                text.push_str(&number(time.minute(), padded))
            }
            (Token::Second { padded }, _, Some(time)) => {
                text.push_str(&number(time.second(), padded))
            }
            (Token::Meridiem { upper }, _, Some(time)) => {
                let word = match time.hour12().0 {
                    false => names.am,
                    true => names.pm,
                };
                match upper {
                    true => text.push_str(word),
                    false => text.push_str(&word.to_lowercase()),
                }
            }
            _ => {}
        }
    }
    text
}

/// Four digits at least, and a sign for years before year 0.
fn year_text(year: i32) -> String {
    if year < 0 {
        format!("-{:04}", year.unsigned_abs())
    } else {
        format!("{year:04}")
    }
}

fn number(value: u32, padded: bool) -> String {
    if padded {
        format!("{value:02}")
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format(format: &str) -> String {
        let day = NaiveDate::from_ymd_opt(2026, 9, 4).expect("a real day");
        format_date(day, format, &DateDefaults::ENGLISH)
    }

    fn time(hour: u32, minute: u32, second: u32, format: &str) -> String {
        let time = NaiveTime::from_hms_opt(hour, minute, second).expect("a real time");
        format_time(time, format, &DateDefaults::ENGLISH)
    }

    #[test]
    fn every_date_token_formats() {
        assert_eq!(format("MMMM D, YYYY"), "September 4, 2026");
        assert_eq!(format("DD.MM.YYYY"), "04.09.2026");
        assert_eq!(format("D.M.YYYY"), "4.9.2026");
        assert_eq!(format("dd ddd dddd, MMM"), "Fr Fri Friday, Sep");
    }

    #[test]
    fn every_time_token_formats() {
        assert_eq!(time(13, 5, 9, "HH:mm:ss"), "13:05:09");
        assert_eq!(time(13, 5, 9, "H:m:s"), "13:5:9");
        assert_eq!(time(13, 5, 0, "h:mm A"), "1:05 PM");
        assert_eq!(time(9, 5, 0, "hh:mm a"), "09:05 am");
        assert_eq!(time(0, 0, 0, "h A"), "12 AM");
        assert_eq!(time(12, 0, 0, "h A"), "12 PM");
        assert!(uses_twelve_hours("h:mm"));
        assert!(uses_twelve_hours("HH:mm A"));
        assert!(!uses_twelve_hours("HH:mm"));
    }

    #[test]
    fn a_part_the_value_does_not_have_writes_nothing() {
        assert_eq!(format("D HH"), "4 ");
    }

    #[test]
    fn brackets_and_unknown_runs_are_literal() {
        assert_eq!(format("[Day] D [of] MMMM"), "Day 4 of September");
        assert_eq!(format("YY-DDD-d"), "YY-DDD-d");
        assert_eq!(format("D [unclosed"), "4 unclosed");
    }
}
