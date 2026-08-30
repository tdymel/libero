use chrono::Weekday;

use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const DATE_PICKER_DAY_SIZE_SIZE: SizeCss = SizeCss::new("--lsx-date-picker-day-size-");
pub const DATE_PICKER_FONT_SIZE_SIZE: SizeCss = SizeCss::new("--lsx-date-picker-font-size-");

// The picked level, resolved on the root so every day cell inherits it.
pub const DATE_PICKER_DAY: CssVar = CssVar::new("--lsx-date-picker-day");
pub const DATE_PICKER_FONT_SIZE: CssVar = CssVar::new("--lsx-date-picker-font-size");

/// The words and conventions every date and time component shares: one place
/// to translate. English by default; a translation replaces the names, a
/// region the first weekday and the format.
///
/// The names follow dayjs' locale files (`months`, `monthsShort`, `weekdays`,
/// `weekdaysShort`, `weekdaysMin`), so an existing translation copies over.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateDefaults {
    /// `January` to `December`. `MMMM` in a format, and what a typed month
    /// name is matched against.
    pub months: [&'static str; 12],
    /// `Jan` to `Dec`. `MMM` in a format; typed names match these too.
    pub months_short: [&'static str; 12],
    /// `Monday` to `Sunday` - always Monday first, whatever `first_weekday`
    /// says, so `Weekday::num_days_from_monday` indexes them. `dddd` in a
    /// format.
    pub weekdays: [&'static str; 7],
    /// `Mon` to `Sun`. `ddd` in a format.
    pub weekdays_short: [&'static str; 7],
    /// `Mo` to `Su`. `dd` in a format, and a calendar's column headers.
    pub weekdays_min: [&'static str; 7],
    /// The first column of a calendar.
    pub first_weekday: Weekday,
    /// How a date is shown, in dayjs tokens: `YYYY`, `M`, `MM`, `MMM`,
    /// `MMMM`, `D`, `DD`, `dd`, `ddd`, `dddd`. Text in `[brackets]` is
    /// literal.
    pub format: &'static str,
    /// A calendar's month heading.
    pub month_format: &'static str,
    /// Names the button that pages a calendar back.
    pub previous_month: &'static str,
    /// Names the button that pages a calendar forward.
    pub next_month: &'static str,
    /// The error a date field shows for text that is not a date it accepts.
    pub invalid_date: &'static str,
    /// How a time is shown, in dayjs tokens: `H`, `HH`, `h`, `hh`, `m`, `mm`,
    /// `s`, `ss`, `A`, `a`. An `h` or an `A` makes pickers 12-hour.
    pub time_format: &'static str,
    /// `A` in a format, and what a typed `am` matches.
    pub am: &'static str,
    /// `A` in a format, and what a typed `pm` matches.
    pub pm: &'static str,
    /// Between a range's two ends in a field's text.
    pub range_separator: &'static str,
    /// Names the button that pages a calendar's months back a year.
    pub previous_year: &'static str,
    pub next_year: &'static str,
    /// Names the button that pages a calendar's years back ten.
    pub previous_decade: &'static str,
    pub next_decade: &'static str,
    /// The segments that switch a dropdown between its calendar and its clock.
    pub date_label: &'static str,
    pub time_label: &'static str,
    /// The segments that switch a range dropdown between its two ends.
    pub start_label: &'static str,
    pub end_label: &'static str,
    /// Names a `TimePicker`'s columns.
    pub hours_label: &'static str,
    pub minutes_label: &'static str,
    pub seconds_label: &'static str,
}

impl DateDefaults {
    pub const ENGLISH: Self = Self {
        months: [
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ],
        months_short: [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ],
        weekdays: [
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
            "Sunday",
        ],
        weekdays_short: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
        weekdays_min: ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"],
        first_weekday: Weekday::Mon,
        format: "MMMM D, YYYY",
        month_format: "MMMM YYYY",
        previous_month: "Previous month",
        next_month: "Next month",
        invalid_date: "Not a valid date",
        time_format: "HH:mm",
        am: "AM",
        pm: "PM",
        range_separator: " – ",
        previous_year: "Previous year",
        next_year: "Next year",
        previous_decade: "Previous decade",
        next_decade: "Next decade",
        date_label: "Date",
        time_label: "Time",
        start_label: "Start",
        end_label: "End",
        hours_label: "Hours",
        minutes_label: "Minutes",
        seconds_label: "Seconds",
    };
}

str_enum! {
    /// How a `TimePicker` shows the time.
    pub enum TimePickerVariant {
        /// Scrolling columns of hours, minutes and seconds.
        Digital = "digital",
        /// A clock face: the hour, then the minute.
        #[default]
        Analog = "analog",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimePickerDefaults {
    pub size: Size,
    pub variant: TimePickerVariant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DatePickerSizeLevel {
    /// One day cell, square.
    pub day_size: &'static str,
    pub font_size: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DatePickerDefaults {
    pub size: Size,
    pub sizes: Sizes<DatePickerSizeLevel>,
}

impl DatePickerDefaults {
    pub fn size_sx(size: Size) -> Sx {
        sx().var(DATE_PICKER_DAY, DATE_PICKER_DAY_SIZE_SIZE.value(size))
            .var(
                DATE_PICKER_FONT_SIZE,
                DATE_PICKER_FONT_SIZE_SIZE.value(size),
            )
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for DatePickerDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = Vec::new();
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(DATE_PICKER_DAY_SIZE_SIZE.declare(size, level.day_size));
            declarations.push(DATE_PICKER_FONT_SIZE_SIZE.declare(size, level.font_size));
        }
        declarations
    }
}

/// What `DateField` does not share with every other field. The frame's
/// numbers live on `FieldDefaults` and the dropdown's calendar on
/// `DatePickerDefaults`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateFieldDefaults {
    pub size: Size,
    pub radius: Size,
    /// Picking a day in the dropdown closes it.
    pub close_on_change: bool,
}
