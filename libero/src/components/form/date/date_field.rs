use chrono::NaiveDate;
use dioxus::prelude::*;

use super::{
    DatePicker,
    calendar::DateLevel,
    date_value::{DateValue, PickerOptions, Refusal, use_ignored_props_warning, used},
    fields::time_format,
    format::uses_twelve_hours,
    picker_field::{DropdownArgs, Formats, PickerField, picker_field, use_picker_field},
    props::date_props,
};
use crate::{
    components::Input,
    hooks::{use_formats, use_localization, use_theme},
    localization::fill,
    theme::{CalendarVariant, TimePickerVariant},
};

date_props! {
    field DateFieldProps<V: DateValue>(V, V::Bound): format, time_format, limits, exclude_date, today, clock, calendar, columns, close_on_change, level
}

/// One text field for every date and time value, with the [`DatePicker`](super::DatePicker)
/// its type calls for in a dropdown - Mantine's `DateInput`, `TimeInput`,
/// `DateTimePicker` and `DatePickerInput` in one.
///
/// Controlled: it renders `value` and asks for a new one through `onchange`.
/// Typed text stays as typed until the field blurs or Enter is pressed; then
/// it is read leniently. Text that is not an accepted value stays, and the
/// field shows an error. A typed `value` alone does not name `V`, so a typed
/// `onchange` or a turbofish has to.
///
/// `level` makes a `NaiveDate` field a month or a year field: the text reads
/// `(Formats::date)(level)` (`MMMM YYYY` or `YYYY` in both formats) unless
/// `format` says otherwise, the value is the month's first day or the year's
/// January 1, and the dropdown opens on that grid.
///
/// ```no_run
/// # use chrono::NaiveDate;
/// # use dioxus::prelude::*;
/// # use libero::components::{DateField, DateLevel};
/// # fn app() -> Element {
/// let mut month = use_signal(|| NaiveDate::from_ymd_opt(2026, 9, 1));
/// rsx! {
///     DateField::<NaiveDate> {
///         label: "Billing month",
///         level: DateLevel::Month,
///         value: month(),
///         onchange: move |next| month.set(next),
///     }
/// }
/// # }
/// ```
///
/// Props a value type does not use are ignored, with a warning in debug
/// builds: the clock props and `time_format` need a time, `format`, `today`
/// and `exclude_date` a day, `columns` and `close_on_change` a day or a range
/// of days, `calendar` and `days` a day or a date-time, `level` a single
/// `NaiveDate`. A month or year field also ignores `exclude_date`, `columns`,
/// `calendar` and `days`.
#[component]
pub fn DateField<V: DateValue>(props: DateFieldProps<V>) -> Element {
    let level = used::<V, _>("level", props.level)
        .flatten()
        .unwrap_or(DateLevel::Day);
    use_ignored_props_warning::<V>(
        "DateField",
        &[
            ("format", props.format.is_some()),
            ("time_format", props.time_format.is_some()),
            ("exclude_date", props.exclude_date.is_some()),
            ("today", props.today.is_some()),
            ("variant", props.variant.as_ref().is_some()),
            ("with_seconds", props.with_seconds.is_some()),
            ("step", props.step.is_some()),
            ("twelve_hour", props.twelve_hour.is_some()),
            ("calendar", props.calendar.as_ref().is_some()),
            ("days", props.days.is_some()),
            ("columns", props.columns.is_some()),
            ("close_on_change", props.close_on_change.is_some()),
            ("level", props.level.is_some()),
        ],
        // A month or year dropdown has no excluded days, columns or mini
        // calendar.
        match level {
            DateLevel::Day => &[],
            _ => &["exclude_date", "columns", "calendar", "days"],
        },
    );
    let options = FieldOptions {
        format: props.format.clone(),
        time_format: props.time_format.clone(),
        min: props.min,
        max: props.max,
        exclude_date: props.exclude_date,
        columns: props.columns,
        variant: props.variant.clone(),
        with_seconds: props.with_seconds,
        step: props.step,
        twelve_hour: props.twelve_hour,
        close_on_change: props.close_on_change,
        calendar: props.calendar.clone(),
        days: props.days,
        level,
    };
    date_field(picker_field!(props, props.today), options)
}

/// What a field's text and dropdown are drawn from, besides the props every
/// field shares. `DateField` fills all of it; a typed field fills what its
/// value type uses and leaves the rest unset.
pub(super) struct FieldOptions<B: 'static> {
    pub format: Option<String>,
    pub time_format: Option<String>,
    pub min: Option<B>,
    pub max: Option<B>,
    pub exclude_date: Option<Callback<NaiveDate, bool>>,
    pub columns: Option<usize>,
    pub variant: Input<TimePickerVariant>,
    pub with_seconds: Option<bool>,
    pub step: Option<u8>,
    pub twelve_hour: Option<bool>,
    pub close_on_change: Option<bool>,
    pub calendar: Input<CalendarVariant>,
    pub days: Option<usize>,
    pub level: DateLevel,
}

