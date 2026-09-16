use chrono::Weekday;

use crate::theme::DateLevel;

/// `DateLocale::ENGLISH.format`. A named fn, so every copy of the locale
/// holds the same address and compares equal.
fn english_format(level: DateLevel) -> &'static str {
    match level {
        DateLevel::Day => "MMMM D, YYYY",
        DateLevel::Month => "MMMM YYYY",
        DateLevel::Year => "YYYY",
    }
}

/// The words and conventions every date and time component shares: one place
/// to translate. English by default; a translation replaces the names, a
/// region the first weekday and the format.
///
/// The names follow dayjs' locale files (`months`, `monthsShort`, `weekdays`,
/// `weekdaysShort`, `weekdaysMin`), so an existing translation copies over.
///
/// # Fields, in three groups
///
/// - **Names** a language gives: `months`, `months_short`, `weekdays`,
///   `weekdays_short`, `weekdays_min`, `am`, `pm`.
/// - **Conventions** a region picks: `first_weekday`, `format`,
///   `month_format`, `time_format`, `range_separator`.
/// - **Labels** a screen reader or a sighted reader gets: the paging buttons
///   (`previous_month` to `next_days`), the errors (`invalid_date` to
///   `unavailable`), the switch segments
///   (`date_label` to `end_label`), the switches themselves and the
///   `TimePicker` columns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(
    unpredictable_function_pointer_comparisons,
    reason = "`format` compares by address; a miss on a copied closure only re-renders"
)]
pub struct DateLocale {
    // Names.
    /// `January` to `December`. `MMMM` in a format, and what a typed month
    /// name is matched against.
    pub months: [&'static str; 12],
    /// `Jan` to `Dec`. `MMM` in a format; typed names match these too.
    pub months_short: [&'static str; 12],
    /// `Sunday` to `Saturday` - always Sunday first, as dayjs has them,
    /// whatever `first_weekday` says, so `Weekday::num_days_from_sunday`
    /// indexes them. `dddd` in a format.
    pub weekdays: [&'static str; 7],
    /// `Sun` to `Sat`. `ddd` in a format.
    pub weekdays_short: [&'static str; 7],
    /// `Su` to `Sa`. `dd` in a format, and a calendar's column headers.
    pub weekdays_min: [&'static str; 7],
    /// `A` in a format, and what a typed `am` matches.
    pub am: &'static str,
    /// `A` in a format, and what a typed `pm` matches.
    pub pm: &'static str,

    // Conventions.
    /// The first column of a calendar.
    pub first_weekday: Weekday,
    /// How a date field shows a day, a month or a year, in dayjs tokens:
    /// `YYYY`, `M`, `MM`, `MMM`, `MMMM`, `D`, `DD`, `dd`, `ddd`, `dddd`. Text
    /// in `[brackets]` is literal. Call it as `(names.format)(level)`.
    ///
    /// One level overridden, the rest English:
    ///
    /// ```
    /// use libero::{components::DateLevel, localization::DateLocale};
    ///
    /// const ISO_DAYS: DateLocale = DateLocale {
    ///     format: |level| match level {
    ///         DateLevel::Day => "YYYY-MM-DD",
    ///         level => (DateLocale::ENGLISH.format)(level),
    ///     },
    ///     ..DateLocale::ENGLISH
    /// };
    /// assert_eq!((ISO_DAYS.format)(DateLevel::Day), "YYYY-MM-DD");
    /// assert_eq!((ISO_DAYS.format)(DateLevel::Month), "MMMM YYYY");
    /// ```
    pub format: fn(DateLevel) -> &'static str,
    /// A calendar's month heading.
    pub month_format: &'static str,
    /// How a time is shown, in dayjs tokens: `H`, `HH`, `h`, `hh`, `m`, `mm`,
    /// `s`, `ss`, `A`, `a`. An `h` or an `A` makes pickers 12-hour.
    pub time_format: &'static str,
    /// Between a range's two ends in a field's text.
    pub range_separator: &'static str,

    // Labels.
    /// Names the button that pages a calendar back.
    pub previous_month: &'static str,
    /// Names the button that pages a calendar forward.
    pub next_month: &'static str,
    /// Names the button that pages a calendar's months back a year.
    pub previous_year: &'static str,
    pub next_year: &'static str,
    /// Names the button that pages a calendar's years back ten.
    pub previous_decade: &'static str,
    pub next_decade: &'static str,
    /// Names the button that pages a mini calendar's days back.
    pub previous_days: &'static str,
    pub next_days: &'static str,
    /// The error a date field shows for text it cannot read.
    pub invalid_date: &'static str,
    /// The error for a value before `min`, with no `max`: `{min}` in the
    /// field's own format.
    pub on_or_after: &'static str,
    /// The error for a value after `max`, with no `min`: `{max}`.
    pub on_or_before: &'static str,
    /// The error for a value outside both bounds: `{min}` and `{max}`.
    pub between: &'static str,
    /// The error for a day `exclude_date` refuses.
    pub unavailable: &'static str,
    /// The segments that switch a dropdown between its calendar and its clock.
    pub date_label: &'static str,
    pub time_label: &'static str,
    /// The segments that switch a range dropdown between its two ends.
    pub start_label: &'static str,
    pub end_label: &'static str,
    /// Names the calendar/clock switch itself.
    pub part_switch_label: &'static str,
    /// Names the start/end switch itself.
    pub side_switch_label: &'static str,
    /// Names a `TimePicker`'s columns.
    pub hours_label: &'static str,
    pub minutes_label: &'static str,
    pub seconds_label: &'static str,
}

impl DateLocale {
    pub const ENGLISH: Self = Self {
        months: [
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ],
        months_short: [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ],
        weekdays: [
            "Sunday",
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
        ],
        weekdays_short: ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
        weekdays_min: ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"],
        am: "AM",
        pm: "PM",
        first_weekday: Weekday::Mon,
        format: english_format,
        month_format: "MMMM YYYY",
        time_format: "HH:mm",
        range_separator: " – ",
        previous_month: "Previous month",
        next_month: "Next month",
        previous_year: "Previous year",
        next_year: "Next year",
        previous_decade: "Previous decade",
        next_decade: "Next decade",
        previous_days: "Previous days",
        next_days: "Next days",
        invalid_date: "Not a valid date",
        on_or_after: "Must be on or after {min}",
        on_or_before: "Must be on or before {max}",
        between: "Must be between {min} and {max}",
        unavailable: "That date is not available",
        date_label: "Date",
        time_label: "Time",
        start_label: "Start",
        end_label: "End",
        part_switch_label: "Date or time",
        side_switch_label: "Range end",
        hours_label: "Hours",
        minutes_label: "Minutes",
        seconds_label: "Seconds",
    };
}
