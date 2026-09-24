use dioxus::prelude::*;

use super::{
    calendar::DateLevel,
    date_value::{DateValue, PickerArgs, PickerOptions, use_ignored_props_warning},
    format::uses_twelve_hours,
    props::date_props,
};
use crate::{
    components::common::{Input, parts_enum},
    hooks::{use_formats, use_theme},
};

parts_enum! {
    /// The inner parts of every date and time picker, for its `parts` prop. Each
    /// says which value draws it: a `NaiveDate` or `DateRange` of days the calendar,
    /// a `NaiveTime` the clock, a `TimeDelta` the duration's columns, a date-time
    /// the calendar and then the clock.
    pub enum ChronoPickerPart {
        /// Calendar: the row over a month, a year or a decade, with paging buttons and title.
        Header = "header" => "& [data-slot='header']",
        /// Calendar: a paging button, in the header or beside the mini calendar's row.
        Nav = "nav" => "& [data-slot='nav']",
        /// Calendar: the month, year or decade heading, a button that climbs a level.
        Title = "title" => "& [data-slot='title']",
        /// Days: the months side by side, or the mini calendar's row.
        Months = "months" => "& [data-slot='months']",
        /// Days: a weekday name over a month's columns.
        Weekday = "weekday" => "& [data-slot='weekday']",
        /// Days: a day button.
        Day = "day" => "& [data-slot='day']",
        /// Mini calendar: the month over a day's number.
        Month = "month" => "& [data-slot='day'] > [data-slot='month']",
        /// Days, `columns` over 1: an empty cell instead of a neighbour's day.
        Blank = "blank" => "& [data-slot='blank']",
        /// Months and years: their grid.
        Cells = "cells" => "& [data-slot='cells']",
        /// Months and years: a month or a year button.
        Cell = "cell" => "& [data-slot='cell']",
        /// Mini calendar: the row of days between its paging buttons.
        Strip = "strip" => "& [data-slot='strip']",
        /// Digital clock, duration: the columns.
        Columns = "columns" => "& [data-slot='columns']",
        /// Digital clock, duration: one column, a spinbutton.
        Spin = "spin" => "& [data-slot='spin']",
        /// Digital clock, duration: a column's value.
        Value = "value" => "& [data-slot='spin'] > [data-slot='value']",
        /// Digital clock, duration: the faded values above and below a column's value.
        Neighbour = "neighbour" => "& [data-slot='spin'] > [data-slot='neighbour']",
        /// Digital clock: the `:` between columns.
        Separator = "separator" => "& [data-slot='separator']",
        /// Duration: the unit after each column.
        Unit = "unit" => "& [data-slot='unit']",
        /// Analog clock: the digits over the face, buttons that pick the hand.
        Readout = "readout" => "& [data-slot='readout']",
        /// Analog clock: the face, a slider.
        Face = "face" => "& [data-slot='face']",
        /// Analog clock: a number on the face.
        Mark = "mark" => "& [data-slot='mark']",
        /// Analog clock: the ring of ticks for steps finer than the marks.
        Ticks = "ticks" => "& [data-slot='ticks']",
        /// Analog clock: the hand.
        Hand = "hand" => "& [data-slot='hand']",
        /// Analog clock: the dot the hand turns on.
        Pivot = "pivot" => "& [data-slot='pivot']",
    }
}

date_props! {
    picker ChronoPickerProps<V: DateValue>(V, V::Bound): limits, exclude_date, allow_deselect, columns, level, calendar, today, clock
}

/// One picker for every date and time value; the value's type picks what it draws.
///
/// A typed `value` alone does not name `V` (props go through `SuperInto`): a typed
/// `onchange` or a turbofish does. Props the type does not use warn in debug builds.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::chrono::NaiveTime;
/// # use libero::components::ChronoPicker;
/// # fn app() -> Element {
/// let mut time = use_signal(|| None::<NaiveTime>);
/// rsx! { ChronoPicker { value: time(), onchange: move |v: Option<NaiveTime>| time.set(v) } }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/chrono-picker>
#[component]
pub fn ChronoPicker<V: DateValue>(props: ChronoPickerProps<V>) -> Element {
    let theme = use_theme();
    let time_format = use_formats().time;
    let level = props.level.unwrap_or(DateLevel::Day);
    use_ignored_props_warning::<V>(
        "ChronoPicker",
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
            calendar: props.calendar.copied_or(theme.chrono_picker.calendar),
            days: props.days.unwrap_or(theme.chrono_picker.days),
        },
        today: props.today,
        size: props.size,
        focusable: props.focusable.unwrap_or(true),
        name: props.name,
        class: props.class,
        sx: props.sx,
        parts: props.parts,
        states: props.states,
        attributes: props.attributes,
    })
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveTime};

    use super::*;
    use crate::components::{
        common::{Part, part_table},
        form::{DatePicker, MonthPicker, TimePicker},
    };

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        let slots: Vec<_> = part_table::<ChronoPickerPart>()
            .into_iter()
            .map(|(slot, _)| slot)
            .collect();

        assert_eq!(
            slots,
            [
                "header",
                "nav",
                "title",
                "months",
                "weekday",
                "day",
                "month",
                "blank",
                "cells",
                "cell",
                "strip",
                "columns",
                "spin",
                "value",
                "neighbour",
                "separator",
                "unit",
                "readout",
                "face",
                "mark",
                "ticks",
                "hand",
                "pivot",
            ]
        );
        for part in ChronoPickerPart::ALL {
            let target = format!("[data-slot='{}']", part.slot());
            assert!(part.selector().starts_with("& "), "{part:?}");
            assert!(part.selector().ends_with(&target), "{part:?}");
        }
    }

    /// Every slot of the enum is drawn by one of the pickers, so no row documents nothing.
    /// `Ticks` shows only once the minute hand is picked.
    #[test]
    fn every_part_is_drawn() {
        let html = dioxus_ssr::render_element(rsx! {
            crate::LiberoProvider {
                DatePicker { value: NaiveDate::from_ymd_opt(2026, 9, 14), columns: 2 }
                DatePicker { value: NaiveDate::from_ymd_opt(2026, 9, 14), calendar: "mini" }
                MonthPicker {}
                TimePicker { value: NaiveTime::from_hms_opt(9, 30, 0), variant: "digital" }
                TimePicker { value: NaiveTime::from_hms_opt(9, 30, 0), variant: "analog" }
                ChronoPicker::<chrono::TimeDelta> {}
            }
        });

        for part in ChronoPickerPart::ALL {
            let slot = format!("data-slot=\"{}\"", part.slot());
            assert!(
                *part == ChronoPickerPart::Ticks || html.contains(&slot),
                "{part:?} not drawn"
            );
        }
    }
}
