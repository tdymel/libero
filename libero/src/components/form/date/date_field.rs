use chrono::NaiveDate;
use dioxus::prelude::*;

use super::{
    DatePicker,
    calendar::DateLevel,
    date_value::{DateValue, PickerOptions},
    fields::time_format,
    format::uses_twelve_hours,
    picker_field::{DropdownArgs, Formats, picker_field, use_picker_field},
};
use crate::{
    components::{FieldName, Input, Validators, common::field_props},
    hooks::use_theme,
    theme::TimePickerVariant,
};

field_props! {
    extends(input);
    pub struct DateFieldProps<V: DateValue> {
        /// The value in the field; strictly controlled. `None` is the empty
        /// field. Its type picks the dropdown. Inside a `Form`, a path `name`
        /// can supply it instead.
        #[props(default)]
        value: Option<V>,
        /// Called with the value the caller should hold next: when typed text
        /// is committed - on blur or Enter - and on every pick. Emptied text
        /// commits `None`.
        #[props(default)]
        onchange: Option<EventHandler<Option<V>>>,
        /// Rules over the value, shown once the field loses focus or its form
        /// is submitted.
        #[props(default, into)]
        validate: Validators<Option<V>>,
        /// How the text shows a day, in dayjs tokens. Defaults to the theme's
        /// `DateDefaults::format`. Typing is lenient either way: only the
        /// order of day, month and year has to match.
        #[props(default, into)]
        format: Option<String>,
        /// How the text shows a time. Defaults to the theme's
        /// `DateDefaults::time_format`, adjusted for `with_seconds` and
        /// `twelve_hour`.
        #[props(default, into)]
        time_format: Option<String>,
        /// The earliest value that can be picked or typed - a range's
        /// earliest end.
        #[props(default)]
        min: Option<V::Bound>,
        /// The latest value that can be picked or typed - a range's latest
        /// end.
        #[props(default)]
        max: Option<V::Bound>,
        /// Days that cannot be picked or typed. Ignored for a time.
        #[props(default)]
        exclude_date: Option<Callback<NaiveDate, bool>>,
        /// The day marked as today, and the year typed text without one falls
        /// back to. Unset, the platform clock answers after mount.
        #[props(default)]
        today: Option<NaiveDate>,
        /// A clock's look. Only for values with a time.
        #[props(default, into)]
        variant: Input<TimePickerVariant>,
        /// Seconds in the text and the clock. Only for values with a time.
        #[props(default)]
        with_seconds: Option<bool>,
        /// Minutes between the offered minutes. Only for values with a time.
        #[props(default)]
        step: Option<u8>,
        /// A 12-hour clock. Defaults to whether the time format is one.
        #[props(default)]
        twelve_hour: Option<bool>,
        /// Months side by side. Only for a range of days.
        #[props(default)]
        columns: Option<usize>,
        /// Picking a day, or a range's end, closes the dropdown.
        #[props(default)]
        close_on_change: Option<bool>,
        /// What the field posts as - the value in ISO 8601, whatever the text
        /// shows. A path also binds it to the surrounding `Form`'s value when
        /// it has no `onchange`.
        #[props(default, into)]
        name: FieldName<Option<V>>,
        #[props(default, into)]
        placeholder: Option<String>,
    }
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
#[component]
pub fn DateField<V: DateValue>(props: DateFieldProps<V>) -> Element {
    let theme = use_theme();
    let names = &theme.date;
    let with_seconds = props.with_seconds.unwrap_or(false);
    let time = props
        .time_format
        .clone()
        .unwrap_or_else(|| time_format(names, props.twelve_hour, with_seconds));
    let options = PickerOptions {
        min: props.min,
        max: props.max,
        exclude_date: props.exclude_date,
        allow_deselect: false,
        columns: props.columns,
        level: DateLevel::Day,
        variant: props.variant.copied_or(theme.time_picker.variant),
        with_seconds,
        step: props.step,
        twelve_hour: props
            .twelve_hour
            .unwrap_or_else(|| uses_twelve_hours(&time)),
    };
    let close = props
        .close_on_change
        .unwrap_or(theme.date_field.close_on_change);
    let formats = Formats {
        date: props
            .format
            .clone()
            .unwrap_or_else(|| names.format.to_string()),
        time,
        names,
    };
    use_picker_field(
        picker_field!(props, formats, props.today),
        move |value: V| value.accepts(&options),
        move |args: DropdownArgs<V>| {
            rsx! {
                DatePicker::<V> {
                    value: args.value,
                    onchange: move |next: Option<V>| {
                        args.pick.call((next, close && V::closes(next)));
                    },
                    min: options.min,
                    max: options.max,
                    exclude_date: options.exclude_date,
                    columns: options.columns,
                    today: args.today,
                    size: args.size,
                    variant: Input::Value(options.variant),
                    with_seconds: options.with_seconds,
                    step: options.step,
                    twelve_hour: options.twelve_hour,
                    focusable: false,
                }
            }
        },
    )
}
