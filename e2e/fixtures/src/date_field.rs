//! `DateField` at month level.

use dioxus::prelude::*;
use libero::{
    chrono::NaiveDate,
    components::{DateField, DateLevel, Flex, Text},
};

use crate::Routes;

pub const ROUTES: Routes = &[("/date-field/month", || rsx! { MonthFieldPage {} })];

/// March 2026 held and `today` pinned, so the grid never depends on the clock.
/// The readout shows the held value in ISO 8601.
#[component]
fn MonthFieldPage() -> Element {
    let mut month = use_signal(|| NaiveDate::from_ymd_opt(2026, 3, 1));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            DateField::<NaiveDate> {
                label: "Billing month",
                level: DateLevel::Month,
                today: NaiveDate::from_ymd_opt(2026, 3, 18),
                value: month(),
                onchange: move |next| month.set(next),
            }
            Text { id: "readout", {month().map(|day| day.to_string()).unwrap_or_default()} }
        }
    }
}
