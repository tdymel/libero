//! The docs' copyable locales, written and read back: every day of three
//! years and every minute of a day.

use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, Weekday};

use super::calendar::{DateLevel, first_of_month};
use super::format::{format_date, format_time};
use super::parse::{parse_at, parse_date};
use super::parse_time::{parse_date_time, parse_time};
use crate::theme::DateDefaults;

// The docs file is the source; this test only reads it.
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../docs/src/pages/form/date_locales.rs"
));

const LOCALES: [(&str, DateDefaults); 4] = [
    ("en", DateDefaults::ENGLISH),
    ("de", GERMAN),
    ("fr", FRENCH),
    ("ja", JAPANESE),
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
    for (name, locale) in LOCALES {
        for day in days() {
            let format = (locale.format)(DateLevel::Day);
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
    for (name, locale) in LOCALES {
        for day in days().filter(|day| day.day() == 15) {
            let format = (locale.format)(DateLevel::Month);
            let text = format_date(day, format, &locale);
            let read = parse_at(&text, format, &locale, None, DateLevel::Month);
            assert_eq!(read, Ok(first_of_month(day)), "{name}: {text}");

            let format = (locale.format)(DateLevel::Year);
            let text = format_date(day, format, &locale);
            let read = parse_at(&text, format, &locale, None, DateLevel::Year);
            let january = NaiveDate::from_ymd_opt(day.year(), 1, 1);
            assert_eq!(read.ok(), january, "{name}: {text}");
        }
    }
}

#[test]
fn every_minute_reads_back() {
    for (name, locale) in LOCALES {
        for time in minutes() {
            let text = format_time(time, locale.time_format, &locale);
            assert_eq!(
                parse_time(&text, locale.time_format, &locale),
                Ok(time),
                "{name}: {text}"
            );
        }
    }
}

/// As a date-time field shows it: the day, a space, the time.
#[test]
fn a_moment_reads_back() {
    for (name, locale) in LOCALES {
        for day in days().step_by(29) {
            for time in minutes().step_by(97) {
                let text = format!(
                    "{} {}",
                    format_date(day, (locale.format)(DateLevel::Day), &locale),
                    format_time(time, locale.time_format, &locale)
                );
                let read = parse_date_time(
                    &text,
                    (locale.format)(DateLevel::Day),
                    locale.time_format,
                    &locale,
                    None,
                    None,
                );
                assert_eq!(read, Ok(NaiveDateTime::new(day, time)), "{name}: {text}");
            }
        }
    }
}

#[test]
fn the_weekday_names_follow_dayjs_sunday_first() {
    let sunday = NaiveDate::from_ymd_opt(2026, 3, 1).expect("a Sunday");
    assert_eq!(format_date(sunday, "dddd", &GERMAN), "Sonntag");
    assert_eq!(format_date(sunday, "dddd", &FRENCH), "dimanche");
    assert_eq!(format_date(sunday, "dddd", &JAPANESE), "日曜日");
    assert_eq!(JAPANESE.first_weekday, Weekday::Sun);
}
