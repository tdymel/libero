//! A picker field under a finger, then a button: Tab out of its sheet (todo 4606), the refused
//! text box (4607) and its name (4608).

use dioxus::prelude::*;
use libero::{
    chrono::NaiveDate,
    components::{ChronoField, Flex},
};

use crate::Routes;

pub const ROUTES: Routes = &[("/keyboardless", || rsx! { KeyboardlessPage {} })];

#[component]
fn KeyboardlessPage() -> Element {
    let day = |day| NaiveDate::from_ymd_opt(2026, 9, day);
    let mut due = use_signal(move || day(14));

    rsx! {
        Flex { direction: "column", gap: "md", padding: "md",
            div { id: "due",
                ChronoField::<NaiveDate> {
                    label: "Due",
                    today: day(9),
                    value: due(),
                    onchange: move |next| due.set(next),
                }
            }
            button { id: "after", "After" }
        }
    }
}
