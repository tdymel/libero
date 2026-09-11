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

/// The view a calendar shows: days of a month, months of a year, years of a
/// decade. `DatePicker`'s `level` picks the lowest one - the one a pick lands
/// on; `DateField`'s the one its text reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DateLevel {
    Day,
    Month,
    Year,
}

/// `DateDefaults::ENGLISH.format`. A named fn, so every copy of the theme
/// holds the same address and compares equal.
fn english_format(level: DateLevel) -> &'static str {
    match level {
        DateLevel::Day => "MMMM D, YYYY",
        DateLevel::Month => "MMMM YYYY",
        DateLevel::Year => "YYYY",
    }
}

/// The words and conventions every date and time component shares: one place
/// to translate. English by default; a translation replaces the names, a
/// region the first weekday and the format.
///
/// The names follow dayjs' locale files (`months`, `monthsShort`, `weekdays`,
/// `weekdaysShort`, `weekdaysMin`), so an existing translation copies over.
///
/// # Fields, in three groups
///
/// - **Names** a language gives: `months`, `months_short`, `weekdays`,
///   `weekdays_short`, `weekdays_min`, `am`, `pm`.
/// - **Conventions** a region picks: `first_weekday`, `format`,
///   `month_format`, `time_format`, `range_separator`.
/// - **Labels** a screen reader or a sighted reader gets: the paging buttons
///   (`previous_month` to `next_days`), `invalid_date`, the switch segments
///   (`date_label` to `end_label`), the switches themselves and the
///   `TimePicker` columns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(
    unpredictable_function_pointer_comparisons,
    reason = "`format` compares by address; a miss on a copied closure only re-renders"
)]
pub struct DateDefaults {
    // Names.
    /// `January` to `December`. `MMMM` in a format, and what a typed month
    /// name is matched against.
    pub months: [&'static str; 12],
    /// `Jan` to `Dec`. `MMM` in a format; typed names match these too.
    pub months_short: [&'static str; 12],
    /// `Sunday` to `Saturday` - always Sunday first, as dayjs has them,
    /// whatever `first_weekday` says, so `Weekday::num_days_from_sunday`
    /// indexes them. `dddd` in a format.
    pub weekdays: [&'static str; 7],
    /// `Sun` to `Sat`. `ddd` in a format.
    pub weekdays_short: [&'static str; 7],
    /// `Su` to `Sa`. `dd` in a format, and a calendar's column headers.
    pub weekdays_min: [&'static str; 7],
    /// `A` in a format, and what a typed `am` matches.
    pub am: &'static str,
    /// `A` in a format, and what a typed `pm` matches.
    pub pm: &'static str,

    // Conventions.
    /// The first column of a calendar.
    pub first_weekday: Weekday,
    /// How a date field shows a day, a month or a year, in dayjs tokens:
    /// `YYYY`, `M`, `MM`, `MMM`, `MMMM`, `D`, `DD`, `dd`, `ddd`, `dddd`. Text
    /// in `[brackets]` is literal. Call it as `(names.format)(level)`.
    ///
    /// One level overridden, the rest English:
    ///
    /// ```
    /// use libero::{components::DateLevel, theme::DateDefaults};
    ///
    /// const ISO_DAYS: DateDefaults = DateDefaults {
    ///     format: |level| match level {
    ///         DateLevel::Day => "YYYY-MM-DD",
    ///         level => (DateDefaults::ENGLISH.format)(level),
    ///     },
    ///     ..DateDefaults::ENGLISH
    /// };
    /// assert_eq!((ISO_DAYS.format)(DateLevel::Day), "YYYY-MM-DD");
    /// assert_eq!((ISO_DAYS.format)(DateLevel::Month), "MMMM YYYY");
    /// ```
    pub format: fn(DateLevel) -> &'static str,
    /// A calendar's month heading.
    pub month_format: &'static str,
    /// How a time is shown, in dayjs tokens: `H`, `HH`, `h`, `hh`, `m`, `mm`,
    /// `s`, `ss`, `A`, `a`. An `h` or an `A` makes pickers 12-hour.
    pub time_format: &'static str,
    /// Between a range's two ends in a field's text.
    pub range_separator: &'static str,

    // Labels.
    /// Names the button that pages a calendar back.
    pub previous_month: &'static str,
    /// Names the button that pages a calendar forward.
    pub next_month: &'static str,
    /// Names the button that pages a calendar's months back a year.
    pub previous_year: &'static str,
    pub next_year: &'static str,
    /// Names the button that pages a calendar's years back ten.
    pub previous_decade: &'static str,
    pub next_decade: &'static str,
    /// Names the button that pages a mini calendar's days back.
    pub previous_days: &'static str,
    pub next_days: &'static str,
    /// The error a date field shows for text that is not a date it accepts.
    pub invalid_date: &'static str,
    /// The segments that switch a dropdown between its calendar and its clock.
    pub date_label: &'static str,
    pub time_label: &'static str,
    /// The segments that switch a range dropdown between its two ends.
    pub start_label: &'static str,
    pub end_label: &'static str,
    /// Names the calendar/clock switch itself.
    pub part_switch_label: &'static str,
    /// Names the start/end switch itself.
    pub side_switch_label: &'static str,
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
            "Sunday",
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
        ],
        weekdays_short: ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
        weekdays_min: ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"],
        am: "AM",
        pm: "PM",
        first_weekday: Weekday::Mon,
        format: english_format,
        month_format: "MMMM YYYY",
        time_format: "HH:mm",
        range_separator: " – ",
        previous_month: "Previous month",
        next_month: "Next month",
        previous_year: "Previous year",
        next_year: "Next year",
        previous_decade: "Previous decade",
        next_decade: "Next decade",
        previous_days: "Previous days",
        next_days: "Next days",
        invalid_date: "Not a valid date",
        date_label: "Date",
        time_label: "Time",
        start_label: "Start",
        end_label: "End",
        part_switch_label: "Date or time",
        side_switch_label: "Range end",
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

str_enum! {
    /// How a calendar lays out its days.
    pub enum CalendarVariant {
        /// A month of days, with a heading that climbs to months and years.
        #[default]
        Full = "full",
        /// One row of days, with buttons that page it - Mantine's
        /// `MiniCalendar`.
        Mini = "mini",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimePickerDefaults {
    pub size: Size,
    pub variant: TimePickerVariant,
    /// Minutes between the offered minutes.
    pub step: u8,
}

impl TimePickerDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        variant: TimePickerVariant::Analog,
        step: 5,
    };
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
    /// A month of days, or the mini calendar's row.
    pub calendar: CalendarVariant,
    /// Days in the mini calendar's row.
    pub days: usize,
}

impl DatePickerDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        calendar: CalendarVariant::Full,
        days: 7,
        sizes: Sizes::new(
            DatePickerSizeLevel {
                day_size: "28px",
                font_size: "12px",
            },
            DatePickerSizeLevel {
                day_size: "32px",
                font_size: "13px",
            },
            DatePickerSizeLevel {
                day_size: "36px",
                font_size: "14px",
            },
            DatePickerSizeLevel {
                day_size: "40px",
                font_size: "16px",
            },
            DatePickerSizeLevel {
                day_size: "44px",
                font_size: "18px",
            },
            DatePickerSizeLevel {
                day_size: "48px",
                font_size: "20px",
            },
        ),
    };

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
    /// Picking a day, or the second end of a range of days, closes the
    /// dropdown. Times, date-times and their ranges never close on a pick.
    pub close_on_change: bool,
}

impl DateFieldDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        radius: Size::Sm,
        close_on_change: true,
    };
}
