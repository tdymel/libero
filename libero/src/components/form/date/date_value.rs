//! One trait over every date and time value; the value's type picks what is drawn.

use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, Weekday};
use dioxus::prelude::*;

#[cfg(feature = "date-field")]
use super::field_value::SealedField;
use super::{
    ChronoPickerPart, DatePreset, DateRange,
    calendar::{Calendar, CalendarPreset, DateLevel, Selection, first_of_month},
    duration::DurationClock,
    flows::{DateTimeFlow, DateTimeRangeFlow},
    time_picker::Clock,
};
use crate::{
    components::{
        common::{ClassList, Input, Parts, States},
        form::phone_sheet::narrow_query,
    },
    hooks::{use_media_query, use_theme},
    sx::Sx,
    theme::{CalendarVariant, Size, TimePickerVariant},
    utils::warn,
};

/// How a month of days is drawn: the `grid` prop group, resolved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DayGrid {
    pub week_numbers: bool,
    /// Bit `n` marks the weekday `n` days from Sunday.
    pub weekend: u8,
    pub hide_weekdays: bool,
    pub hide_outside_days: bool,
}

impl Default for DayGrid {
    fn default() -> Self {
        Self::new(None, None, None, None)
    }
}

impl DayGrid {
    pub(super) fn new(
        week_numbers: Option<bool>,
        weekend_days: Option<&[Weekday]>,
        hide_weekdays: Option<bool>,
        hide_outside_days: Option<bool>,
    ) -> Self {
        let weekend = weekend_days
            .unwrap_or(&[Weekday::Sat, Weekday::Sun])
            .iter()
            .fold(0, |mask, day| mask | 1 << day.num_days_from_sunday());
        Self {
            week_numbers: week_numbers.unwrap_or(false),
            weekend,
            hide_weekdays: hide_weekdays.unwrap_or(false),
            hide_outside_days: hide_outside_days.unwrap_or(false),
        }
    }

    pub(super) fn is_weekend(self, day: NaiveDate) -> bool {
        self.weekend & (1 << day.weekday().num_days_from_sunday()) != 0
    }

    pub(super) fn weekend_days(self) -> Vec<Weekday> {
        [
            Weekday::Sun,
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
        ]
        .into_iter()
        .filter(|day| self.weekend & (1 << day.num_days_from_sunday()) != 0)
        .collect()
    }
}

/// The calendar's preset buttons for `presets`, and the handler that picks one by index.
/// Calls a hook: one identity across renders, so a re-render from above skips the calendar.
fn use_calendar_presets<V: DateValue>(
    presets: &[DatePreset<V>],
    selection: impl Fn(V) -> Selection,
    onchange: Option<EventHandler<Option<V>>>,
) -> (Vec<CalendarPreset>, EventHandler<usize>) {
    let values: Vec<V> = presets.iter().map(|preset| preset.value).collect();
    let buttons = presets
        .iter()
        .map(|preset| CalendarPreset {
            label: preset.label.clone(),
            selection: selection(preset.value),
        })
        .collect();
    let onpreset = use_callback(move |index: usize| {
        if let (Some(onchange), Some(value)) = (onchange, values.get(index)) {
            onchange.call(Some(*value));
        }
    });
    (buttons, onpreset)
}

/// Everything a picker takes besides the value, resolved. Each value type reads its part.
#[derive(Clone, Copy)]
pub struct PickerOptions<B: 'static> {
    pub min: Option<B>,
    pub max: Option<B>,
    pub exclude_date: Option<Callback<NaiveDate, bool>>,
    pub allow_deselect: bool,
    pub columns: Option<usize>,
    pub level: DateLevel,
    pub open_to: Option<DateLevel>,
    pub variant: TimePickerVariant,
    pub with_seconds: bool,
    pub step: Option<u8>,
    pub hour_step: Option<u8>,
    pub second_step: Option<u8>,
    pub twelve_hour: bool,
    pub calendar: CalendarVariant,
    pub days: usize,
    pub grid: DayGrid,
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
            open_to: None,
            variant: TimePickerVariant::default(),
            with_seconds: false,
            step: None,
            hour_step: None,
            second_step: None,
            twelve_hour: false,
            calendar: CalendarVariant::Full,
            days: 7,
            grid: DayGrid::default(),
        }
    }
}

