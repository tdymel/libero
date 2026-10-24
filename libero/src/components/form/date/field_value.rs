//! What a field asks of its value beyond the picker: text both ways and the limits' errors.

use std::rc::Rc;

use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime};
use dioxus::prelude::*;

use super::{
    DateRange,
    calendar::{DateLevel, first_of_month},
    chrono_field::refusal_message,
    date_value::{DateValue, PickerOptions},
    flows::{short_day, short_days},
    format::{format_date, format_time},
    parse::{Unreadable, parse_at},
    parse_time::{parse_date_time, parse_time},
    range::{Iso, split_range},
};
use crate::{
    components::form::FieldError,
    localization::{DateLocale, Localization},
};

/// The formats a field shows its value in, and reads typed text against.
#[derive(Clone)]
pub struct Formats {
    pub date: String,
    pub time: String,
    pub names: &'static DateLocale,
    /// Between a range's two ends, from `localization::Formats`.
    pub range_separator: &'static str,
    /// What a `NaiveDate` stands for: typed text reads as its first day.
    pub level: DateLevel,
}

/// A value a date or time field can hold.
pub trait FieldValue: Iso + PartialEq + 'static {
    /// The input's text.
    fn show(self, formats: &Formats) -> String;
    /// A narrow field's text, where `show` would overflow it; `None` keeps `show`.
    fn show_short(self, _: &Formats) -> Option<String> {
        None
    }
    /// A range of these as a narrow field shows it.
    fn show_short_range(_: DateRange<Self>, _: &Formats) -> Option<String> {
        None
    }
    /// Typed text back. `current` fills in whatever the text leaves out.
    fn read(
        text: &str,
        formats: &Formats,
        current: Option<Self>,
        today: Option<NaiveDate>,
    ) -> Result<Self, Unreadable>;
    /// Whether the value counts as one: a range still missing its end posts nothing and
    /// fails a `required` rule.
    fn is_complete(self) -> bool {
        true
    }
    /// Names the dropdown's dialog.
    fn dialog_label(names: &DateLocale) -> &'static str {
        names.date_label
    }
    /// The error for text `read` cannot read.
    fn unreadable(names: &DateLocale) -> &'static str {
        names.invalid_date
    }
}

impl FieldValue for NaiveDate {
    fn show(self, formats: &Formats) -> String {
        format_date(self, &formats.date, formats.names)
    }

    fn show_short_range(range: DateRange<Self>, formats: &Formats) -> Option<String> {
        Some(short_days(range, formats.names))
    }

    fn read(
        text: &str,
        formats: &Formats,
        current: Option<Self>,
        today: Option<NaiveDate>,
    ) -> Result<Self, Unreadable> {
        let fallback_year = today.or(current).map(|day| day.year());
        parse_at(
            text,
            &formats.date,
            formats.names,
            fallback_year,
            formats.level,
        )
    }
}

impl FieldValue for NaiveTime {
    fn show(self, formats: &Formats) -> String {
        format_time(self, &formats.time, formats.names)
    }

    fn read(
        text: &str,
        formats: &Formats,
        _: Option<Self>,
        _: Option<NaiveDate>,
    ) -> Result<Self, Unreadable> {
        parse_time(text, &formats.time, formats.names)
    }

    fn dialog_label(names: &DateLocale) -> &'static str {
        names.time_label
    }
}

impl FieldValue for NaiveDateTime {
    fn show(self, formats: &Formats) -> String {
        format!(
            "{} {}",
            self.date().show(formats),
            self.time().show(formats)
        )
    }

    /// `14 Sep 09:00 – 16 Sep 17:00`, years only when they differ.
    fn show_short_range(range: DateRange<Self>, formats: &Formats) -> Option<String> {
        let years = range
            .end
            .is_some_and(|end| end.year() != range.start.year());
        let moment = |at: Self| {
            let day = short_day(at.date(), years, formats.names);
            format!("{day} {}", at.time().show(formats))
        };
        Some(match range.end {
            Some(end) if end.date() == range.start.date() => {
                format!("{} – {}", moment(range.start), end.time().show(formats))
            }
            Some(end) => format!("{} – {}", moment(range.start), moment(end)),
            None => format!("{} –", moment(range.start)),
        })
    }

    fn read(
        text: &str,
        formats: &Formats,
        current: Option<Self>,
        today: Option<NaiveDate>,
    ) -> Result<Self, Unreadable> {
        let fallback_year = today
            .or(current.map(|value| value.date()))
            .map(|day| day.year());
        parse_date_time(
            text,
            &formats.date,
            &formats.time,
            formats.names,
            fallback_year,
            current.map(|value| value.time()),
        )
    }
}

impl<T: FieldValue + Ord> FieldValue for DateRange<T> {
    fn show(self, formats: &Formats) -> String {
        match self.end {
            Some(end) => format!(
                "{}{}{}",
                self.start.show(formats),
                formats.range_separator,
                end.show(formats)
            ),
            None => self.start.show(formats),
        }
    }

    fn show_short(self, formats: &Formats) -> Option<String> {
        T::show_short_range(self, formats)
    }