impl<B> Default for FieldOptions<B> {
    fn default() -> Self {
        Self {
            format: None,
            time_format: None,
            min: None,
            max: None,
            exclude_date: None,
            columns: None,
            variant: Input::None,
            with_seconds: None,
            step: None,
            twelve_hour: None,
            close_on_change: None,
            calendar: Input::None,
            days: None,
            level: DateLevel::Day,
        }
    }
}

/// The one path from a field's props to its text and its dropdown, so every
/// field hands the picker the same props. Calls hooks: only from a component
/// body.
pub(super) fn date_field<V: DateValue>(
    field: PickerField<'_, V>,
    options: FieldOptions<V::Bound>,
) -> Element {
    let theme = use_theme();
    let names = &use_localization().date;
    let conventions = use_formats();
    let with_seconds = options.with_seconds.unwrap_or(false);
    let time = options
        .time_format
        .unwrap_or_else(|| time_format(conventions.time, options.twelve_hour, with_seconds));
    let level = options.level;
    let picker = PickerOptions {
        min: options.min,
        max: options.max,
        exclude_date: options.exclude_date,
        allow_deselect: false,
        columns: options.columns,
        level,
        variant: options.variant.copied_or(theme.time_picker.variant),
        with_seconds,
        step: options.step,
        twelve_hour: options
            .twelve_hour
            .unwrap_or_else(|| uses_twelve_hours(&time)),
        calendar: options.calendar.copied_or(theme.date_picker.calendar),
        days: options.days.unwrap_or(theme.date_picker.days),
    };
    let close = options
        .close_on_change
        .unwrap_or(theme.date_field.close_on_change);
    let formats = Formats {
        date: options
            .format
            .unwrap_or_else(|| (conventions.date)(level).to_string()),
        time,
        names,
        range_separator: conventions.range_separator,
        level,
    };
    // Only a day level passes the props a month or year dropdown drops.
    let day = level == DateLevel::Day;
    let bounds = formats.clone();
    use_picker_field(
        field,
        formats,
        move |value: V| {
            value
                .accepts(&picker)
                .map_err(|refusal| refusal_message::<V>(refusal, picker.min, picker.max, &bounds))
        },
        // Only the props `V` reads, so the picker has nothing to warn about.
        move |args: DropdownArgs<V>| {
            rsx! {
                DatePicker::<V> {
                    value: args.value,
                    onchange: move |next: Option<V>| {
                        args.pick.call((next, close && V::closes(next)));
                    },
                    min: picker.min,
                    max: picker.max,
                    exclude_date: used::<V, _>("exclude_date", picker.exclude_date).flatten().filter(|_| day),
                    columns: used::<V, _>("columns", picker.columns).flatten().filter(|_| day),
                    level: used::<V, _>("level", level),
                    today: used::<V, _>("today", args.today).flatten(),
                    size: args.size,
                    variant: used::<V, _>("variant", picker.variant).map_or(Input::None, Input::Value),
                    with_seconds: used::<V, _>("with_seconds", picker.with_seconds),
                    step: used::<V, _>("step", picker.step).flatten(),
                    twelve_hour: used::<V, _>("twelve_hour", picker.twelve_hour),
                    calendar: used::<V, _>("calendar", picker.calendar).filter(|_| day).map_or(Input::None, Input::Value),
                    days: used::<V, _>("days", picker.days).filter(|_| day),
                }
            }
        },
    )
}

