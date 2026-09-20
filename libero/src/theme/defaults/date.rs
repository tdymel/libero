use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const CHRONO_DAY_SIZE_SIZE: SizeCss = SizeCss::new("--lsx-chrono-day-size-");
pub const CHRONO_FONT_SIZE_SIZE: SizeCss = SizeCss::new("--lsx-chrono-font-size-");

// The picked level, resolved on the root so every day cell inherits it.
pub const CHRONO_DAY: CssVar = CssVar::new("--lsx-chrono-day");
pub const CHRONO_FONT_SIZE: CssVar = CssVar::new("--lsx-chrono-font-size");

/// The view a calendar shows: days of a month, months of a year, years of a decade.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DateLevel {
    Day,
    Month,
    Year,
}

str_enum! {
    /// How a `TimePicker` shows the time.
    pub enum TimePickerVariant {
        /// A digital clock: one spinbutton column per hours, minutes, seconds, AM/PM.
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

/// Theme defaults for `TimePicker`, set on [`Theme`](crate::theme::Theme).
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
pub struct ChronoSizeLevel {
    /// One day cell, square.
    pub day_size: &'static str,
    pub font_size: &'static str,
}

/// Theme defaults for `ChronoPicker`, set on [`Theme`](crate::theme::Theme).
///
/// Also styles the other date pickers, the clocks' sizes and the fields' dropdowns.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChronoPickerDefaults {
    pub size: Size,
    pub sizes: Sizes<ChronoSizeLevel>,
    /// A month of days, or the mini calendar's row.
    pub calendar: CalendarVariant,
    /// Days in the mini calendar's row.
    pub days: usize,
}

impl ChronoPickerDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        calendar: CalendarVariant::Full,
        days: 7,
        sizes: Sizes::new(
            ChronoSizeLevel {
                day_size: "28px",
                font_size: "12px",
            },
            ChronoSizeLevel {
                day_size: "32px",
                font_size: "13px",
            },
            ChronoSizeLevel {
                day_size: "36px",
                font_size: "14px",
            },
            ChronoSizeLevel {
                day_size: "40px",
                font_size: "16px",
            },
            ChronoSizeLevel {
                day_size: "44px",
                font_size: "18px",
            },
            ChronoSizeLevel {
                day_size: "48px",
                font_size: "20px",
            },
        ),
    };

    pub fn size_sx(size: Size) -> Sx {
        sx().var(CHRONO_DAY, CHRONO_DAY_SIZE_SIZE.value(size))
            .var(CHRONO_FONT_SIZE, CHRONO_FONT_SIZE_SIZE.value(size))
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for ChronoPickerDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = Vec::new();
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(CHRONO_DAY_SIZE_SIZE.declare(size, level.day_size));
            declarations.push(CHRONO_FONT_SIZE_SIZE.declare(size, level.font_size));
        }
        declarations
    }
}

/// Theme defaults for `ChronoField`, set on [`Theme`](crate::theme::Theme).
///
/// Covers every date and time field; frame and calendar come from `FieldDefaults` and `ChronoPickerDefaults`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChronoFieldDefaults {
    pub size: Size,
    pub radius: Size,
    /// Picking a day, or a range's second day, closes the dropdown; times never do.
    pub close_on_change: bool,
}

impl ChronoFieldDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        radius: Size::Sm,
        close_on_change: true,
    };
}
