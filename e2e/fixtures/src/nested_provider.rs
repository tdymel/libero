//! A date field under the fixture's English provider, and one under a nested
//! `LiberoProvider` with German words and formats. 2026-03-14 held, `today` pinned.
//! A button switches the outer words to German; two force and release reduced motion inside.

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    chrono::NaiveDate,
    components::{Box, ChronoField, Flex},
    hooks::{use_accessibility, use_localization_handle},
    localization::{Formats, Localization},
    sx::sx,
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
                InnerMotion {}
            }
        }
    }
}

/// Forces reduced motion through the inner provider (todo 2232); `#inner-motion` pads 7px when reduced.
#[component]
fn InnerMotion() -> Element {
    let calm = use_accessibility();
    let follow = calm.clone();
    rsx! {
        button { id: "inner-calm", onclick: move |_| calm.set_reduced_motion(Some(true)), "Calm" }
        button { id: "inner-follow", onclick: move |_| follow.set_reduced_motion(None), "Follow" }
        Box {
            id: "inner-motion",
            sx: sx().padding("1px").media("(prefers-reduced-motion: reduce)", sx().padding("7px")),
            "Motion"
        }
    }
}
