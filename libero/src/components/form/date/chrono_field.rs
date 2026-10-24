use chrono::NaiveDate;
use dioxus::prelude::*;

use super::{
    ChronoPicker, DatePreset,
    calendar::DateLevel,
    date_value::{DateValue, DayGrid, PickerOptions, use_ignored_props_warning, used},
    field_value::{Formats, Refusal},
    fields::time_format,
    format::uses_twelve_hours,
    picker_field::{DropdownArgs, PickerField, picker_field, use_picker_field},
    props::date_props,
};
use crate::{
    components::common::Input,
    components::form::phone_sheet::narrow_query,
    hooks::{use_formats, use_localization, use_media_query, use_theme},
    localization::{DateLocale, fill},
    theme::{CalendarVariant, TimePickerVariant},
};

date_props! {
    field ChronoFieldProps<V: DateValue>(V, V::Bound): format, time_format, limits, exclude_date, today, clock, calendar, grid, presets, columns, close_on_change, level, open_to
}

/// One text field for every date and time value, with its [`ChronoPicker`] in a dropdown.
///
/// A typed `value` alone does not name `V`: a typed `onchange` or a turbofish does.
/// Props the type does not use warn in debug builds.
///
/// ```
/// # use libero::chrono::NaiveDate;
/// # use dioxus::prelude::*;
/// # use libero::components::{ChronoField, DateLevel};
/// # fn app() -> Element {
/// let mut month = use_signal(|| NaiveDate::from_ymd_opt(2026, 9, 1));
/// rsx! {
///     ChronoField::<NaiveDate> {
///         label: "Billing month",
///         level: DateLevel::Month,
///         value: month(),
///         onchange: move |next| month.set(next),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/chrono-field>
#[component]
pub fn ChronoField<V: DateValue>(props: ChronoFieldProps<V>) -> Element {
    let level = used::<V, _>("level", props.level)
        .flatten()
        .unwrap_or(DateLevel::Day);
    use_ignored_props_warning::<V>(
        "ChronoField",
        &[
            ("format", props.format.is_some()),
            ("time_format", props.time_format.is_some()),
            ("exclude_date", props.exclude_date.is_some()),
            ("today", props.today.is_some()),
            ("variant", props.variant.as_ref().is_some()),
            ("with_seconds", props.with_seconds.is_some()),
            ("step", props.step.is_some()),
            ("hour_step", props.hour_step.is_some()),
            ("second_step", props.second_step.is_some()),
            ("twelve_hour", props.twelve_hour.is_some()),
            ("week_numbers", props.week_numbers.is_some()),
            ("weekend_days", props.weekend_days.is_some()),
            ("hide_weekdays", props.hide_weekdays.is_some()),
            ("hide_outside_days", props.hide_outside_days.is_some()),
            ("presets", !props.presets.is_empty()),
            ("calendar", props.calendar.as_ref().is_some()),
            ("days", props.days.is_some()),
            ("columns", props.columns.is_some()),
            ("close_on_change", props.close_on_change.is_some()),
            ("level", props.level.is_some()),
            ("open_to", props.open_to.is_some()),
        ],
        match level {
            DateLevel::Day => &[],
            _ => &[
                "exclude_date",
                "columns",
                "calendar",
                "days",
                "week_numbers",
                "weekend_days",
                "hide_weekdays",
                "hide_outside_days",
                "presets",
            ],
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
        hour_step: props.hour_step,
        second_step: props.second_step,
        twelve_hour: props.twelve_hour,
        close_on_change: props.close_on_change,
        calendar: props.calendar.clone(),
        days: props.days,
        grid: DayGrid::new(
            props.week_numbers,
            props.weekend_days.as_deref(),
            props.hide_weekdays,
            props.hide_outside_days,
        ),
        level,
        open_to: props.open_to,
    };
    let presets = props.presets.clone();
    chrono_field(picker_field!(props, props.today), options, presets)
}

/// A field's type-specific props. A typed field fills what its value type uses.
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
    pub hour_step: Option<u8>,
    pub second_step: Option<u8>,
    pub twelve_hour: Option<bool>,
    pub close_on_change: Option<bool>,
    pub calendar: Input<CalendarVariant>,
    pub days: Option<usize>,
    pub grid: DayGrid,
    pub level: DateLevel,
    pub open_to: Option<DateLevel>,
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
            hour_step: None,
            second_step: None,
            twelve_hour: None,
            close_on_change: None,
            calendar: Input::None,
            days: None,
            grid: DayGrid::default(),
            level: DateLevel::Day,
            open_to: None,
        }
    }
}

