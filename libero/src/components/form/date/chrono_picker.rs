use dioxus::prelude::*;

use super::{
    calendar::DateLevel,
    date_value::{DateValue, DayGrid, PickerArgs, PickerOptions, use_ignored_props_warning},
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
        /// Calendar with `presets`: the column of preset buttons, a row on a phone.
        Presets = "presets" => "& [data-slot='presets']",
        /// Calendar with `presets`: a preset button.
        Preset = "preset" => "& [data-slot='preset']",
        /// Days with `week_numbers`: a week's number, and the empty cell over them.
        WeekNumber = "week-number" => "& [data-slot='week-number']",
    }
}

date_props! {
    picker ChronoPickerProps<V: DateValue>(V, V::Bound): limits, exclude_date, allow_deselect, columns, level, open_to, calendar, grid, presets, today, clock
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
            ("open_to", props.open_to.is_some()),
            ("calendar", props.calendar.as_ref().is_some()),
            ("days", props.days.is_some()),
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
        ],
        match level {
            DateLevel::Day => &[],
            _ => &[
                "exclude_date",
                "allow_deselect",
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
            open_to: props.open_to,
            variant: props.variant.copied_or(theme.time_picker.variant),
            with_seconds: props.with_seconds.unwrap_or(false),
            step: props.step,
            hour_step: props.hour_step,
            second_step: props.second_step,
            twelve_hour: props
                .twelve_hour
                .unwrap_or_else(|| uses_twelve_hours(time_format)),
            calendar: props.calendar.copied_or(theme.chrono_picker.calendar),
            days: props.days.unwrap_or(theme.chrono_picker.days),
            grid: DayGrid::new(
                props.week_numbers,
                props.weekend_days.as_deref(),
                props.hide_weekdays,
                props.hide_outside_days,
            ),
        },
        presets: props.presets,
        today: props.today,
        size: props.size,
        focusable: props.focusable.unwrap_or(true),
        disabled: props.disabled,
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
                "presets",
                "preset",
                "week-number",
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
                DatePicker {
                    week_numbers: true,
                    presets: vec![super::super::DatePreset::new("Day", NaiveDate::from_ymd_opt(2026, 9, 1).unwrap())],
                }
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

    fn january(body: Element) -> String {
        dioxus_ssr::render_element(rsx! {
            crate::LiberoProvider { {body} }
        })
    }

    /// Todo 4003: a week's number is its Thursday's ISO week, whatever day the week starts on.
    #[test]
    fn week_numbers_head_each_week() {
        let html = january(rsx! {
            DatePicker { value: NaiveDate::from_ymd_opt(2026, 1, 15), week_numbers: true }
        });
        assert!(
            html.contains(r#"role="rowheader" data-slot="week-number" aria-label="Week 1""#),
            "{html}"
        );
        assert!(html.contains(r#"aria-label="Week 2""#));
        assert!(html.contains(r#"role="columnheader" data-slot="week-number" aria-label="Week""#));
    }

    /// Todo 4003: Saturday and Sunday by default, or the days given.
    #[test]
    fn weekend_days_are_marked() {
        let value = NaiveDate::from_ymd_opt(2026, 1, 15);
        let weekend = |html: String| html.matches(r#"data-weekend="true""#).count();
        assert_eq!(weekend(january(rsx! { DatePicker { value } })), 12);
        assert_eq!(
            weekend(january(
                rsx! { DatePicker { value, weekend_days: vec![chrono::Weekday::Fri] } }
            )),
            6
        );
    }

    /// Todo 4003: hidden outside days leave blanks; hidden weekday names stay in the grid.
    #[test]
    fn outside_days_and_weekday_names_can_hide() {
        let html = january(rsx! {
            DatePicker { value: NaiveDate::from_ymd_opt(2026, 1, 15), hide_outside_days: true, hide_weekdays: true }
        });
        assert!(!html.contains(r#"data-outside="true""#));
        assert!(html.contains(r#"data-slot="blank""#));
        assert!(html.contains(r#"role="row" data-hidden="true""#));
        assert!(html.contains(r#"data-slot="weekday""#));
    }

    /// Todo 4003: `open_to` picks the first view; one below `level` is ignored.
    #[test]
    fn open_to_picks_the_first_view() {
        let years = january(rsx! { DatePicker { open_to: DateLevel::Year } });
        assert!(years.contains(r#"data-slot="cell""#) && !years.contains(r#"data-slot="day""#));
        let months = january(rsx! { MonthPicker { open_to: DateLevel::Day } });
        assert!(months.contains(r#"data-slot="cell""#) && !months.contains(r#"data-slot="day""#));
    }

    /// Todo 3988: a preset equal to the value is pressed; one outside the limits is disabled.
    #[test]
    fn presets_say_which_one_is_picked() {
        let day = |d| NaiveDate::from_ymd_opt(2026, 1, d).unwrap();
        let html = january(rsx! {
            DatePicker {
                value: Some(day(15)),
                min: day(10),
                presets: vec![
                    super::super::DatePreset::new("Picked", day(15)),
                    super::super::DatePreset::new("Other", day(20)),
                    super::super::DatePreset::new("Early", day(1)),
                ],
            }
        });
        let preset = |label: &str| {
            let tag = html.split(&format!(">{label}<")).next().unwrap_or_default();
            tag.rsplit("<button").next().unwrap_or_default().to_string()
        };
        assert!(
            preset("Picked").contains(r#"aria-pressed="true""#),
            "{html}"
        );
        assert!(preset("Other").contains(r#"aria-pressed="false""#));
        assert!(preset("Early").contains("disabled"));
        assert!(!preset("Other").contains("disabled"));
    }

    /// Todos 2307, 2308: a gridcell wraps each month and year button; the decade title is text.
    #[test]
    fn month_and_year_cells_wrap_their_buttons() {
        let html = dioxus_ssr::render_element(rsx! {
            crate::LiberoProvider {
                MonthPicker { value: NaiveDate::from_ymd_opt(2026, 3, 1) }
                super::super::YearPicker { value: NaiveDate::from_ymd_opt(2026, 1, 1) }
            }
        });
        let button_cells = html
            .split("<button")
            .skip(1)
            .filter(|tag| {
                tag.split('>')
                    .next()
                    .is_some_and(|tag| tag.contains("gridcell"))
            })
            .count();
        assert_eq!(button_cells, 0, "{html}");
        assert_eq!(
            html.matches(r#"<div role="gridcell""#).count(),
            24,
            "{html}"
        );
        assert!(
            html.contains(r#"<div aria-live="polite" data-slot="title">2020 – 2029</div>"#),
            "{html}"
        );
    }

    /// Every control `aria-disabled` and out of the tab order, nothing `inert`, `posted`
    /// hidden inputs all natively disabled.
    fn assert_soft(html: &str, posted: usize) {
        assert!(!html.contains("inert"), "{html}");
        assert!(!html.contains(r#"tabindex="0""#), "{html}");
        let tags = html
            .split('<')
            .skip(1)
            .map(|tag| tag.split('>').next().unwrap_or_default());
        for tag in tags {
            let control = tag.starts_with("button")
                || ["spinbutton", "slider", "tab"]
                    .iter()
                    .any(|role| tag.contains(&format!(r#"role="{role}""#)));
            if control {
                assert!(tag.contains(r#"aria-disabled="true""#), "{tag}");
            }
            if tag.starts_with("input") {
                assert!(tag.contains("disabled"), "{tag}");
            }
        }
        assert_eq!(html.matches(r#"type="hidden""#).count(), posted, "{html}");
    }

    /// Todo 3881: a disabled picker is soft and posts nothing; out-of-range cells stay natively disabled.
    #[test]
    fn a_disabled_picker_is_soft_and_posts_nothing() {
        use super::super::{DateRange, DateRangePicker, YearPicker};
        use chrono::{NaiveDateTime, TimeDelta};

        let day = NaiveDate::from_ymd_opt(2026, 10, 21);
        let time = NaiveTime::from_hms_opt(9, 30, 0);
        let html = dioxus_ssr::render_element(rsx! {
            crate::LiberoProvider {
                DatePicker { value: day, name: "a", disabled: true }
                DateRangePicker { name: "b", disabled: true }
                MonthPicker { value: day, name: "c", disabled: true }
                YearPicker { value: day, name: "d", disabled: true }
                TimePicker { value: time, variant: "digital", name: "e", disabled: true }
                TimePicker { value: time, variant: "analog", name: "f", disabled: true }
                ChronoPicker::<TimeDelta> { name: "g", disabled: true }
                ChronoPicker::<NaiveDateTime> { name: "h", disabled: true }
                ChronoPicker::<DateRange<NaiveDateTime>> { name: "i", disabled: true }
            }
        });
        assert_soft(&html, 9);
    }

    /// Todo 4253: a half range posts nothing, a whole one posts `start/end`.
    #[test]
    fn a_half_range_posts_nothing() {
        use super::super::{DateRange, DateRangePicker};
        use chrono::NaiveDateTime;

        let day = |d| NaiveDate::from_ymd_opt(2026, 9, d).expect("a real day");
        let moment = |d| day(d).and_hms_opt(9, 0, 0).expect("a real time");
        let html = dioxus_ssr::render_element(rsx! {
            crate::LiberoProvider {
                DateRangePicker { name: "a", value: DateRange::new(day(1), None) }
                ChronoPicker::<DateRange<NaiveDateTime>> {
                    name: "b",
                    value: DateRange::new(moment(1), None),
                }
                DateRangePicker { name: "c", value: DateRange::new(day(1), Some(day(5))) }
            }
        });
        let posted: Vec<_> = html
            .split("<input")
            .skip(1)
            .map(|tag| tag.split('>').next().unwrap_or_default())
            .filter(|tag| tag.contains(r#"type="hidden""#))
            .collect();
        assert_eq!(posted.len(), 3, "{html}");
        assert!(!posted[0].contains("2026"), "{}", posted[0]);
        assert!(!posted[1].contains("2026"), "{}", posted[1]);
        assert!(posted[2].contains("2026-09-01/2026-09-05"), "{}", posted[2]);
    }

    /// Todo 3881: a disabled `Fieldset` disables the picker as its own prop does.
    #[test]
    fn a_disabled_fieldset_disables_the_picker() {
        let html = dioxus_ssr::render_element(rsx! {
            crate::LiberoProvider {
                crate::components::Fieldset::<()> { label: "When", disabled: true,
                    DatePicker { value: NaiveDate::from_ymd_opt(2026, 10, 21), name: "a" }
                    TimePicker { variant: "digital", name: "b" }
                }
            }
        });
        assert_soft(&html, 2);
    }

    /// Todo 2310: the analog readout's colons are not read between the buttons.
    #[test]
    fn the_analog_colons_are_hidden() {
        let html = dioxus_ssr::render_element(rsx! {
            crate::LiberoProvider {
                TimePicker {
                    value: NaiveTime::from_hms_opt(9, 30, 0),
                    variant: "analog",
                    with_seconds: true,
                }
            }
        });
        assert!(!html.contains("<span>:</span>"), "{html}");
        assert_eq!(
            html.matches(r#"<span aria-hidden="true">:</span>"#).count(),
            2,
            "{html}"
        );
    }
}
