//! `DatePicker`'s day grid, and `DateRangePicker`'s two.

use dioxus::prelude::*;
use libero::{
    chrono::NaiveDate,
    components::{Button, DatePicker, DateRange, DateRangePicker, Flex},
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/calendar", || rsx! { CalendarPage {} }),
    ("/calendar/range", || rsx! { RangePage {} }),
];

/// Picked and `today` pinned to the same Wednesday, so the grid and its tab
/// stop never depend on the clock. Buttons either side give Tab a way in.
#[component]
fn CalendarPage() -> Element {
    let mut day = use_signal(|| NaiveDate::from_ymd_opt(2026, 3, 18));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            DatePicker {
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