/// A picker to draw: the value, its options and the shared props.
pub struct PickerArgs<V: DateValue> {
    pub value: Option<V>,
    pub onchange: Option<EventHandler<Option<V>>>,
    pub options: PickerOptions<V::Bound>,
    pub presets: Vec<DatePreset<V>>,
    pub today: Option<NaiveDate>,
    pub size: Input<Size>,
    pub focusable: bool,
    /// Soft: `aria-disabled` cells, presses and keys swallowed, nothing posted.
    pub disabled: bool,
    pub name: Option<String>,
    pub class: Input<ClassList>,
    pub sx: Input<Sx>,
    pub parts: Input<Parts<ChronoPickerPart>>,
    pub states: Input<States>,
    pub attributes: Vec<Attribute>,
}

/// A value [`ChronoPicker`](super::ChronoPicker) and [`ChronoField`](super::ChronoField) can hold.
/// Sealed: `NaiveDate`, `NaiveTime`, `NaiveDateTime`, `DateRange` of days or date-times, `TimeDelta`.
#[allow(
    private_bounds,
    reason = "the seal, which keeps its machinery out of rustdoc"
)]
pub trait DateValue: Sealed {
    /// What `min` and `max` are: the value's own type, or a range's end type.
    type Bound: Copy + PartialOrd + 'static;
}

/// Without `date-field`, a value owes the fields nothing.
#[cfg(not(feature = "date-field"))]
pub(super) trait SealedField: Copy + PartialEq + 'static {}

#[cfg(not(feature = "date-field"))]
impl<T: Copy + PartialEq + 'static> SealedField for T {}

/// The part of [`DateValue`] only libero calls.
pub(super) trait Sealed: SealedField {
    /// The type-dependent props this value type reads; debug builds warn about the others.
    const USES: &'static [&'static str];

    /// Whether the picker scrolls its months, below `sm` (todo 3903): a field then
    /// leaves the closing to the sheet's Save.
    fn scrolls(_: &PickerOptions<<Self as DateValue>::Bound>, _narrow: bool) -> bool
    where
        Self: DateValue,
    {
        false
    }

    /// Draws the picker for this value type. Calls hooks: never behind a condition.
    fn picker(args: PickerArgs<Self>) -> Element
    where
        Self: DateValue;
}

/// Debug builds: warns once per mount about each set prop `V` does not use or `dropped` names.
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

/// A date-time flow with the props both flows take; `extra` adds the ones only one reads.
macro_rules! date_time_flow {
    ($flow:ident, $args:ident, { $($extra:tt)* }) => {{
        let onchange = $args.onchange;
        rsx! {
            $flow {
                value: $args.value,
                onpick: move |next| {
                    if let Some(onchange) = onchange {
                        onchange.call(next);
                    }
                },
                min: $args.options.min,
                max: $args.options.max,
                exclude_date: $args.options.exclude_date,
                today: $args.today,
                size: $args.size,
                variant: $args.options.variant,
                with_seconds: $args.options.with_seconds,
                step: $args.options.step,
                hour_step: $args.options.hour_step,
                second_step: $args.options.second_step,
                grid: $args.options.grid,
                twelve_hour: $args.options.twelve_hour,
                focusable: $args.focusable,
                disabled: $args.disabled,
                name: $args.name,
                class: $args.class,
                sx: $args.sx,
                parts: $args.parts,
                states: $args.states,
                attributes: $args.attributes,
                $($extra)*
            }
        }
    }};
}

