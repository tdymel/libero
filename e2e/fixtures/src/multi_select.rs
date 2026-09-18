//! `MultiSelect`, for the combobox archetype. Not `searchable`, for the same
//! reason as `Select`.

use dioxus::prelude::*;
use libero::components::{Flex, MultiSelect};

use crate::{Routes, common::Fruit};

pub const ROUTES: Routes = &[
    ("/multi-select", || rsx! { MultiSelectPage {} }),
    ("/multi-select/search", || rsx! { MultiSelectSearchPage {} }),
    (
        "/multi-select/refused",
        || rsx! { MultiSelectRefusedPage {} },
    ),
];

/// One chip already held, so the trigger draws the chip and its remove control
/// at rest - the part of this component that is neither a field nor a list.
#[component]
fn MultiSelectPage() -> Element {
    let mut value = use_signal(|| vec![Fruit::Cherry]);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            MultiSelect {
                label: "Fruit",
                placeholder: "Pick fruit",
                value: value(),
                onchange: move |next| value.set(next),
            }
        }
    }
}

/// `searchable`: the search box's Home and End edit the query (todo 547).
#[component]
fn MultiSelectSearchPage() -> Element {
    let mut value = use_signal(Vec::<Fruit>::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            MultiSelect {
                label: "Fruit",
                searchable: true,
                search_placeholder: "Search fruit",
                value: value(),
                onchange: move |next| value.set(next),
            }
        }
    }
}

/// A controlled caller that refuses any selection holding Apple (todo 876).
#[component]
fn MultiSelectRefusedPage() -> Element {
    let mut value = use_signal(|| vec![Fruit::Cherry]);

    rsx! {
        form { max_width: "320px",
            MultiSelect {
                label: "Fruit",
                name: "fruit",
                value: value(),
                onchange: move |next: Vec<Fruit>| {
                    if !next.contains(&Fruit::Apple) {
                        value.set(next);
                    }
                },
            }
        }
    }
}
