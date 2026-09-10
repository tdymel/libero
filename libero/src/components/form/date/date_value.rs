//! One trait over every date and time value. The value's type picks what is
//! drawn, so `DatePicker` and `DateField` are one component each for all of
//! them, and the typed pickers are the same drawing under a name.

use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime};
use dioxus::prelude::*;

use super::{
    DateRange,
    calendar::{Calendar, DateLevel, Selection, first_of_month},
    fields::{day_allowed, moment_allowed},
    flows::{DateTimeFlow, DateTimeRangeFlow},
    picker_field::FieldValue,
    time_picker::Clock,
};
use crate::{
    components::{ClassList, Input, States},
    hooks::use_theme,
    sx::Sx,
    theme::{CalendarVariant, Size, TimePickerVariant},
    utils::warn,
};

/// Everything a picker takes besides the value, resolved once by the
/// component that draws it. Each value type reads the part it needs.
#[derive(Clone, Copy)]
pub struct PickerOptions<B: 'static> {
    pub min: Option<B>,
    pub max: Option<B>,
    pub exclude_date: Option<Callback<NaiveDate, bool>>,
    pub allow_deselect: bool,
    pub columns: Option<usize>,
    pub level: DateLevel,
    pub variant: TimePickerVariant,
    pub with_seconds: bool,
    pub step: Option<u8>,
    pub twelve_hour: bool,
    pub calendar: CalendarVariant,
    pub days: usize,
}

impl<B> Default for PickerOptions<B> {
    fn default() -> Self {
        Self {
            min: None,
            max: None,
            exclude_date: None,
            allow_deselect: false,
            columns: None,
            level: DateLevel::Day,
            variant: TimePickerVariant::default(),
            with_seconds: false,
            step: None,
            twelve_hour: false,
            calendar: CalendarVariant::Full,
            days: 7,
        }
    }
}

/// A picker to draw: the value, its options and the shared props.
pub struct PickerArgs<V: DateValue> {
    pub value: Option<V>,
    pub onchange: Option<EventHandler<Option<V>>>,
    pub options: PickerOptions<V::Bound>,
    pub today: Option<NaiveDate>,
    pub size: Input<Size>,
    pub focusable: bool,
    pub name: Option<String>,
    pub class: Input<ClassList>,
    pub sx: Input<Sx>,
    pub states: Input<States>,
    pub attributes: Vec<Attribute>,
}

/// A value [`DatePicker`](super::DatePicker) and [`DateField`](super::DateField)
/// can hold: `NaiveDate`, `NaiveTime`, `NaiveDateTime`, and a `DateRange` of
/// days or of date-times. Sealed: the five are all there is.
#[allow(
    private_bounds,
    reason = "the seal, which keeps its machinery out of rustdoc"
)]
pub trait DateValue: Sealed {
    /// What `min` and `max` are: the value's own type, or a range's end type.
    type Bound: Copy + PartialOrd + 'static;
}

/// The part of [`DateValue`] only libero calls.
pub(super) trait Sealed: FieldValue {
    /// The type-dependent props this value type reads; debug builds warn about
    /// the others when set.
    const USES: &'static [&'static str];

    /// Whether a typed or picked value passes `min`, `max` and `exclude_date`.
    fn accepts(self, options: &PickerOptions<<Self as DateValue>::Bound>) -> bool
    where
        Self: DateValue;

    /// Whether a pick leaves nothing more to pick, so a field may close its
    /// dropdown.
    fn closes(value: Option<Self>) -> bool;

    /// Draws the picker for this value type. Calls hooks: only from a
    /// component's body, never behind a condition.
    fn picker(args: PickerArgs<Self>) -> Element
    where
        Self: DateValue;
}

