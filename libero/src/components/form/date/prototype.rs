//! A prototype of one date field for every value: the value's type picks the
//! dropdown, where the five public fields each name their own.

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use dioxus::prelude::*;

use super::{
    DatePicker, DateRange, DateRangePicker, TimePicker,
    fields::{day_allowed, moment_allowed, time_format},
    flows::{DateTimeFlow, DateTimeRangeFlow},
    format::uses_twelve_hours,
    picker_field::{DropdownArgs, FieldValue, Formats, picker_field, use_picker_field},
};
use crate::{
    components::{FieldName, Input, Validators, common::field_props},
    hooks::use_theme,
    theme::TimePickerVariant,
};

/// Everything a dropdown needs besides the value, resolved once by the field.
#[derive(Clone, Copy)]
pub struct DropdownOptions<B: 'static> {
    pub min: Option<B>,
    pub max: Option<B>,
    pub exclude_date: Option<Callback<NaiveDate, bool>>,
    pub variant: TimePickerVariant,
    pub with_seconds: bool,
    pub step: Option<u8>,
    pub twelve_hour: bool,
    pub columns: Option<usize>,
    pub close_on_change: bool,
}

/// A value [`DateFieldPrototype`] can hold: `NaiveDate`, `NaiveTime`,
/// `NaiveDateTime`, and a `DateRange` of days or of moments. Sealed - its
/// methods take libero's own unnameable types.
pub trait DateValue: FieldValue {
    /// What `min` and `max` are: the value's own type, or a range's end type.
    type Bound: Copy + PartialOrd + 'static;

    /// Whether a typed or picked value passes `min`, `max` and `exclude_date`.
    fn accepts(self, options: &DropdownOptions<Self::Bound>) -> bool;

    /// The picker the dropdown shows.
    fn dropdown(args: DropdownArgs<Self>, options: DropdownOptions<Self::Bound>) -> Element;
}

impl DateValue for NaiveDate {
    type Bound = NaiveDate;

    fn accepts(self, options: &DropdownOptions<NaiveDate>) -> bool {
        day_allowed(options.min, options.max, options.exclude_date)(self)
    }

    fn dropdown(args: DropdownArgs<Self>, options: DropdownOptions<NaiveDate>) -> Element {
        rsx! {
            DatePicker {
                value: args.value,
                min: options.min,
                max: options.max,
                exclude_date: options.exclude_date,
                today: args.today,
                size: args.size,
                focusable: false,
                onchange: move |day| args.pick.call((day, options.close_on_change)),
            }
        }
    }
}

impl DateValue for NaiveTime {
    type Bound = NaiveTime;

    fn accepts(self, options: &DropdownOptions<NaiveTime>) -> bool {
        !(options.min.is_some_and(|min| self < min) || options.max.is_some_and(|max| self > max))
    }

    fn dropdown(args: DropdownArgs<Self>, options: DropdownOptions<NaiveTime>) -> Element {
        rsx! {
            TimePicker {
                value: args.value,
                variant: Input::Value(options.variant),
                with_seconds: options.with_seconds,
                step: options.step,
                twelve_hour: options.twelve_hour,
                min: options.min,
                max: options.max,
                size: args.size,
                focusable: false,
                onchange: move |time| args.pick.call((time, false)),
            }
        }
    }
}

impl DateValue for NaiveDateTime {
    type Bound = NaiveDateTime;

    fn accepts(self, options: &DropdownOptions<NaiveDateTime>) -> bool {
        moment_allowed(options.min, options.max, options.exclude_date)(self)
    }

    fn dropdown(args: DropdownArgs<Self>, options: DropdownOptions<NaiveDateTime>) -> Element {
        rsx! {
            DateTimeFlow {
                value: args.value,
                onpick: move |moment| args.pick.call((moment, false)),
                min: options.min,
                max: options.max,
                exclude_date: options.exclude_date,
                today: args.today,
                size: args.size,
                variant: options.variant,
                with_seconds: options.with_seconds,
                step: options.step,
                twelve_hour: options.twelve_hour,
            }
        }
    }
}

impl DateValue for DateRange<NaiveDate> {
    type Bound = NaiveDate;

    fn accepts(self, options: &DropdownOptions<NaiveDate>) -> bool {
        self.start.accepts(options) && self.end.is_none_or(|end| end.accepts(options))
    }

