//! The typed date and time fields: each names its own value type and fills
//! the part of [`FieldOptions`] that type uses, then draws through the same
//! path as `ChronoField`.

use dioxus::prelude::*;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use super::{
    DateRange,
    chrono_field::{FieldOptions, chrono_field},
    format::{Token, WeekdayWidth, tokens, uses_twelve_hours},
    picker_field::picker_field,
    props::date_props,
};
use crate::components::common::Input;

/// Whether a day passes `min`, `max` and `exclude_date`.
pub(super) fn day_allowed(
    min: Option<NaiveDate>,
    max: Option<NaiveDate>,
    exclude_date: Option<Callback<NaiveDate, bool>>,
) -> impl Fn(NaiveDate) -> bool + Copy + 'static {
    move |day| {
        !(min.is_some_and(|min| day < min)
            || max.is_some_and(|max| day > max)
            || exclude_date.is_some_and(|exclude| exclude.call(day)))
    }
}

/// The time format a field shows when the caller names none: `theme`, the
/// formats' own, adjusted for `with_seconds` and `twelve_hour`; separators,
/// padding and text stay.
pub(super) fn time_format(theme: &str, twelve_hour: Option<bool>, with_seconds: bool) -> String {
    let twelve = twelve_hour.unwrap_or_else(|| uses_twelve_hours(theme));
    if !with_seconds && twelve == uses_twelve_hours(theme) {
        return theme.to_string();
    }
    let mut parts = tokens(theme);
    // Seconds go after the minutes, split off as the minutes are from the hour.
    // `with_seconds: false` leaves a theme's own seconds alone.
    if with_seconds
        && !parts
            .iter()
            .any(|part| matches!(part, Token::Second { .. }))
    {
        let minute = parts
            .iter()
            .position(|part| matches!(part, Token::Minute { .. }));
        let separator = match minute.and_then(|at| at.checked_sub(1)).map(|at| parts[at]) {
            Some(Token::Literal(separator)) => separator,
            _ => ":",
        };
        let at = minute
            .or_else(|| last_clock_part(&parts))
            .map_or(parts.len(), |at| at + 1);
        parts.splice(
            at..at,
            [Token::Literal(separator), Token::Second { padded: true }],
        );
    }
    if twelve != uses_twelve_hours(theme) {
        for part in &mut parts {
            if let Token::Hour { twelve: hour, .. } = part {
                *hour = twelve;
            }
        }
        if twelve {
            // `A` goes after the last clock part, as in `h:mm A`.
            let at = last_clock_part(&parts).map_or(parts.len(), |at| at + 1);
            parts.splice(
                at..at,
                [Token::Literal(" "), Token::Meridiem { upper: true }],
            );
        } else {
            // A meridiem goes with the blank that set it apart.
            while let Some(at) = parts
                .iter()
                .position(|part| matches!(part, Token::Meridiem { .. }))
            {
                parts.remove(at);
                let blank = |part: Option<&Token>| matches!(part, Some(Token::Literal(text)) if text.trim().is_empty());
                if at > 0 && blank(parts.get(at - 1)) {
                    parts.remove(at - 1);
                } else if at == 0 && blank(parts.first()) {
                    parts.remove(0);
                }
            }
        }
    }
    parts.iter().map(token_text).collect()
}

/// The index of the last hour, minute or second.
fn last_clock_part(parts: &[Token]) -> Option<usize> {
    parts.iter().rposition(|part| {
        matches!(
            part,
            Token::Hour { .. } | Token::Minute { .. } | Token::Second { .. }
        )
    })
}

/// A token written back as format text. Literal text with letters goes in
/// brackets, so it cannot read as a token.
fn token_text(token: &Token) -> String {
    let pick = |yes: bool, long: &str, short: &str| if yes { long } else { short }.to_string();
    match *token {
        Token::Literal(text) if text.chars().any(char::is_alphabetic) => format!("[{text}]"),
        Token::Literal(text) => text.to_string(),
        Token::Year => "YYYY".into(),
        Token::Month { padded } => pick(padded, "MM", "M"),
        Token::MonthName { short } => pick(short, "MMM", "MMMM"),
        Token::Day { padded } => pick(padded, "DD", "D"),
        Token::WeekdayName(WeekdayWidth::Min) => "dd".into(),
        Token::WeekdayName(WeekdayWidth::Short) => "ddd".into(),
        Token::WeekdayName(WeekdayWidth::Long) => "dddd".into(),
        Token::Hour {
            padded,
            twelve: false,
        } => pick(padded, "HH", "H"),
        Token::Hour {
            padded,
            twelve: true,
        } => pick(padded, "hh", "h"),
        Token::Minute { padded } => pick(padded, "mm", "m"),
        Token::Second { padded } => pick(padded, "ss", "s"),
        Token::Meridiem { upper } => pick(upper, "A", "a"),
    }
}

date_props! {
    field DateFieldProps(NaiveDate, NaiveDate): format, limits, exclude_date, today, calendar, close_on_change
}