/// The one path from a field's props to its text and dropdown. Calls hooks.
pub(super) fn chrono_field<V: DateValue>(
    field: PickerField<'_, V>,
    options: FieldOptions<V::Bound>,
    presets: Vec<DatePreset<V>>,
) -> Element {
    let theme = use_theme();
    let names = &use_localization().date;
    let conventions = use_formats();
    let with_seconds = options.with_seconds.unwrap_or(false);
    let time = options
        .time_format
        .unwrap_or_else(|| time_format(conventions.time, options.twelve_hour, with_seconds));
    let level = options.level;
    let (min, max) = V::limits(options.min, options.max);
    let picker = PickerOptions {
        min,
        max,
        exclude_date: options.exclude_date,
        allow_deselect: false,
        columns: options.columns,
        level,
        open_to: options.open_to,
        variant: options.variant.copied_or(theme.time_picker.variant),
        with_seconds,
        step: options.step,
        hour_step: options.hour_step,
        second_step: options.second_step,
        twelve_hour: options
            .twelve_hour
            .unwrap_or_else(|| uses_twelve_hours(&time)),
        calendar: options.calendar.copied_or(theme.chrono_picker.calendar),
        days: options.days.unwrap_or(theme.chrono_picker.days),
        grid: options.grid,
    };
    let close = options
        .close_on_change
        .unwrap_or(theme.chrono_field.close_on_change);
    let formats = Formats {
        date: options
            .format
            .unwrap_or_else(|| (conventions.date)(level).to_string()),
        time,
        names,
        range_separator: conventions.range_separator,
        level,
    };
    let narrow = use_media_query(&narrow_query());
    // Only a day level passes the props a month or year dropdown drops.
    let day = level == DateLevel::Day;
    let bounds = formats.clone();
    use_picker_field(
        field,
        formats,
        move |value: V| {
            value
                .accepts(&picker)
                .map_err(|refusal| value.refusal(refusal, picker.min, picker.max, &bounds))
        },
        // The scrolling range leaves the closing to its Save (todo 3903).
        move |next| close && !V::scrolls(&picker, *narrow.peek()) && V::closes(next),
        V::scrolls(&picker, narrow()),
        // Only the props `V` reads, so the picker has nothing to warn about.
        move |args: DropdownArgs<V>| {
            rsx! {
                ChronoPicker::<V> {
                    value: args.value,
                    onchange: args.pick,
                    min: picker.min,
                    max: picker.max,
                    exclude_date: used::<V, _>("exclude_date", picker.exclude_date).flatten().filter(|_| day),
                    columns: used::<V, _>("columns", picker.columns).flatten().filter(|_| day),
                    level: used::<V, _>("level", level),
                    open_to: used::<V, _>("open_to", picker.open_to).flatten(),
                    today: used::<V, _>("today", args.today).flatten(),
                    size: args.size,
                    variant: used::<V, _>("variant", picker.variant).map_or(Input::None, Input::Value),
                    with_seconds: used::<V, _>("with_seconds", picker.with_seconds),
                    step: used::<V, _>("step", picker.step).flatten(),
                    hour_step: used::<V, _>("hour_step", picker.hour_step).flatten(),
                    second_step: used::<V, _>("second_step", picker.second_step).flatten(),
                    twelve_hour: used::<V, _>("twelve_hour", picker.twelve_hour),
                    calendar: used::<V, _>("calendar", picker.calendar).filter(|_| day).map_or(Input::None, Input::Value),
                    days: used::<V, _>("days", picker.days).filter(|_| day),
                    week_numbers: used::<V, _>("week_numbers", picker.grid.week_numbers).filter(|_| day),
                    weekend_days: used::<V, _>("weekend_days", picker.grid.weekend_days()).filter(|_| day),
                    hide_weekdays: used::<V, _>("hide_weekdays", picker.grid.hide_weekdays).filter(|_| day),
                    hide_outside_days: used::<V, _>("hide_outside_days", picker.grid.hide_outside_days).filter(|_| day),
                    presets: used::<V, _>("presets", presets.clone()).filter(|_| day).unwrap_or_default(),
                }
            }
        },
    )
}

