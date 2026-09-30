//! `Cascader`.

use dioxus::prelude::*;
use libero::components::{Cascader, CascaderLayout, CascaderOption, FieldStatus, Flex, Text};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/cascader", || rsx! { CascaderPage {} }),
    ("/cascader/search", || rsx! { CascaderSearchPage {} }),
    ("/cascader/paths", || rsx! { CascaderPathsPage {} }),
    ("/cascader/any-level", || rsx! { CascaderAnyLevelPage {} }),
    ("/cascader/low", || rsx! { CascaderLowPage {} }),
];

/// The trigger near the window's foot of a page that scrolls on: a phone's sheet
/// would cover it (todo 1546).
#[component]
fn CascaderLowPage() -> Element {
    let mut place = use_signal(|| None::<String>);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            div { height: "85vh" }
            Cascader {
                label: "Place",
                data: places(),
                value: place(),
                onchange: move |next: Option<String>| place.set(next),
            }
            div { height: "100vh" }
        }
    }
}

/// `any_level`: a branch is a pick of its own. `#picked` shows the committed value.
#[component]
fn CascaderAnyLevelPage() -> Element {
    let mut place = use_signal(|| None::<String>);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Cascader {
                label: "Place",
                any_level: true,
                data: places(),
                value: place(),
                onchange: move |next: Option<String>| place.set(next),
            }
            Text { id: "picked", {place().unwrap_or_default()} }
        }
    }
}

/// The flat layout: one row per leaf path.
#[component]
fn CascaderPathsPage() -> Element {
    let mut place = use_signal(|| None::<String>);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Cascader {
                label: "Place",
                layout: CascaderLayout::Paths,
                data: places(),
                value: place(),
                onchange: move |next: Option<String>| place.set(next),
            }
        }
    }
}

/// Three levels, with the middle root disabled so the keys and a click both
/// have one to pass over. `#picked` shows the committed value.
#[component]
fn CascaderPage() -> Element {
    let mut place = use_signal(|| None::<String>);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Cascader {
                label: "Place",
                data: places(),
                value: place(),
                onchange: move |next: Option<String>| place.set(next),
            }
            Text { id: "picked", {place().unwrap_or_default()} }
        }
    }
}

/// `searchable` with every caption, an error and `required`: the open search
/// box takes over the whole field wiring from the trigger.
#[component]
fn CascaderSearchPage() -> Element {
    let mut place = use_signal(|| None::<String>);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Cascader {
                label: "Place",
                description: "Where the parcel goes.",
                helper: "Cities only.",
                status: FieldStatus::Error("Pick a place.".into()),
                required: true,
                searchable: true,
                search_placeholder: "Search places",
                data: places(),
                value: place(),
                onchange: move |next: Option<String>| place.set(next),
            }
        }
    }
}

fn places() -> Vec<CascaderOption<String>> {
    vec![
        CascaderOption::new("europe", "Europe").children(vec![
            CascaderOption::new("france", "France").children(vec![
                CascaderOption::new("paris", "Paris"),
                CascaderOption::new("lyon", "Lyon"),
            ]),
            CascaderOption::new("germany", "Germany")
                .children(vec![CascaderOption::new("berlin", "Berlin")]),
        ]),
        CascaderOption::new("asia", "Asia")
            .disabled(true)
            .children(vec![
                CascaderOption::new("japan", "Japan")
                    .children(vec![CascaderOption::new("tokyo", "Tokyo")]),
            ]),
        CascaderOption::new("oceania", "Oceania").children(vec![
            CascaderOption::new("australia", "Australia")
                .children(vec![CascaderOption::new("sydney", "Sydney")]),
        ]),
    ]
}
