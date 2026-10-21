use std::time::Duration;

use super::DateLocale;

/// How [`DateLocale::format_duration`] writes a span. Whole seconds; a fraction is dropped.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DurationStyle {
    /// `4:05`, `1:04:05` from an hour: a timer's face, the same in every language.
    #[default]
    Clock,
    /// `1 h 4 min 5 s`, the parts that are not zero, in the language's short units.
    Short,
    /// `1 hour 4 minutes 5 seconds`, what a screen reader should say for it.
    Spoken,
}

/// The unit a zero span is said in.
#[derive(Clone, Copy)]
pub(crate) enum ZeroIn {
    Minutes,
    Seconds,
}

/// `m:ss`, or `h:mm:ss` from an hour.
pub(crate) fn clock(total: u64) -> String {
    let (hours, minutes, seconds) = (total / 3600, total / 60 % 60, total % 60);
    match hours {
        0 => format!("{minutes}:{seconds:02}"),
        _ => format!("{hours}:{minutes:02}:{seconds:02}"),
    }
}

impl DateLocale {
    /// `span` as text in this language. `libero::hooks::format_duration` is this for
    /// the active language.
    ///
    /// ```
    /// use std::time::Duration;
    /// use libero::localization::{DateLocale, DurationStyle};
    ///
    /// let span = Duration::from_secs(3845);
    /// assert_eq!(DateLocale::ENGLISH.format_duration(span, DurationStyle::Clock), "1:04:05");
    /// assert_eq!(DateLocale::GERMAN.format_duration(span, DurationStyle::Short), "1 Std. 4 Min. 5 Sek.");
    /// ```
    pub fn format_duration(&self, span: Duration, style: DurationStyle) -> String {
        let total = span.as_secs();
        match style {
            DurationStyle::Clock => clock(total),
            DurationStyle::Short => self.short_duration(total, ZeroIn::Seconds),
            DurationStyle::Spoken => self.spoken_duration(total),
        }
    }

    /// `1 h 30 min`: the parts that are not zero, `zero` naming the unit of none.
    pub(crate) fn short_duration(&self, total: u64, zero: ZeroIn) -> String {
        let units = [self.hours_short, self.minutes_short, self.seconds_short];
        let zero = match zero {
            ZeroIn::Minutes => 1,
            ZeroIn::Seconds => 2,
        };
        join(total, zero, |unit, n| format!("{n} {}", units[unit]))
    }

    fn spoken_duration(&self, total: u64) -> String {
        let units = [self.hours_value, self.minutes_value, self.seconds_value];
        join(total, 2, |unit, n| {
            units[unit](u32::try_from(n).unwrap_or(u32::MAX))
        })
    }
}

/// The parts of `total` seconds that are not zero, each through `part(unit, n)`; for
/// none, unit `zero` with 0.
fn join(total: u64, zero: usize, part: impl Fn(usize, u64) -> String) -> String {
    let parts = [total / 3600, total / 60 % 60, total % 60];
    if total == 0 {
        return part(zero, 0);
    }
    (0..3)
        .filter(|&unit| parts[unit] > 0)
        .map(|unit| part(unit, parts[unit]))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn every(locale: &DateLocale, seconds: u64) -> [String; 3] {
        let span = Duration::from_secs(seconds);
        [
            DurationStyle::Clock,
            DurationStyle::Short,
            DurationStyle::Spoken,
        ]
        .map(|style| locale.format_duration(span, style))
    }

    #[test]
    fn english_writes_a_clock_short_units_and_spoken_words() {
        let english = &DateLocale::ENGLISH;
        assert_eq!(every(english, 0), ["0:00", "0 s", "0 seconds"]);
        assert_eq!(every(english, 1), ["0:01", "1 s", "1 second"]);
        assert_eq!(
            every(english, 65),
            ["1:05", "1 min 5 s", "1 minute 5 seconds"]
        );
        assert_eq!(
            every(english, 3600 + 60),
            ["1:01:00", "1 h 1 min", "1 hour 1 minute"]
        );
        assert_eq!(
            every(english, 2 * 3600 + 5),
            ["2:00:05", "2 h 5 s", "2 hours 5 seconds"]
        );
    }

    #[test]
    fn german_writes_its_own_units_and_plurals() {
        let german = &DateLocale::GERMAN;
        assert_eq!(every(german, 0), ["0:00", "0 Sek.", "0 Sekunden"]);
        assert_eq!(every(german, 1), ["0:01", "1 Sek.", "1 Sekunde"]);
        assert_eq!(
            every(german, 65),
            ["1:05", "1 Min. 5 Sek.", "1 Minute 5 Sekunden"]
        );
        assert_eq!(
            every(german, 2 * 3600 + 120),
            ["2:02:00", "2 Std. 2 Min.", "2 Stunden 2 Minuten"]
        );
    }

    #[test]
    fn a_fraction_of_a_second_is_dropped_and_hours_run_past_a_day() {
        let english = &DateLocale::ENGLISH;
        let span = Duration::from_millis(59_999);
        assert_eq!(english.format_duration(span, DurationStyle::Clock), "0:59");
        let days = Duration::from_secs(100 * 3600);
        assert_eq!(
            english.format_duration(days, DurationStyle::Clock),
            "100:00:00"
        );
    }
}
