use dioxus::prelude::*;

use super::{
    calendar::DateLevel,
    date_value::{DateValue, PickerArgs, PickerOptions},
    format::uses_twelve_hours,
    props::date_props,
};
use crate::{components::Input, hooks::use_theme};

date_props! {
    picker DatePickerProps<V: DateValue>(V, V::Bound): limits, exclude_date, allow_deselect, columns, level, calendar, today, clock
}

/// One picker for every date and time value - the value's type picks what it
/// draws: a month of days for `NaiveDate`, a clock for `NaiveTime`, a day then
/// a time for `NaiveDateTime`, and both twice over for a `DateRange` of
/// either. `level` turns a `NaiveDate` picker into a month or a year picker.
///
/// Controlled: it renders `value` and asks for a new one through `onchange`.
/// A typed `value` alone does not name `V` - dioxus passes props through
/// `SuperInto` - so a typed `onchange` or a turbofish has to.
///
/// Props a value type does not use are ignored: the clock props need a time,
/// `exclude_date` and `columns` days, `level` and `allow_deselect` a single
/// `NaiveDate`, `calendar` and `days` a day or a date-time.
#[component]
pub fn DatePicker<V: DateValue>(props: DatePickerProps<V>) -> Element {
    let theme = use_theme();
    V::picker(PickerArgs {
        value: props.value,
        onchange: props.onchange,
        options: PickerOptions {
            min: props.min,
            max: props.max,
            exclude_date: props.exclude_date,
            allow_deselect: props.allow_deselect.unwrap_or(false),
            columns: props.columns,
            level: props.level.unwrap_or(DateLevel::Day),
            variant: props.variant.copied_or(theme.time_picker.variant),
            with_seconds: props.with_seconds.unwrap_or(false),
            step: props.step,
            twelve_hour: props
                .twelve_hour
                .unwrap_or_else(|| uses_twelve_hours(theme.date.time_format)),
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
