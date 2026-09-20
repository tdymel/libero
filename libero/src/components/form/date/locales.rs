//! Every shipped language and format pairing, written and read back over three years of days
//! and every minute of a day.

use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta};

use super::calendar::{DateLevel, first_of_month};
use super::format::{format_date, format_time};
use super::parse::{parse_at, parse_date};
use super::parse_time::{parse_date_time, parse_time};
use crate::localization::{DateLocale, Formats};

const LOCALES: [(&str, DateLocale, Formats); 4] = [
    ("en-US", DateLocale::ENGLISH, Formats::AMERICAN),
    ("de", DateLocale::GERMAN, Formats::GERMAN),
    ("en, German formats", DateLocale::ENGLISH, Formats::GERMAN),
    (
        "de, American formats",
        DateLocale::GERMAN,
        Formats::AMERICAN,
    ),
];

fn days() -> impl Iterator<Item = NaiveDate> {
    let first = NaiveDate::from_ymd_opt(2024, 1, 1).expect("a real day");
    first.iter_days().take_while(|day| day.year() < 2027)
}

fn minutes() -> impl Iterator<Item = NaiveTime> {
    (0..24 * 60).map(|minute| NaiveTime::MIN + TimeDelta::minutes(minute))
}

#[test]
fn every_day_reads_back() {
    for (name, locale, formats) in LOCALES {
        for day in days() {
            let format = (formats.date)(DateLevel::Day);
            let text = format_date(day, format, &locale);
            assert_eq!(
                parse_date(&text, format, &locale, None),
                Ok(day),
                "{name}: {text}"
            );
        }
    }
}

/// A month reads back as its first day, a year as its January 1.
#[test]
fn every_month_and_year_reads_back() {
    for (name, locale, formats) in LOCALES {
        for day in days().filter(|day| day.day() == 15) {
            let format = (formats.date)(DateLevel::Month);
            let text = format_date(day, format, &locale);
            let read = parse_at(&text, format, &locale, None, DateLevel::Month);
            assert_eq!(read, Ok(first_of_month(day)), "{name}: {text}");

            let format = (formats.date)(DateLevel::Year);
            let text = format_date(day, format, &locale);
            let read = parse_at(&text, format, &locale, None, DateLevel::Year);
            let january = NaiveDate::from_ymd_opt(day.year(), 1, 1);
            assert_eq!(read.ok(), january, "{name}: {text}");
        }
    }
}

#[test]
fn every_minute_reads_back() {
    for (name, locale, formats) in LOCALES {
        for time in minutes() {
            let text = format_time(time, formats.time, &locale);
            assert_eq!(
                parse_time(&text, formats.time, &locale),
                Ok(time),
                "{name}: {text}"
            );
        }
    }
}

/// As a date-time field shows it: the day, a space, the time.
#[test]
fn a_moment_reads_back() {
    for (name, locale, formats) in LOCALES {
        for day in days().step_by(29) {
            for time in minutes().step_by(97) {
                let text = format!(
                    "{} {}",
                    format_date(day, (formats.date)(DateLevel::Day), &locale),
                    format_time(time, formats.time, &locale)
                );
                let read = parse_date_time(
                    &text,
                    (formats.date)(DateLevel::Day),
                    formats.time,
                    &locale,
                    None,
                    None,
                );
                assert_eq!(read, Ok(NaiveDateTime::new(day, time)), "{name}: {text}");
            }
        }
    }
}

/// The language names the month, the formats place it.
#[test]
fn the_language_and_the_formats_mix() {
    let day = NaiveDate::from_ymd_opt(2026, 3, 1).expect("a Sunday");
    let afternoon = NaiveTime::from_hms_opt(13, 5, 0).expect("a time");
    let shown = |locale: &DateLocale, formats: &Formats| {
        format!(
            "{} {}",
            format_date(day, (formats.date)(DateLevel::Day), locale),
            format_time(afternoon, formats.time, locale)
        )
    };
    let (english, german) = (&DateLocale::ENGLISH, &DateLocale::GERMAN);
    assert_eq!(shown(english, &Formats::AMERICAN), "March 1, 2026 1:05 PM");
    assert_eq!(shown(german, &Formats::GERMAN), "1. März 2026 13:05");
    assert_eq!(shown(english, &Formats::GERMAN), "1. March 2026 13:05");
    assert_eq!(shown(german, &Formats::AMERICAN), "März 1, 2026 1:05 PM");
    assert_eq!(format_date(day, "dddd", german), "Sonntag");
}
