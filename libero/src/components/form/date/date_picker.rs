use chrono::NaiveDate;
use dioxus::prelude::*;

use super::{
    calendar::DateLevel,
    date_value::{DateValue, PickerArgs, PickerOptions},
    format::uses_twelve_hours,
};
use crate::{
    components::{Input, common::base_props},
    hooks::use_theme,
    theme::{Size, TimePickerVariant},
};

base_props! {
    pub struct DatePickerProps<V: DateValue> {
        /// The picked value; strictly controlled. Its type picks the picker.
        #[props(default)]
        value: Option<V>,
        /// Called with the value the caller should hold next.
        #[props(default)]
        onchange: Option<EventHandler<Option<V>>>,
        /// The earliest value that can be picked - a range's earliest end.
        #[props(default)]
        min: Option<V::Bound>,
        /// The latest value that can be picked - a range's latest end.
        #[props(default)]
        max: Option<V::Bound>,
        /// Days that cannot be picked, on top of `min` and `max`. A `Callback`
        /// always compares equal, so changing only this closure does not
        /// redraw the picker. Ignored for a time, a month and a year.
        #[props(default)]
        exclude_date: Option<Callback<NaiveDate, bool>>,
        /// Clicking the picked day again clears it. Only for a day.
        #[props(default)]
        allow_deselect: Option<bool>,
        /// Months side by side. Only for a day or a range of days.
        #[props(default)]
        columns: Option<usize>,
        /// Whether a `NaiveDate` is picked as a day, a month or a year. Days
        /// by default; ignored for every other value.
        #[props(default)]
        level: Option<DateLevel>,
        /// The day marked as today. Unset, the platform clock answers after
        /// mount - on the web; elsewhere no day is marked.
        #[props(default)]
        today: Option<NaiveDate>,
        #[props(default, into)]
        size: Input<Size>,
        /// Columns of numbers, or a clock face. Only for values with a time.
        #[props(default, into)]
        variant: Input<TimePickerVariant>,
        /// A seconds column. Only for values with a time.
        #[props(default)]
        with_seconds: Option<bool>,
        /// Minutes between the offered minutes. Only for values with a time.
        #[props(default)]
        step: Option<u8>,
        /// A 12-hour clock. Defaults to whether the theme's
        /// `DateDefaults::time_format` is one.
        #[props(default)]
        twelve_hour: Option<bool>,
        /// Emits a hidden input of that name, posting the value as ISO 8601.
        #[props(default, into)]
        name: Option<String>,
        /// `false` keeps the picker out of the tab order - for a picker inside
        /// a dropdown whose text input must keep focus. On by default.
        #[props(default)]
        focusable: Option<bool>,
    }
}

/// One picker for every date and time value - the value's type picks what it
/// draws: a month of days for `NaiveDate`, a clock for `NaiveTime`, a day then
/// a time for `NaiveDateTime`, and both twice over for a `DateRange` of
/// either. `level` turns a `NaiveDate` picker into a month or a year picker.
///
/// Controlled: it renders `value` and asks for a new one through `onchange`.
/// A typed `value` alone does not name `V` - dioxus passes props through
/// `SuperInto` - so a typed `onchange` or a turbofish has to.
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
