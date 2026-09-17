use chrono::Weekday;

use crate::theme::DateLevel;

/// `Formats::AMERICAN.date`. A named fn, so every copy of the formats holds the
/// same address and compares equal.
fn american_date(level: DateLevel) -> &'static str {
    match level {
        DateLevel::Day => "MMMM D, YYYY",
        DateLevel::Month => "MMMM YYYY",
        DateLevel::Year => "YYYY",
    }
}

/// `Formats::GERMAN.date`, named for the same reason.
fn german_date(level: DateLevel) -> &'static str {
    match level {
        DateLevel::Day => "D. MMMM YYYY",
        DateLevel::Month => "MMMM YYYY",
        DateLevel::Year => "YYYY",
    }
}

/// How dates, times and numbers are written: the conventions a region picks,
/// apart from the language that names things. [`AMERICAN`](Self::AMERICAN) by
/// default, [`GERMAN`](Self::GERMAN) ships too.
///
/// Handed to `LiberoProvider { formats }`, read with
/// [`use_formats`](crate::hooks::use_formats). Any language goes with any
/// formats: English words, German dates.
///
/// ```
/// use dioxus::prelude::*;
/// use libero::{LiberoProvider, localization::{Formats, Localization}};
///
/// // English words, German dates: "14. September 2026 15:30", Monday first.
/// fn App() -> Element {
///     rsx! {
///         LiberoProvider {
///             localization: &Localization::ENGLISH,
///             formats: &Formats::GERMAN,
///         }
///     }
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(
    unpredictable_function_pointer_comparisons,
    reason = "`date` compares by address; a miss on a copied closure only re-renders"
)]
pub struct Formats {
    /// The first column of a calendar.
    pub first_weekday: Weekday,
    /// How a date field shows a day, a month or a year, in dayjs tokens:
    /// `YYYY`, `M`, `MM`, `MMM`, `MMMM`, `D`, `DD`, `dd`, `ddd`, `dddd`. Text
    /// in `[brackets]` is literal. Call it as `(formats.date)(level)`.
    pub date: fn(DateLevel) -> &'static str,
    /// A calendar's month heading.
    pub month_heading: &'static str,
    /// How a time is shown, in dayjs tokens: `H`, `HH`, `h`, `hh`, `m`, `mm`,
    /// `s`, `ss`, `A`, `a`. An `h` or an `A` makes pickers 12-hour.
    pub time: &'static str,
    /// Between a range's two ends in a field's text.
    pub range_separator: &'static str,
    /// Between a number's whole and its fraction: `5.4 MB`, `5,4 MB`.
    pub decimal_separator: &'static str,
}

impl Formats {
    /// Sunday first, a 12-hour clock, `September 14, 2026`, `5.4`.
    pub const AMERICAN: Self = Self {
        first_weekday: Weekday::Sun,
        date: american_date,
        month_heading: "MMMM YYYY",
        time: "h:mm A",
        range_separator: " – ",
        decimal_separator: ".",
    };

    /// Monday first, a 24-hour clock, `14. September 2026`, `5,4`.
    pub const GERMAN: Self = Self {
        first_weekday: Weekday::Mon,
        date: german_date,
        month_heading: "MMMM YYYY",
        time: "HH:mm",
        range_separator: " – ",
        decimal_separator: ",",
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn american_formats_are_american() {
        let american = Formats::AMERICAN;
        assert_eq!(american.first_weekday, Weekday::Sun);
        assert_eq!((american.date)(DateLevel::Day), "MMMM D, YYYY");
        assert_eq!((american.date)(DateLevel::Month), "MMMM YYYY");
        assert_eq!((american.date)(DateLevel::Year), "YYYY");
        assert_eq!(american.time, "h:mm A");
        assert_eq!(american.decimal_separator, ".");
    }

    #[test]
    fn german_formats_are_german() {
        let german = Formats::GERMAN;
        assert_eq!(german.first_weekday, Weekday::Mon);
        assert_eq!((german.date)(DateLevel::Day), "D. MMMM YYYY");
        assert_eq!((german.date)(DateLevel::Month), "MMMM YYYY");
        assert_eq!((german.date)(DateLevel::Year), "YYYY");
        assert_eq!(german.time, "HH:mm");
        assert_eq!(german.decimal_separator, ",");
    }

    /// A copy compares equal, so a provider handed the same formats twice
    /// does not re-render.
    #[test]
    fn a_copy_compares_equal() {
        let copy = Formats::GERMAN;
        assert_eq!(copy, Formats::GERMAN);
        assert_ne!(Formats::AMERICAN, Formats::GERMAN);
    }
}
