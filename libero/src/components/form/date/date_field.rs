use chrono::NaiveDate;
use dioxus::prelude::*;

use super::{
    DatePicker,
    calendar::DateLevel,
    date_value::{DateValue, PickerOptions, use_ignored_props_warning, used},
    fields::time_format,
    format::uses_twelve_hours,
    picker_field::{DropdownArgs, Formats, PickerField, picker_field, use_picker_field},
    props::date_props,
};
use crate::{
    components::Input,
    hooks::use_theme,
    theme::{CalendarVariant, TimePickerVariant},
};

date_props! {
    field DateFieldProps<V: DateValue>(V, V::Bound): format, time_format, limits, exclude_date, today, clock, calendar, columns, close_on_change
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
/// Props a value type does not use are ignored, with a warning in debug
/// builds: the clock props and `time_format` need a time, `format`, `today`
/// and `exclude_date` a day, `columns` and `close_on_change` a day or a range
/// of days, `calendar` and `days` a day or a date-time.
#[component]
pub fn DateField<V: DateValue>(props: DateFieldProps<V>) -> Element {
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
        ],
        &[],
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
    let names = &theme.date;
    let with_seconds = options.with_seconds.unwrap_or(false);
    let time = options
        .time_format
        .unwrap_or_else(|| time_format(names, options.twelve_hour, with_seconds));
    let picker = PickerOptions {
        min: options.min,
        max: options.max,
        exclude_date: options.exclude_date,
        allow_deselect: false,
        columns: options.columns,
        level: DateLevel::Day,
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
        date: options.format.unwrap_or_else(|| names.format.to_string()),
        time,
        names,
    };
    use_picker_field(
        field,
        formats,
        move |value: V| value.accepts(&picker),
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
                    exclude_date: used::<V, _>("exclude_date", picker.exclude_date).flatten(),
                    columns: used::<V, _>("columns", picker.columns).flatten(),
                    today: used::<V, _>("today", args.today).flatten(),
                    size: args.size,
                    variant: used::<V, _>("variant", picker.variant).map_or(Input::None, Input::Value),
                    with_seconds: used::<V, _>("with_seconds", picker.with_seconds),
                    step: used::<V, _>("step", picker.step).flatten(),
                    twelve_hour: used::<V, _>("twelve_hour", picker.twelve_hour),
                    calendar: used::<V, _>("calendar", picker.calendar).map_or(Input::None, Input::Value),
                    days: used::<V, _>("days", picker.days),
                }
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveTime};
    use dioxus::prelude::*;

    use crate::{
        LiberoProvider,
        components::{
            Input,
            form::date::{DateField, DateLevel, DatePicker},
        },
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