/// The error for a value the field refuses, naming the bounds it set in the
/// field's own format.
fn refusal_message<V: DateValue>(
    refusal: Refusal,
    min: Option<V::Bound>,
    max: Option<V::Bound>,
    formats: &Formats,
) -> String {
    let names = formats.names;
    let show = |bound| V::show_bound(bound, formats);
    let (min, max) = (min.map(show), max.map(show));
    match (refusal, min, max) {
        (Refusal::Excluded, ..) => names.unavailable.to_string(),
        (_, Some(min), Some(max)) => fill(names.between, &[("min", &min), ("max", &max)]),
        (_, Some(min), None) => fill(names.on_or_after, &[("min", &min)]),
        (_, None, Some(max)) => fill(names.on_or_before, &[("max", &max)]),
        (_, None, None) => names.invalid_date.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, NaiveDate, NaiveTime};
    use dioxus::prelude::*;

    use super::{Formats, PickerOptions, refusal_message};
    use crate::{
        LiberoProvider,
        components::{
            Input,
            form::date::{DateField, DateLevel, DatePicker, date_value::Sealed},
        },
        localization::DateLocale,
        theme::CalendarVariant,
        utils::take_warnings,
    };

    fn warnings_of(app: fn() -> Element) -> Vec<String> {
        take_warnings();
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        take_warnings()
    }

    #[test]
    fn a_time_field_warns_about_exclude_date() {
        let warnings = warnings_of(|| {
            rsx! {
                LiberoProvider {
                    DateField::<NaiveTime> { exclude_date: |_: NaiveDate| false }
                }
            }
        });
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].contains("`exclude_date`"), "{warnings:?}");
    }

    #[test]
    fn a_day_field_does_not_warn_about_its_own_props() {
        let warnings = warnings_of(|| {
            rsx! {
                LiberoProvider {
                    DateField::<NaiveDate> {
                        format: "YYYY-MM-DD",
                        exclude_date: |_: NaiveDate| false,
                        today: NaiveDate::from_ymd_opt(2026, 9, 25).unwrap(),
                        calendar: Input::Value(CalendarVariant::Mini),
                        days: 5,
                        columns: 2,
                        close_on_change: false,
                    }
                }
            }
        });
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    #[test]
    fn a_month_field_shows_its_month_and_warns_about_columns() {
        let warnings = warnings_of(|| {
            rsx! {
                LiberoProvider {
                    DateField::<NaiveDate> {
                        level: DateLevel::Month,
                        value: NaiveDate::from_ymd_opt(2026, 9, 1),
                        columns: 2,
                    }
                }
            }
        });
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].contains("`columns`"), "{warnings:?}");

        let html = dioxus_ssr::render_element(rsx! {
            LiberoProvider {
                DateField::<NaiveDate> { level: DateLevel::Year, value: NaiveDate::from_ymd_opt(2026, 1, 1) }
            }
        });
        assert!(html.contains(r#"value="2026""#), "{html}");
    }

    /// Todo 536: each refusal names its rule, the bounds in the field's format.
    #[test]
    fn a_refusal_names_the_bound_it_missed() {
        let american = crate::localization::Formats::AMERICAN;
        let day = |day| NaiveDate::from_ymd_opt(2026, 3, day).unwrap();
        let at = |level| Formats {
            date: (american.date)(level).to_string(),
            time: american.time.to_string(),
            names: &DateLocale::ENGLISH,
            range_separator: american.range_separator,
            level,
        };
        let refused = |value: NaiveDate, min, max, level| {
            let options = PickerOptions {
                min,
                max,
                exclude_date: Some(Callback::new(|day: NaiveDate| day.day() == 7)),
                level,
                ..PickerOptions::default()
            };
            value
                .accepts(&options)
                .map_err(|refusal| refusal_message::<NaiveDate>(refusal, min, max, &at(level)))
        };
        let mut dom = VirtualDom::new(|| rsx! {});
        dom.rebuild_in_place();
        // `Callback::new` needs a scope.
        dom.in_scope(ScopeId::ROOT, || {
            assert_eq!(
                refused(day(1), Some(day(5)), None, DateLevel::Day),
                Err("Must be on or after March 5, 2026".into())
            );
            assert_eq!(
                refused(day(20), None, Some(day(9)), DateLevel::Day),
                Err("Must be on or before March 9, 2026".into())
            );
            assert_eq!(
                refused(day(1), Some(day(5)), Some(day(9)), DateLevel::Day),
                Err("Must be between March 5, 2026 and March 9, 2026".into())
            );
            assert_eq!(
                refused(day(7), Some(day(5)), Some(day(9)), DateLevel::Day),
                Err("That date is not available".into())
            );
            assert_eq!(refused(day(6), Some(day(5)), None, DateLevel::Day), Ok(()));
            // A month field names its bound as a month.
            assert_eq!(
                refused(
                    NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
                    Some(day(5)),
                    None,
                    DateLevel::Month
                ),
                Err("Must be on or after March 2026".into())
            );
        });
    }

    #[test]
    fn a_time_field_warns_about_level() {
        let warnings = warnings_of(|| {
            rsx! {
                LiberoProvider {
                    DateField::<NaiveTime> { level: DateLevel::Month }
                }
            }
        });
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].contains("`level`"), "{warnings:?}");
    }

    #[test]
    fn a_month_picker_warns_about_columns() {
        let warnings = warnings_of(|| {
            rsx! {
                LiberoProvider {
                    DatePicker::<NaiveDate> { level: DateLevel::Month, columns: 2 }
                }
            }
        });
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].contains("`columns`"), "{warnings:?}");
    }
}
