/// The words every date and time component shares: one place to translate.
/// English by default, German in [`GERMAN`](Self::GERMAN). How dates are
/// written - first weekday, patterns, clock - is [`Formats`](super::Formats).
///
/// The names follow dayjs' locale files (`months`, `monthsShort`, `weekdays`,
/// `weekdaysShort`, `weekdaysMin`), so an existing translation copies over.
///
/// # Fields, in two groups
///
/// - **Names**: `months`, `months_short`, `weekdays`, `weekdays_short`,
///   `weekdays_min`, `am`, `pm`.
/// - **Labels** a screen reader or a sighted reader gets: the paging buttons
///   (`previous_month` to `next_days`), the errors (`invalid_date` to
///   `unavailable`), the switch segments
///   (`date_label` to `end_label`), the switches themselves and the
///   `TimePicker` columns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateLocale {
    // Names.
    /// `January` to `December`. `MMMM` in a format, and what a typed month
    /// name is matched against.
    pub months: [&'static str; 12],
    /// `Jan` to `Dec`. `MMM` in a format; typed names match these too.
    pub months_short: [&'static str; 12],
    /// `Sunday` to `Saturday` - always Sunday first, as dayjs has them,
    /// whatever `Formats::first_weekday` says, so
    /// `Weekday::num_days_from_sunday` indexes them. `dddd` in a format.
    pub weekdays: [&'static str; 7],
    /// `Sun` to `Sat`. `ddd` in a format.
    pub weekdays_short: [&'static str; 7],
    /// `Su` to `Sa`. `dd` in a format, and a calendar's column headers.
    pub weekdays_min: [&'static str; 7],
    /// `A` in a format, and what a typed `am` matches.
    pub am: &'static str,
    /// `A` in a format, and what a typed `pm` matches.
    pub pm: &'static str,

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

    pub const GERMAN: Self = Self {
        months: [
            "Januar",
            "Februar",
            "März",
            "April",
            "Mai",
            "Juni",
            "Juli",
            "August",
            "September",
            "Oktober",
            "November",
            "Dezember",
        ],
        months_short: [
            "Jan.", "Feb.", "März", "Apr.", "Mai", "Juni", "Juli", "Aug.", "Sept.", "Okt.", "Nov.",
            "Dez.",
        ],
        weekdays: [
            "Sonntag",
            "Montag",
            "Dienstag",
            "Mittwoch",
            "Donnerstag",
            "Freitag",
            "Samstag",
        ],
        weekdays_short: ["So.", "Mo.", "Di.", "Mi.", "Do.", "Fr.", "Sa."],
        weekdays_min: ["So", "Mo", "Di", "Mi", "Do", "Fr", "Sa"],
        am: "AM",
        pm: "PM",
        previous_month: "Vorheriger Monat",
        next_month: "Nächster Monat",
        previous_year: "Vorheriges Jahr",
        next_year: "Nächstes Jahr",
        previous_decade: "Vorheriges Jahrzehnt",
        next_decade: "Nächstes Jahrzehnt",
        previous_days: "Vorherige Tage",
        next_days: "Nächste Tage",
        invalid_date: "Kein gültiges Datum",
        // "Frühestens" and "spätestens" fit a day and a time alike.
        on_or_after: "Frühestens {min}",
        on_or_before: "Spätestens {max}",
        between: "Muss zwischen {min} und {max} liegen",
        unavailable: "Dieses Datum ist nicht verfügbar",
        date_label: "Datum",
        time_label: "Uhrzeit",
        start_label: "Beginn",
        end_label: "Ende",
        part_switch_label: "Datum oder Uhrzeit",
        side_switch_label: "Ende des Zeitraums",
        hours_label: "Stunden",
        minutes_label: "Minuten",
        seconds_label: "Sekunden",
    };
}