    fn read(
        text: &str,
        formats: &Formats,
        current: Option<Self>,
        today: Option<NaiveDate>,
    ) -> Result<Self, Unreadable> {
        let (start, end) = split_range(text, formats.range_separator);
        let start = T::read(start, formats, current.map(|range| range.start), today)?;
        let end = end
            .map(|end| {
                let current_end = current.and_then(|range| range.end).or(Some(start));
                T::read(end, formats, current_end, today)
            })
            .transpose()?;
        Ok(DateRange::new(start, end).ordered())
    }

    fn is_complete(self) -> bool {
        self.end.is_some()
    }

    fn dialog_label(names: &DateLocale) -> &'static str {
        T::dialog_label(names)
    }
}

/// Which rule a typed value broke, so the field can say which.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Refusal {
    /// Before `min` or after `max`.
    OutOfRange,
    /// On a day `exclude_date` refuses.
    Excluded,
}

/// `Err(OutOfRange)` for a value outside `min` and `max`.
pub(super) fn within<T: PartialOrd>(
    value: T,
    min: Option<T>,
    max: Option<T>,
) -> Result<(), Refusal> {
    match min.is_some_and(|min| value < min) || max.is_some_and(|max| value > max) {
        true => Err(Refusal::OutOfRange),
        false => Ok(()),
    }
}

fn date_time_bound(bound: NaiveDateTime, formats: &Formats, names: &DateLocale) -> String {
    format!(
        "{} {}",
        format_date(bound.date(), &formats.date, names),
        format_time(bound.time(), &formats.time, names)
    )
}

/// `Err(Excluded)` for a day `exclude_date` refuses.
fn available(
    day: NaiveDate,
    exclude_date: Option<Callback<NaiveDate, bool>>,
) -> Result<(), Refusal> {
    match exclude_date.is_some_and(|exclude| exclude.call(day)) {
        true => Err(Refusal::Excluded),
        false => Ok(()),
    }
}

/// The part of [`DateValue`] only a field calls.
pub(super) trait SealedField: FieldValue {
    /// Whether a typed or picked value passes `min`, `max` and `exclude_date`.
    fn accepts(self, options: &PickerOptions<<Self as DateValue>::Bound>) -> Result<(), Refusal>
    where
        Self: DateValue;

    /// `min` and `max` with the value type's defaults, for an error that names them.
    fn limits(
        min: Option<<Self as DateValue>::Bound>,
        max: Option<<Self as DateValue>::Bound>,
    ) -> (
        Option<<Self as DateValue>::Bound>,
        Option<<Self as DateValue>::Bound>,
    )
    where
        Self: DateValue,
    {
        (min, max)
    }

    /// A bound as the field shows it in `names`' language, for an error that names it.
    fn show_bound(
        bound: <Self as DateValue>::Bound,
        formats: &Formats,
        names: &DateLocale,
    ) -> String
    where
        Self: DateValue;

    /// The error for a value `accepts` refused, naming the bounds it set; worded at render.
    fn refusal(
        self,
        refusal: Refusal,
        min: Option<<Self as DateValue>::Bound>,
        max: Option<<Self as DateValue>::Bound>,
        formats: &Formats,
    ) -> Rc<dyn FieldError>
    where
        Self: DateValue,
    {
        let formats = formats.clone();
        Rc::new(move |locale: &Localization| {
            refusal_message::<Self>(refusal, min, max, &formats, &locale.date)
        })
    }

    /// Whether a pick leaves nothing more to pick, so a field may close its dropdown.
    fn closes(value: Option<Self>) -> bool;
}

impl SealedField for NaiveDate {
    fn accepts(self, options: &PickerOptions<NaiveDate>) -> Result<(), Refusal> {
        let (min, max) = (options.min, options.max);
        // As the picker's grid: a month or year that `min` and `max` reach.
        match options.level {
            DateLevel::Day => {
                within(self, min, max).and_then(|()| available(self, options.exclude_date))
            }
            DateLevel::Month => within(first_of_month(self), min.map(first_of_month), max),
            DateLevel::Year => within(
                self.year(),
                min.map(|min| min.year()),
                max.map(|max| max.year()),
            ),
        }
    }

    fn show_bound(bound: NaiveDate, formats: &Formats, names: &DateLocale) -> String {
        format_date(bound, &formats.date, names)
    }

    fn closes(_: Option<Self>) -> bool {
        true
    }
}

impl SealedField for NaiveTime {
    fn accepts(self, options: &PickerOptions<NaiveTime>) -> Result<(), Refusal> {
        within(self, options.min, options.max)
    }

    fn show_bound(bound: NaiveTime, formats: &Formats, names: &DateLocale) -> String {
        format_time(bound, &formats.time, names)
    }

    fn closes(_: Option<Self>) -> bool {
        false
    }
}

impl SealedField for NaiveDateTime {
    fn accepts(self, options: &PickerOptions<NaiveDateTime>) -> Result<(), Refusal> {
        within(self, options.min, options.max)
            .and_then(|()| available(self.date(), options.exclude_date))
    }

    fn show_bound(bound: NaiveDateTime, formats: &Formats, names: &DateLocale) -> String {
        date_time_bound(bound, formats, names)
    }

