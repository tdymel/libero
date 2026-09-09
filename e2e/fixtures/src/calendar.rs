//! `DatePicker`'s day grid.

use dioxus::prelude::*;
use libero::{
    chrono::NaiveDate,
    components::{Button, DatePicker, Flex},
};

use crate::Routes;

pub const ROUTES: Routes = &[("/calendar", || rsx! { CalendarPage {} })];

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