    fn dropdown(args: DropdownArgs<Self>, options: DropdownOptions<NaiveDate>) -> Element {
        rsx! {
            DateRangePicker {
                value: args.value,
                min: options.min,
                max: options.max,
                exclude_date: options.exclude_date,
                columns: options.columns,
                today: args.today,
                size: args.size,
                focusable: false,
                onchange: move |range: Option<DateRange<NaiveDate>>| {
                    let complete = range.is_some_and(|range| range.end.is_some());
                    args.pick.call((range, options.close_on_change && complete));
                },
            }
        }
    }
}

impl DateValue for DateRange<NaiveDateTime> {
    type Bound = NaiveDateTime;

    fn accepts(self, options: &DropdownOptions<NaiveDateTime>) -> bool {
        self.start.accepts(options) && self.end.is_none_or(|end| end.accepts(options))
    }

    fn dropdown(args: DropdownArgs<Self>, options: DropdownOptions<NaiveDateTime>) -> Element {
        rsx! {
            DateTimeRangeFlow {
                value: args.value,
                onpick: move |range| args.pick.call((range, false)),
                min: options.min,
                max: options.max,
                exclude_date: options.exclude_date,
                today: args.today,
                size: args.size,
                variant: options.variant,
                with_seconds: options.with_seconds,
                step: options.step,
                twelve_hour: options.twelve_hour,
            }
        }
    }
}

field_props! {
    extends(input);
    pub struct DateFieldPrototypeProps<V: DateValue> {
        /// The value in the field; strictly controlled. Its type picks the
        /// dropdown.
        #[props(default)]
        value: Option<V>,
        /// Called when typed text is committed - on blur or Enter - and on
        /// every pick.
        #[props(default)]
        onchange: Option<EventHandler<Option<V>>>,
        #[props(default, into)]
        validate: Validators<Option<V>>,
        /// How the text shows a day, in dayjs tokens.
        #[props(default, into)]
        format: Option<String>,
        /// How the text shows a time.
        #[props(default, into)]
        time_format: Option<String>,
        /// The earliest value - a range's earliest end.
        #[props(default)]
        min: Option<V::Bound>,
        /// The latest value - a range's latest end.
        #[props(default)]
        max: Option<V::Bound>,
        /// Days that cannot be picked or typed. Ignored for a time.
        #[props(default)]
        exclude_date: Option<Callback<NaiveDate, bool>>,
        /// The day marked as today, and the year a yearless text takes.
        #[props(default)]
        today: Option<NaiveDate>,
        /// A clock's look. Only for values with a time.
        #[props(default, into)]
        variant: Input<TimePickerVariant>,
        #[props(default)]
        with_seconds: Option<bool>,
        #[props(default)]
        step: Option<u8>,
        #[props(default)]
        twelve_hour: Option<bool>,
        /// Months side by side. Only for a range of days.
        #[props(default)]
        columns: Option<usize>,
        /// Picking a day, or a range's end, closes the dropdown.
        #[props(default)]
        close_on_change: Option<bool>,
        /// Posts the value as ISO 8601.
        #[props(default, into)]
        name: FieldName<Option<V>>,
        #[props(default, into)]
        placeholder: Option<String>,
    }
}

/// One text field for every date and time value, with the dropdown its type
/// calls for - a prototype of what could replace `DateField`, `TimeField`,
/// `DateTimeField`, `DateRangeField` and `DateTimeRangeField`.
#[component]
pub fn DateFieldPrototype<V: DateValue>(props: DateFieldPrototypeProps<V>) -> Element {
    let theme = use_theme();
    let names = &theme.date;
    let with_seconds = props.with_seconds.unwrap_or(false);
    let time = props
        .time_format
        .clone()
        .unwrap_or_else(|| time_format(names, props.twelve_hour, with_seconds));
    let options = DropdownOptions {
        min: props.min,
        max: props.max,
        exclude_date: props.exclude_date,
        variant: props.variant.copied_or(theme.time_picker.variant),
        with_seconds,
        step: props.step,
        twelve_hour: props
            .twelve_hour
            .unwrap_or_else(|| uses_twelve_hours(&time)),
        columns: props.columns,
        close_on_change: props
            .close_on_change
            .unwrap_or(theme.date_field.close_on_change),
    };
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
        move |args: DropdownArgs<V>| V::dropdown(args, options),
    )
}
