//! A span between two days or two moments.

use std::{
    fmt::{self, Display},
    str::FromStr,
};

use chrono::{NaiveDate, NaiveDateTime};

/// Two days or two `NaiveDateTime`s, the start picked first. `end: None` is a range still being picked.
///
/// ```
/// # use libero::chrono::NaiveDate;
/// # use libero::components::DateRange;
/// let range: DateRange<NaiveDate> = "2026-09-01/2026-09-05".parse().unwrap();
/// assert_eq!(range.to_string(), "2026-09-01/2026-09-05");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DateRange<T> {
    pub start: T,
    pub end: Option<T>,
}

impl<T: Copy + Ord> DateRange<T> {
    pub const fn new(start: T, end: Option<T>) -> Self {
        Self { start, end }
    }

    /// The ends in order: an end before the start swaps with it.
    pub fn ordered(self) -> Self {
        match self.end {
            Some(end) if end < self.start => Self::new(end, Some(self.start)),
            _ => self,
        }
    }

    /// What a pick of `value` makes of `current`: a new start, or the missing end
    /// (swapped in when it comes before the start).
    pub fn pick(current: Option<Self>, value: T) -> Self {
        match current {
            Some(range) if range.end.is_none() => Self::new(range.start, Some(value)).ordered(),
            _ => Self::new(value, None),
        }
    }
}

/// A moment in ISO 8601, `2026-09-14T13:05:00`. `chrono`'s `Display` writes a space for the `T`.
pub(super) fn iso_date_time(value: NaiveDateTime) -> String {
    format!("{}T{}", value.date(), value.time())
}

fn write_interval(f: &mut fmt::Formatter<'_>, start: String, end: Option<String>) -> fmt::Result {
    write!(f, "{start}/{}", end.unwrap_or_default())
}

impl Display for DateRange<NaiveDate> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_interval(
            f,
            self.start.to_string(),
            self.end.map(|end| end.to_string()),
        )
    }
}

impl Display for DateRange<NaiveDateTime> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_interval(f, iso_date_time(self.start), self.end.map(iso_date_time))
    }
}

/// Text that is not an ISO 8601 interval of the range's type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseRangeError;

impl Display for ParseRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("not an ISO 8601 interval")
    }
}

impl std::error::Error for ParseRangeError {}

impl<T: FromStr + Copy + Ord> FromStr for DateRange<T> {
    type Err = ParseRangeError;

    /// `start/end` or `start/`, each end as its type's `FromStr` reads it.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (start, end) = text.split_once('/').ok_or(ParseRangeError)?;
        let end = match end {
            "" => None,
            end => Some(end.parse().map_err(|_| ParseRangeError)?),
        };
        Ok(Self::new(start.parse().map_err(|_| ParseRangeError)?, end))
    }
}

/// Typed range text split at the theme's `separator`, else a dash, ` - ` or ` to `.
/// The end is `None` when nothing follows.
pub(super) fn split_range<'a>(text: &'a str, separator: &str) -> (&'a str, Option<&'a str>) {
    // Unspaced only without ASCII: `-` or `to` would split `2026-09-01` or `October`.
    let bare = separator.trim();
    let bare = (bare != separator && !bare.is_ascii()).then_some(bare);
    let theme = [Some(separator), bare].into_iter().flatten();
    let separators = theme.chain(["–", "—", " - ", " to "]);
    for separator in separators.filter(|separator| !separator.is_empty()) {
        if let Some((start, end)) = text.split_once(separator) {
            let end = end.trim();
            return (start.trim(), (!end.is_empty()).then_some(end));
        }
    }
    (text.trim(), None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 9, day).expect("a real day")
    }

    #[test]
    fn picks_start_then_end_then_start_over() {
        let first = DateRange::pick(None, day(10));
        assert_eq!(first, DateRange::new(day(10), None));
        let second = DateRange::pick(Some(first), day(14));
        assert_eq!(second, DateRange::new(day(10), Some(day(14))));
        assert_eq!(
            DateRange::pick(Some(second), day(3)),
            DateRange::new(day(3), None)
        );
        // An end before the start swaps in.
        assert_eq!(
            DateRange::pick(Some(first), day(2)),
            DateRange::new(day(2), Some(day(10)))
        );
    }

    #[test]
    fn iso_intervals_round_trip() {
        let range = DateRange::new(day(1), Some(day(5)));
        assert_eq!(range.to_string(), "2026-09-01/2026-09-05");
        assert_eq!("2026-09-01/2026-09-05".parse(), Ok(range));
        let half = DateRange::new(day(1), None);
        assert_eq!(half.to_string(), "2026-09-01/");
        assert_eq!("2026-09-01/".parse(), Ok(half));
        assert!("2026-09-01".parse::<DateRange<NaiveDate>>().is_err());

        let moments = "2026-09-14T09:00:00/2026-09-20T17:30:00";
        let parsed: DateRange<NaiveDateTime> = moments.parse().expect("an interval");
        assert_eq!(parsed.to_string(), moments);
    }

    #[test]
    fn typed_ranges_split_at_any_separator() {
        let split = |text| split_range(text, " – ");
        assert_eq!(split("1.9 – 5.9"), ("1.9", Some("5.9")));
        assert_eq!(split("1.9-5.9"), ("1.9-5.9", None));
        assert_eq!(
            split("2026-09-01 - 2026-09-05"),
            ("2026-09-01", Some("2026-09-05"))
        );
        assert_eq!(split("sep 1 to sep 5"), ("sep 1", Some("sep 5")));
        assert_eq!(split("1.9 – "), ("1.9", None));
    }

    #[test]
    fn typed_ranges_split_at_the_theme_separator() {
        let split = |text| split_range(text, " ～ ");
        assert_eq!(split("9月1日 ～ 9月5日"), ("9月1日", Some("9月5日")));
        assert_eq!(split("9月1日～9月5日"), ("9月1日", Some("9月5日")));
        assert_eq!(split("9月1日 ～ "), ("9月1日", None));
        // The defaults still split.
        assert_eq!(split("9月1日 – 9月5日"), ("9月1日", Some("9月5日")));

        // An ASCII separator only splits with its spaces.
        let split = |text| split_range(text, " bis ");
        assert_eq!(split("1.9 bis 5.9"), ("1.9", Some("5.9")));
        assert_eq!(split("1.9bis5.9"), ("1.9bis5.9", None));
        assert_eq!(split_range("1.9 – 5.9", ""), ("1.9", Some("5.9")));
    }
}