/// The error for a value the field refuses, naming the bounds it set in the
/// field's own format, in `names`' language.
pub(super) fn refusal_message<V: DateValue>(
    refusal: Refusal,
    min: Option<V::Bound>,
    max: Option<V::Bound>,
    formats: &Formats,
    names: &DateLocale,
) -> String {
    let show = |bound| V::show_bound(bound, formats, names);
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

    use super::{DateLocale, Formats, PickerOptions, Refusal, refusal_message};
    use crate::{
        LiberoProvider,
        components::{
            common::Input,
            form::date::{ChronoField, ChronoPicker, DateLevel, field_value::SealedField},
        },
        localization::Localization,
        theme::CalendarVariant,
        utils::warnings_of,
    };

    #[test]
    fn a_time_field_warns_about_exclude_date() {
        let warnings = warnings_of(|| {
            rsx! {
                LiberoProvider {
                    ChronoField::<NaiveTime> { exclude_date: |_: NaiveDate| false }
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
                    ChronoField::<NaiveDate> {
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
    fn a_month_field_warns_about_columns() {
        let warnings = warnings_of(|| {
            rsx! {
                LiberoProvider {
                    ChronoField::<NaiveDate> {
                        level: DateLevel::Month,
                        value: NaiveDate::from_ymd_opt(2026, 9, 1),
                        columns: 2,
                    }
                }
            }
        });
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].contains("`columns`"), "{warnings:?}");
    }

    #[test]
    fn a_month_field_shows_its_month() {
        let html = dioxus_ssr::render_element(rsx! {
            LiberoProvider {
                ChronoField::<NaiveDate> { level: DateLevel::Month, value: NaiveDate::from_ymd_opt(2026, 9, 1) }
            }
        });
        assert!(html.contains(r#"value="September 2026""#), "{html}");
    }

    #[test]
    fn a_year_field_shows_its_year() {
        let html = dioxus_ssr::render_element(rsx! {
            LiberoProvider {
                ChronoField::<NaiveDate> { level: DateLevel::Year, value: NaiveDate::from_ymd_opt(2026, 1, 1) }
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
            value.accepts(&options).map_err(|refusal| {
                refusal_message::<NaiveDate>(refusal, min, max, &at(level), &DateLocale::ENGLISH)
            })
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
        // Todo 4092: the error words itself at render, so a language switch re-words it.
        let error = day(1).refusal(
            Refusal::OutOfRange,
            Some(day(5)),
            None,
            &at(DateLevel::Month),
        );
        let german = refusal_message::<NaiveDate>(
            Refusal::OutOfRange,
            Some(day(5)),
            None,
            &at(DateLevel::Month),
            &DateLocale::GERMAN,
        );
        assert_eq!(
            error.message(&Localization::ENGLISH),
            "Must be on or after March 2026"
        );
        assert_eq!(error.message(&Localization::GERMAN), german);
        assert_ne!(german, "Must be on or after March 2026");
    }

    #[test]
    fn a_range_checks_its_days_whatever_the_level() {
        let day = |day| NaiveDate::from_ymd_opt(2026, 3, day).unwrap();
        let options = PickerOptions {
            min: Some(day(5)),
            level: DateLevel::Month,
            ..PickerOptions::default()
        };
        let range = crate::components::form::date::DateRange {
            start: day(1),
            end: Some(day(9)),
        };
        assert!(range.accepts(&options).is_err());
    }

    #[test]
    fn a_time_field_warns_about_level() {
        let warnings = warnings_of(|| {
            rsx! {
                LiberoProvider {
                    ChronoField::<NaiveTime> { level: DateLevel::Month }
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
                    ChronoPicker::<NaiveDate> { level: DateLevel::Month, columns: 2 }
                }
            }
        });
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].contains("`columns`"), "{warnings:?}");
    }
}
