//! `Cascader`.

use dioxus::prelude::*;
use libero::components::{
    ActionIcon, Cascader, CascaderDropdown, CascaderLayout, CascaderNodeArgs, CascaderOption,
    FieldStatus, Flex, Text, use_cascader,
};
use pictogram_icons_lucide as lucide;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/cascader", || rsx! { CascaderPage {} }),
    ("/cascader/search", || rsx! { CascaderSearchPage {} }),
    ("/cascader/paths", || rsx! { CascaderPathsPage {} }),
    ("/cascader/any-level", || rsx! { CascaderAnyLevelPage {} }),
    ("/cascader/any-level-deselect", || {
        rsx! { CascaderAnyLevelDeselectPage {} }
    }),
    ("/cascader/low", || rsx! { CascaderLowPage {} }),
    ("/cascader/outside", || rsx! { CascaderOutsidePage {} }),
    ("/cascader/trigger", || rsx! { CascaderTriggerPage {} }),
];

/// minmax's ore picker (3895): a round icon button opens the headless cascade; each row trails
/// its `have/need` stock in `[data-stock]`. `#picked` shows the committed value.
#[component]
fn CascaderTriggerPage() -> Element {
    let mut ore = use_signal(|| None::<String>);
    let cascader = use_cascader(ores(), ore(), move |next| ore.set(next));
    let name = cascader
        .labels()
        .last()
        .cloned()
        .unwrap_or_else(|| "none".to_string());

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            input { id: "before", font_size: "16px", "aria-label": "Before" }
            CascaderDropdown {
                state: cascader.clone(),
                node: move |row: CascaderNodeArgs<String>| {
                    let (have, need) = stock(&row.value);
                    rsx! {
                        span { "{row.label}" }
                        if let Some(need) = need {
                            span { "data-stock": "", style: "margin-inline-start:auto;padding-inline-start:12px", "{have}/{need}" }
                        }
                    }
                },
                ActionIcon {
                    attributes: cascader.a11y_attributes(),
                    radius: "xl",
                    aria_label: "Ore: {name}",
                    icon: lucide::pickaxe::outlined,
                }
            }
            Text { id: "picked", {ore().unwrap_or_default()} }
        }
    }
}

/// Tier 1 then its ores; Tier 2 has one.
fn ores() -> Vec<CascaderOption<String>> {
    vec![
        CascaderOption::new("tier-1", "Tier 1").children(vec![
            CascaderOption::new("iron", "Iron ore"),
            CascaderOption::new("copper", "Copper ore"),
        ]),
        CascaderOption::new("tier-2", "Tier 2")
            .children(vec![CascaderOption::new("gold", "Gold ore")]),
    ]
}

/// Have and need per ore; a tier has none.
fn stock(value: &str) -> (u32, Option<u32>) {
    match value {
        "iron" => (3, Some(10)),
        "copper" => (12, Some(8)),
        "gold" => (0, Some(2)),
        _ => (0, None),
    }
}

/// Searchable, with a field above it to click while the list is open below (1497)
/// and one after it for Tab (2289).
#[component]
fn CascaderOutsidePage() -> Element {
    let mut place = use_signal(|| None::<String>);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            input { id: "outside", font_size: "16px", "aria-label": "Note" }
            Cascader {
                label: "Place",
                searchable: true,
                data: places(),
                value: place(),
                onchange: move |next: Option<String>| place.set(next),
            }
            input { id: "after", font_size: "16px", "aria-label": "After" }
        }
    }
}

/// The trigger near the window's foot of a page that scrolls on: a phone's sheet
/// would cover it (todo 1546), and the helper and error under it (todo 1658).
#[component]
fn CascaderLowPage() -> Element {
    let mut place = use_signal(|| None::<String>);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            div { height: "85vh" }
            div { id: "low-field",
                Cascader {
                    label: "Place",
                    helper: "Cities only.",
                    status: FieldStatus::Error("Pick a place.".into()),
                    data: places(),
                    value: place(),
                    onchange: move |next: Option<String>| place.set(next),
                }
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

/// `any_level` with `allow_deselect` (3814): `#picked` shows the committed value, `#changes`
/// how many times it changed.
#[component]
fn CascaderAnyLevelDeselectPage() -> Element {
    let mut place = use_signal(|| None::<String>);
    let mut changes = use_signal(|| 0);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Cascader {
                label: "Place",
                any_level: true,
                allow_deselect: true,
                data: places(),
                value: place(),
                onchange: move |next: Option<String>| {
                    place.set(next);
                    changes += 1;
                },
            }
            Text { id: "picked", {place().unwrap_or_default()} }
            Text { id: "changes", "{changes}" }
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
