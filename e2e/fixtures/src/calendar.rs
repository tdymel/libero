//! `ChronoPicker`'s day grid, `DateRangePicker`'s two, and `MonthPicker`'s
//! month and year grids.

use dioxus::prelude::*;
use libero::{
    chrono::NaiveDate,
    components::{Button, ChronoPicker, DateRange, DateRangePicker, Flex, MonthPicker},
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/calendar", || rsx! { CalendarPage {} }),
    ("/calendar/range", || rsx! { RangePage {} }),
    ("/calendar/month", || rsx! { MonthPage {} }),
    ("/calendar/mini", || rsx! { MiniPage {} }),
    ("/calendar/limited", || rsx! { LimitedPage {} }),
];

/// March 2026 with `min` on the 10th, so the days before it are disabled.
#[component]
fn LimitedPage() -> Element {
    rsx! {
        ChronoPicker::<NaiveDate> {
            value: NaiveDate::from_ymd_opt(2026, 3, 18),
            today: NaiveDate::from_ymd_opt(2026, 3, 18),
            min: NaiveDate::from_ymd_opt(2026, 3, 10),
        }
    }
}

/// `CalendarPage`'s day as the one-row mini strip.
#[component]
fn MiniPage() -> Element {
    let mut day = use_signal(|| NaiveDate::from_ymd_opt(2026, 3, 18));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            ChronoPicker {
                value: day(),
                today: NaiveDate::from_ymd_opt(2026, 3, 18),
                calendar: "mini",
                onchange: move |next: Option<NaiveDate>| day.set(next),
            }
            Button { id: "after", "After" }
        }
    }
}

/// March 2026 picked and `today`; its title climbs to the year view.
#[component]
fn MonthPage() -> Element {
    let mut month = use_signal(|| NaiveDate::from_ymd_opt(2026, 3, 1));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            MonthPicker {
                value: month(),
                today: NaiveDate::from_ymd_opt(2026, 3, 18),
                onchange: move |next: Option<NaiveDate>| month.set(next),
            }
            Button { id: "after", "After" }
        }
    }
}

/// Picked and `today` pinned to the same Wednesday, so the grid and its tab
/// stop never depend on the clock. Buttons either side give Tab a way in.
#[component]
fn CalendarPage() -> Element {
    let mut day = use_signal(|| NaiveDate::from_ymd_opt(2026, 3, 18));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            ChronoPicker {
                value: day(),
                today: NaiveDate::from_ymd_opt(2026, 3, 18),
                onchange: move |next: Option<NaiveDate>| day.set(next),
            }
            Button { id: "after", "After" }
        }
    }
}

/// Empty, with `today` pinned to Wednesday 2026-03-18: March and April.
#[component]
fn RangePage() -> Element {
    let mut range = use_signal(|| None::<DateRange<NaiveDate>>);

    rsx! {
        DateRangePicker {
            value: range(),
            today: NaiveDate::from_ymd_opt(2026, 3, 18),
            onchange: move |next: Option<DateRange<NaiveDate>>| range.set(next),
        }
    }
}