/// Warns once per mount about each prop set to something `V` does not use, or
/// that `dropped` names. Debug builds only.
pub(super) fn use_ignored_props_warning<V: DateValue>(
    component: &str,
    set: &[(&'static str, bool)],
    dropped: &[&str],
) {
    use_hook(|| {
        if !cfg!(debug_assertions) {
            return;
        }
        let ignored = set
            .iter()
            .filter(|(prop, set)| *set && (!V::USES.contains(prop) || dropped.contains(prop)));
        for (prop, _) in ignored {
            warn(&format!(
                "{component}::<{}>: `{prop}` is set but ignored - the component's docs list \
                 the props each value type uses.",
                std::any::type_name::<V>()
            ));
        }
    });
}

/// `value` if `V` reads `prop`.
pub(super) fn used<V: DateValue, T>(prop: &str, value: T) -> Option<T> {
    V::USES.contains(&prop).then_some(value)
}

impl DateValue for NaiveDate {
    type Bound = NaiveDate;
}

impl Sealed for NaiveDate {
    const USES: &'static [&'static str] = &[
        "format",
        "today",
        "exclude_date",
        "calendar",
        "days",
        "columns",
        "close_on_change",
        "level",
        "allow_deselect",
    ];

    fn accepts(self, options: &PickerOptions<NaiveDate>) -> bool {
        day_allowed(options.min, options.max, options.exclude_date)(self)
    }

    fn closes(_: Option<Self>) -> bool {
        true
    }

    fn picker(args: PickerArgs<Self>) -> Element {
        let PickerArgs {
            onchange, options, ..
        } = args;
        let level = options.level;
        let size = args.size.copied_or(use_theme().date_picker.size);
        // A month is held as its first day, a year as its January 1.
        let value = match level {
            DateLevel::Day => args.value,
            DateLevel::Month => args.value.map(first_of_month),
            DateLevel::Year => args
                .value
                .and_then(|day| NaiveDate::from_ymd_opt(day.year(), 1, 1)),
        };
        let allow_deselect = options.allow_deselect && level == DateLevel::Day;
        // Only days come as a mini calendar.
        let mini = level == DateLevel::Day && options.calendar == CalendarVariant::Mini;
        // One identity across renders, so the calendar's props compare equal
        // and a re-render from above skips it.
        let onpick = use_callback(move |day: NaiveDate| {
            let next = match allow_deselect && value == Some(day) {
                true => None,
                false => Some(day),
            };
            if let Some(onchange) = &onchange {
                onchange.call(next);
            }
        });
        // Keyed by level and layout inside a one-item list: dioxus remounts a
        // keyed child whose key changes, and the calendar's own view state
        // starts over. Inline, as a nested `rsx!` costs a node.
        rsx! {
            for args in std::iter::once(args) {
                Calendar {
                    key: "{level:?}-{mini}",
                variant: if mini { CalendarVariant::Mini } else { CalendarVariant::Full },
                days: options.days,
                selection: Selection::Single(value),
                onpick,
                columns: match level {
                    DateLevel::Day => options.columns.unwrap_or(1),
                    _ => 1,
                },
                lowest: level,
                min: options.min,
                max: options.max,
                exclude_date: match level {
                    DateLevel::Day => options.exclude_date,
                    _ => None,
                },
                today: args.today,
                size,
                focusable: args.focusable,
                hidden: args.name.map(|name| (name, value.map(|day| day.to_string()).unwrap_or_default())),
                class: args.class,
                sx: args.sx,
                states: args.states,
                    attributes: args.attributes,
                }
            }
        }
    }
}

impl DateValue for NaiveTime {
    type Bound = NaiveTime;
}

impl Sealed for NaiveTime {
    const USES: &'static [&'static str] = &[
        "time_format",
        "variant",
        "with_seconds",
        "step",
        "twelve_hour",
    ];

    fn accepts(self, options: &PickerOptions<NaiveTime>) -> bool {
        !(options.min.is_some_and(|min| self < min) || options.max.is_some_and(|max| self > max))
    }

    fn closes(_: Option<Self>) -> bool {
        false
    }

    fn picker(args: PickerArgs<Self>) -> Element {
        let options = args.options;
        rsx! {
            Clock {
                value: args.value,
                onchange: args.onchange,
                variant: options.variant,
                with_seconds: options.with_seconds,
                step: options.step,
                twelve_hour: options.twelve_hour,
                min: options.min,
                max: options.max,
                size: args.size,
                focusable: args.focusable,
                name: args.name,
                class: args.class,
                sx: args.sx,
                states: args.states,
                attributes: args.attributes,
            }
        }
    }
}

impl DateValue for NaiveDateTime {
    type Bound = NaiveDateTime;
}

