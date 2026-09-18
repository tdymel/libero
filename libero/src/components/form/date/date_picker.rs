use dioxus::prelude::*;

use super::{
    calendar::DateLevel,
    date_value::{DateValue, PickerArgs, PickerOptions, use_ignored_props_warning},
    format::uses_twelve_hours,
    props::date_props,
};
use crate::{
    components::Input,
    hooks::{use_formats, use_theme},
};

date_props! {
    picker DatePickerProps<V: DateValue>(V, V::Bound): limits, exclude_date, allow_deselect, columns, level, calendar, today, clock
}

/// One picker for every date and time value - the value's type picks what it
/// draws: a month of days for `NaiveDate`, a clock for `NaiveTime`, a day then
/// a time for `NaiveDateTime`, both twice over for a `DateRange` of either,
/// and a column each for the hours, minutes and seconds of a `TimeDelta`.
/// `level` turns a `NaiveDate` picker into a month or a year picker.
///
/// Controlled: it renders `value` and asks for a new one through `onchange`.
/// A typed `value` alone does not name `V` - dioxus passes props through
/// `SuperInto` - so a typed `onchange` or a turbofish has to.
///
/// Props a value type does not use are ignored, with a warning in debug
/// builds: the clock props need a time (a `TimeDelta` reads `with_seconds`
/// and `step`), `today` and `exclude_date` days,
/// `columns` a day or a range of days, `level` and `allow_deselect` a single
/// `NaiveDate`, `calendar` and `days` a day or a date-time. A month or year
/// picker also ignores `exclude_date`, `columns`, `calendar`, `days` and
/// `allow_deselect`.
#[component]
pub fn DatePicker<V: DateValue>(props: DatePickerProps<V>) -> Element {
    let theme = use_theme();
    let time_format = use_formats().time;
    let level = props.level.unwrap_or(DateLevel::Day);
    use_ignored_props_warning::<V>(
        "DatePicker",
        &[
            ("exclude_date", props.exclude_date.is_some()),
            ("allow_deselect", props.allow_deselect.is_some()),
            ("columns", props.columns.is_some()),
            ("level", props.level.is_some()),
            ("calendar", props.calendar.as_ref().is_some()),
            ("days", props.days.is_some()),
            ("today", props.today.is_some()),
            ("variant", props.variant.as_ref().is_some()),
            ("with_seconds", props.with_seconds.is_some()),
            ("step", props.step.is_some()),
            ("twelve_hour", props.twelve_hour.is_some()),
        ],
        // Months and years have no excluded days, columns, mini calendar or
        // deselect.
        match level {
            DateLevel::Day => &[],
            _ => &[
                "exclude_date",
                "allow_deselect",
                "columns",
                "calendar",
                "days",
            ],
        },
    );
    V::picker(PickerArgs {
        value: props.value,
        onchange: props.onchange,
        options: PickerOptions {
            min: props.min,
            max: props.max,
            exclude_date: props.exclude_date,
            allow_deselect: props.allow_deselect.unwrap_or(false),
            columns: props.columns,
            level,
            variant: props.variant.copied_or(theme.time_picker.variant),
            with_seconds: props.with_seconds.unwrap_or(false),
            step: props.step,
            twelve_hour: props
                .twelve_hour
                .unwrap_or_else(|| uses_twelve_hours(time_format)),
            calendar: props.calendar.copied_or(theme.date_picker.calendar),
            days: props.days.unwrap_or(theme.date_picker.days),
        },
        today: props.today,
        size: props.size,
        focusable: props.focusable.unwrap_or(true),
        name: props.name,
        class: props.class,
        sx: props.sx,
        states: props.states,
        attributes: props.attributes,
    })
}
