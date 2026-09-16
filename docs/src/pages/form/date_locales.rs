// A custom localization to copy: only what differs from English is written out.
// Needs `Localization`, `DateLocale`, `DateLevel` and `Weekday` in scope.

pub const AMERICAN: Localization = Localization {
    date: DateLocale {
        first_weekday: Weekday::Sun,
        format: |level| match level {
            DateLevel::Day => "MMM D, YYYY",
            DateLevel::Month => "MMMM YYYY",
            DateLevel::Year => "YYYY",
        },
        time_format: "h:mm A",
        ..DateLocale::ENGLISH
    },
    ..Localization::ENGLISH
};
