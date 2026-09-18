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

str_enum! {
    /// How a `TimePicker` shows the time.
    pub enum TimePickerVariant {
        /// A digital clock: a spinbutton column each for the hours, minutes,
        /// seconds and AM/PM.
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
        /// One row of days, with buttons that page it.
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