    fn closes(_: Option<Self>) -> bool {
        false
    }
}

impl SealedField for DateRange<NaiveDate> {
    fn accepts(self, options: &PickerOptions<NaiveDate>) -> Result<(), Refusal> {
        // A range picks days whatever `level` says.
        let options = PickerOptions {
            level: DateLevel::Day,
            ..*options
        };
        self.start.accepts(&options)?;
        self.end.map_or(Ok(()), |end| end.accepts(&options))
    }

    fn show_bound(bound: NaiveDate, formats: &Formats, names: &DateLocale) -> String {
        format_date(bound, &formats.date, names)
    }

    fn closes(value: Option<Self>) -> bool {
        value.is_some_and(|range| range.end.is_some())
    }
}

impl SealedField for DateRange<NaiveDateTime> {
    fn accepts(self, options: &PickerOptions<NaiveDateTime>) -> Result<(), Refusal> {
        self.start.accepts(options)?;
        self.end.map_or(Ok(()), |end| end.accepts(options))
    }

    fn show_bound(bound: NaiveDateTime, formats: &Formats, names: &DateLocale) -> String {
        date_time_bound(bound, formats, names)
    }

    fn closes(_: Option<Self>) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_narrow_range_shows_short() {
        let formats = Formats {
            date: "D. MMMM YYYY".to_string(),
            time: "HH:mm".to_string(),
            names: &DateLocale::ENGLISH,
            range_separator: " – ",
            level: DateLevel::Day,
        };
        let day = |month, day| NaiveDate::from_ymd_opt(2026, month, day).expect("a real day");
        let at = |month, date, hour| day(month, date).and_hms_opt(hour, 0, 0).expect("a time");
        let days = DateRange::new(day(9, 14), Some(day(9, 16)));
        assert_eq!(days.show_short(&formats).as_deref(), Some("14–16 Sep"));
        let moments = DateRange::new(at(9, 14, 9), Some(at(9, 16, 17)));
        assert_eq!(
            moments.show_short(&formats).as_deref(),
            Some("14 Sep 09:00 – 16 Sep 17:00")
        );
        let one_day = DateRange::new(at(9, 14, 9), Some(at(9, 14, 17)));
        assert_eq!(
            one_day.show_short(&formats).as_deref(),
            Some("14 Sep 09:00 – 17:00")
        );
        // A single value keeps its full text.
        assert_eq!(day(9, 14).show_short(&formats), None);
    }

    fn wave_dash_formats() -> Formats {
        Formats {
            date: "YYYY年M月D日".to_string(),
            time: crate::localization::Formats::AMERICAN.time.to_string(),
            names: &DateLocale::ENGLISH,
            range_separator: " ～ ",
            level: DateLevel::Day,
        }
    }

    #[test]
    fn ranges_read_back_what_they_show_with_the_theme_separator() {
        let formats = wave_dash_formats();
        let day = |day| NaiveDate::from_ymd_opt(2026, 9, day).expect("a real day");
        let range = DateRange::new(day(1), Some(day(5)));
        let text = range.show(&formats);
        assert_eq!(text, "2026年9月1日 ～ 2026年9月5日");
        assert_eq!(DateRange::read(&text, &formats, None, None), Ok(range));

        let moments = DateRange::new(
            day(1).and_hms_opt(9, 0, 0).expect("a real time"),
            Some(day(5).and_hms_opt(17, 30, 0).expect("a real time")),
        );
        let text = moments.show(&formats);
        assert_eq!(DateRange::read(&text, &formats, None, None), Ok(moments));
    }

    #[test]
    fn months_and_years_read_back_what_they_show() {
        let names = &DateLocale::ENGLISH;
        let american = crate::localization::Formats::AMERICAN;
        let at = |date: &str, level| Formats {
            date: date.to_string(),
            time: american.time.to_string(),
            names,
            range_separator: american.range_separator,
            level,
        };
        let day = |month, day| NaiveDate::from_ymd_opt(2026, month, day).expect("a real day");
        let month = at((american.date)(DateLevel::Month), DateLevel::Month);
        assert_eq!(day(9, 1).show(&month), "September 2026");
        for text in ["September 2026", "sep 2026", "9/2026", "09.2026"] {
            assert_eq!(
                NaiveDate::read(text, &month, None, None),
                Ok(day(9, 1)),
                "{text}"
            );
        }
        // Typed text lands on the month's first day.
        let text = day(9, 25).show(&month);
        assert_eq!(NaiveDate::read(&text, &month, None, None), Ok(day(9, 1)));

        let year = at((american.date)(DateLevel::Year), DateLevel::Year);
        assert_eq!(day(9, 25).show(&year), "2026");
        assert_eq!(NaiveDate::read("2026", &year, None, None), Ok(day(1, 1)));

        let japanese = at("YYYY年M月", DateLevel::Month);
        assert_eq!(day(9, 1).show(&japanese), "2026年9月");
        assert_eq!(
            NaiveDate::read("2026年9月", &japanese, None, None),
            Ok(day(9, 1))
        );
    }
}