impl Sealed for NaiveDateTime {
    const USES: &'static [&'static str] = &[
        "format",
        "today",
        "exclude_date",
        "calendar",
        "days",
        "time_format",
        "variant",
        "with_seconds",
        "step",
        "twelve_hour",
    ];

    fn accepts(self, options: &PickerOptions<NaiveDateTime>) -> bool {
        moment_allowed(options.min, options.max, options.exclude_date)(self)
    }

    fn closes(_: Option<Self>) -> bool {
        false
    }

    fn picker(args: PickerArgs<Self>) -> Element {
        let (options, onchange) = (args.options, args.onchange);
        rsx! {
            DateTimeFlow {
                value: args.value,
                onpick: move |next| {
                    if let Some(onchange) = onchange {
                        onchange.call(next);
                    }
                },
                min: options.min,
                max: options.max,
                exclude_date: options.exclude_date,
                today: args.today,
                size: args.size,
                variant: options.variant,
                with_seconds: options.with_seconds,
                step: options.step,
                twelve_hour: options.twelve_hour,
                calendar: options.calendar,
                days: options.days,
                focusable: args.focusable,
                name: args.name,
                class: args.class,
                sx: args.sx,
                states: args.states,
                attributes: args.attributes,
            }
        }
    }
}

impl DateValue for DateRange<NaiveDate> {
    type Bound = NaiveDate;
}

impl Sealed for DateRange<NaiveDate> {
    const USES: &'static [&'static str] = &[
        "format",
        "today",
        "exclude_date",
        "columns",
        "close_on_change",
    ];

    fn accepts(self, options: &PickerOptions<NaiveDate>) -> bool {
        self.start.accepts(options) && self.end.is_none_or(|end| end.accepts(options))
    }

    fn closes(value: Option<Self>) -> bool {
        value.is_some_and(|range| range.end.is_some())
    }

    fn picker(args: PickerArgs<Self>) -> Element {
        let (value, onchange, options) = (args.value, args.onchange, args.options);
        let size = args.size.copied_or(use_theme().date_picker.size);
        // One identity across renders, as for a single day.
        let onpick = use_callback(move |day: NaiveDate| {
            if let Some(onchange) = &onchange {
                onchange.call(Some(DateRange::pick(value, day)));
            }
        });
        rsx! {
            Calendar {
                selection: Selection::Range(value),
                onpick,
                columns: options.columns.unwrap_or(2),
                lowest: DateLevel::Day,
                min: options.min,
                max: options.max,
                exclude_date: options.exclude_date,
                today: args.today,
                size,
                focusable: args.focusable,
                hidden: args.name.map(|name| (name, value.map(|range| range.to_string()).unwrap_or_default())),
                class: args.class,
                sx: args.sx,
                states: args.states,
                attributes: args.attributes,
            }
        }
    }
}

impl DateValue for DateRange<NaiveDateTime> {
    type Bound = NaiveDateTime;
}

impl Sealed for DateRange<NaiveDateTime> {
    const USES: &'static [&'static str] = &[
        "format",
        "today",
        "exclude_date",
        "time_format",
        "variant",
        "with_seconds",
        "step",
        "twelve_hour",
    ];

    fn accepts(self, options: &PickerOptions<NaiveDateTime>) -> bool {
        self.start.accepts(options) && self.end.is_none_or(|end| end.accepts(options))
    }

    fn closes(_: Option<Self>) -> bool {
        false
    }

    fn picker(args: PickerArgs<Self>) -> Element {
        let (options, onchange) = (args.options, args.onchange);
        rsx! {
            DateTimeRangeFlow {
                value: args.value,
                onpick: move |next| {
                    if let Some(onchange) = onchange {
                        onchange.call(next);
                    }
                },
                min: options.min,
                max: options.max,
                exclude_date: options.exclude_date,
                today: args.today,
                size: args.size,
                variant: options.variant,
                with_seconds: options.with_seconds,
                step: options.step,
                twelve_hour: options.twelve_hour,
                focusable: args.focusable,
                name: args.name,
                class: args.class,
                sx: args.sx,
                states: args.states,
                attributes: args.attributes,
            }
        }
    }
}
