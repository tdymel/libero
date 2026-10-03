//! A date field under the fixture's English provider, and one under a nested
//! `LiberoProvider` with German words and formats. 2026-03-14 held, `today` pinned.
//! A button switches the outer words to German.

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    chrono::NaiveDate,
    components::{ChronoField, Flex},
    hooks::use_localization_handle,
    localization::{Formats, Localization},
};

use crate::Routes;

pub const ROUTES: Routes = &[("/nested-provider", || rsx! { NestedProviderPage {} })];

#[component]
fn NestedProviderPage() -> Element {
    let day = NaiveDate::from_ymd_opt(2026, 3, 14);
    let today = NaiveDate::from_ymd_opt(2026, 3, 18);
    let localization = use_localization_handle();

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            // Todo 2073: the outer switch moves the root's `lang`, the inner provider never does.
            button { id: "outer-german", onclick: move |_| localization.set(&Localization::GERMAN), "Deutsch" }
            div { id: "outer",
                ChronoField::<NaiveDate> { label: "Outer", today, value: day, onchange: |_| {} }
            }
            LiberoProvider { localization: &Localization::GERMAN, formats: &Formats::GERMAN,
                div { id: "inner",
                    ChronoField::<NaiveDate> { label: "Inner", today, value: day, onchange: |_| {} }
                }
            }
        }
    }
}