/// The calendar's hidden form input: `name` posting the value, empty when unset.
fn hidden_value(name: Option<String>, value: Option<impl ToString>) -> Option<(String, String)> {
    name.map(|name| {
        (
            name,
            value.map(|value| value.to_string()).unwrap_or_default(),
        )
    })
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
        "week_numbers",
        "weekend_days",
        "hide_weekdays",
        "hide_outside_days",
        "open_to",
        "presets",
    ];

    fn picker(args: PickerArgs<Self>) -> Element {
        let PickerArgs {
            onchange, options, ..
        } = args;
        let (presets, onpreset) =
            use_calendar_presets(&args.presets, |day| Selection::Single(Some(day)), onchange);
        let level = options.level;
        let size = args.size.copied_or(use_theme().chrono_picker.size);
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
        // One identity across renders, so a re-render from above skips the calendar.
        let onpick = use_callback(move |day: NaiveDate| {
            let next = match allow_deselect && value == Some(day) {
                true => None,
                false => Some(day),
            };
            if let Some(onchange) = &onchange {
                onchange.call(next);
            }
        });
        // A one-item keyed list remounts the calendar, resetting its view, when level or layout change.
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
                    open_to: options.open_to,
                    grid: options.grid,
                    presets: presets.clone(),
                    onpreset,
                    min: options.min,
                    max: options.max,
                    exclude_date: match level {
                        DateLevel::Day => options.exclude_date,
                        _ => None,
                    },
                    today: args.today,
                    size,
                    focusable: args.focusable,
                    disabled: args.disabled,
                    hidden: hidden_value(args.name, value),
                    class: args.class,
                    sx: args.sx,
                    parts: args.parts,
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
        "hour_step",
        "second_step",
        "twelve_hour",
    ];

    fn picker(args: PickerArgs<Self>) -> Element {
        let options = args.options;
        rsx! {
            Clock {
                value: args.value,
                onchange: args.onchange,
                variant: options.variant,
                with_seconds: options.with_seconds,
                step: options.step,
                hour_step: options.hour_step,
                second_step: options.second_step,
                twelve_hour: options.twelve_hour,
                min: options.min,
                max: options.max,
                size: args.size,
                focusable: args.focusable,
                disabled: args.disabled,
                name: args.name,
                class: args.class,
                sx: args.sx,
                parts: args.parts,
                states: args.states,
                attributes: args.attributes,
            }
        }
    }
}

impl DateValue for TimeDelta {
    type Bound = TimeDelta;
}

impl Sealed for TimeDelta {
    const USES: &'static [&'static str] = &["with_seconds", "step"];

    fn picker(args: PickerArgs<Self>) -> Element {
        let options = args.options;
        rsx! {
            DurationClock {
                value: args.value,
                onchange: args.onchange,
                with_seconds: options.with_seconds,
                step: options.step,
                min: options.min,
                max: options.max,
                size: args.size,
                focusable: args.focusable,
                disabled: args.disabled,
                name: args.name,
                class: args.class,
                sx: args.sx,
                parts: args.parts,
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
        "week_numbers",
        "weekend_days",
        "hide_weekdays",
        "hide_outside_days",
        "time_format",
        "variant",
        "with_seconds",
        "step",
        "hour_step",
        "second_step",
        "twelve_hour",
    ];

    fn picker(args: PickerArgs<Self>) -> Element {
        date_time_flow!(DateTimeFlow, args, {
            calendar: args.options.calendar,
            days: args.options.days,
        })
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
        "week_numbers",
        "weekend_days",
        "hide_weekdays",
        "hide_outside_days",
        "presets",
    ];

    fn scrolls(options: &PickerOptions<NaiveDate>, narrow: bool) -> bool {
        narrow && options.columns.is_none()
    }

    fn picker(args: PickerArgs<Self>) -> Element {
        let (value, onchange, options) = (args.value, args.onchange, args.options);
        let (presets, onpreset) = use_calendar_presets(
            &args.presets,
            |range| Selection::Range(Some(range)),
            onchange,
        );
        let size = args.size.copied_or(use_theme().chrono_picker.size);
        // One identity across renders, as for a single day.
        let onpick = use_callback(move |day: NaiveDate| {
            if let Some(onchange) = &onchange {
                onchange.call(Some(DateRange::pick(value, day)));
            }
        });
        // Two months are ~520px: under `sm` one scrolling column (1.4.10, todo 3903).
        let narrow = use_media_query(&narrow_query());
        rsx! {
            Calendar {
                selection: Selection::Range(value),
                onpick,
                columns: options.columns.unwrap_or(2),
                scroll: Self::scrolls(&options, narrow()),
                lowest: DateLevel::Day,
                grid: options.grid,
                presets,
                onpreset,
                min: options.min,
                max: options.max,
                exclude_date: options.exclude_date,
                today: args.today,
                size,
                focusable: args.focusable,
                disabled: args.disabled,
                // A half range posts nothing, as DateRangeField does (todo 4253).
                hidden: hidden_value(args.name, value.filter(|range| range.end.is_some())),
                class: args.class,
                sx: args.sx,
                parts: args.parts,
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
        "week_numbers",
        "weekend_days",
        "hide_weekdays",
        "hide_outside_days",
        "time_format",
        "variant",
        "with_seconds",
        "step",
        "hour_step",
        "second_step",
        "twelve_hour",
    ];

    fn picker(args: PickerArgs<Self>) -> Element {
        date_time_flow!(DateTimeRangeFlow, args, {})
    }
}