/// A text field holding a day, with a `DatePicker` in a dropdown.
///
/// Controlled: it renders `value` and asks for a new one through `onchange`.
/// Typed text stays as typed until the field blurs or Enter is pressed; then
/// it is read leniently against `format`. Text that is not an accepted day
/// stays, and the field shows an error.
#[component]
pub fn DateField(props: DateFieldProps) -> Element {
    let options = FieldOptions {
        format: props.format.clone(),
        min: props.min,
        max: props.max,
        exclude_date: props.exclude_date,
        close_on_change: props.close_on_change,
        calendar: props.calendar.clone(),
        days: props.days,
        ..FieldOptions::default()
    };
    chrono_field::<NaiveDate>(picker_field!(props, props.today), options)
}

date_props! {
    field TimeFieldProps(NaiveTime, NaiveTime): time_format, limits, clock
}

/// A text field holding a time, with a `TimePicker` in a dropdown. Typing reads `13:05`, `1:05 pm`, `1305`.
#[component]
pub fn TimeField(props: TimeFieldProps) -> Element {
    let options = FieldOptions {
        time_format: props.time_format.clone(),
        min: props.min,
        max: props.max,
        variant: props.variant.clone(),
        with_seconds: props.with_seconds,
        step: props.step,
        twelve_hour: props.twelve_hour,
        ..FieldOptions::default()
    };
    chrono_field::<NaiveTime>(picker_field!(props, None), options)
}

date_props! {
    field DateTimeFieldProps(NaiveDateTime, NaiveDateTime): format, time_format, limits, exclude_date, today, clock, calendar
}

/// A text field holding a day and a time. The dropdown picks the day, then
/// the time, and a `SegmentedControl` goes back.
/// The text shows the day, then the time after a space.
#[component]
pub fn DateTimeField(props: DateTimeFieldProps) -> Element {
    let options = FieldOptions {
        format: props.format.clone(),
        time_format: props.time_format.clone(),
        min: props.min,
        max: props.max,
        exclude_date: props.exclude_date,
        variant: props.variant.clone(),
        with_seconds: props.with_seconds,
        step: props.step,
        twelve_hour: props.twelve_hour,
        calendar: props.calendar.clone(),
        days: props.days,
        ..FieldOptions::default()
    };
    chrono_field::<NaiveDateTime>(picker_field!(props, props.today), options)
}

date_props! {
    field DateRangeFieldProps(DateRange<NaiveDate>, NaiveDate): format, limits, exclude_date, today, columns, close_on_change
}

/// A text field holding a range of days, with two months in a dropdown.
/// An `end` of `None` is a range
/// still being picked. The text joins both days with the theme's
/// `range_separator`; typing takes `–`, ` - ` or ` to ` between them.
#[component]
pub fn DateRangeField(props: DateRangeFieldProps) -> Element {
    let options = FieldOptions {
        format: props.format.clone(),
        min: props.min,
        max: props.max,
        exclude_date: props.exclude_date,
        columns: props.columns,
        close_on_change: props.close_on_change,
        ..FieldOptions::default()
    };
    chrono_field::<DateRange<NaiveDate>>(picker_field!(props, props.today), options)
}

date_props! {
    field DateTimeRangeFieldProps(DateRange<NaiveDateTime>, NaiveDateTime): format, time_format, limits, exclude_date, today, clock
}

/// A text field holding a range of moments. The dropdown picks the start - a
/// day, then a time - before the end; `SegmentedControl`s switch sides and
/// parts.
#[component]
pub fn DateTimeRangeField(props: DateTimeRangeFieldProps) -> Element {
    let options = FieldOptions {
        format: props.format.clone(),
        time_format: props.time_format.clone(),
        min: props.min,
        max: props.max,
        exclude_date: props.exclude_date,
        variant: props.variant.clone(),
        with_seconds: props.with_seconds,
        step: props.step,
        twelve_hour: props.twelve_hour,
        ..FieldOptions::default()
    };
    chrono_field::<DateRange<NaiveDateTime>>(picker_field!(props, props.today), options)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn adjusted(theme: &'static str, twelve_hour: Option<bool>, with_seconds: bool) -> String {
        time_format(theme, twelve_hour, with_seconds)
    }

    #[test]
    fn seconds_and_the_clock_adjust_a_dotted_theme() {
        assert_eq!(adjusted("HH.mm", None, false), "HH.mm");
        assert_eq!(adjusted("HH.mm", None, true), "HH.mm.ss");
        assert_eq!(adjusted("HH.mm", Some(false), true), "HH.mm.ss");
        assert_eq!(adjusted("HH.mm", Some(true), false), "hh.mm A");
        assert_eq!(adjusted("HH.mm", Some(true), true), "hh.mm.ss A");
        assert_eq!(adjusted("H.mm [Uhr]", Some(true), true), "h.mm.ss A [Uhr]");
    }

    #[test]
    fn a_twelve_hour_theme_drops_its_meridiem_for_the_24_hour_clock() {
        assert_eq!(adjusted("h:mm A", None, true), "h:mm:ss A");
        assert_eq!(adjusted("h:mm A", Some(false), false), "H:mm");
        assert_eq!(adjusted("a h:mm", Some(false), true), "H:mm:ss");
        assert_eq!(adjusted("HH:mm:ss", Some(false), false), "HH:mm:ss");
    }
}
