//! `MultiSelect`, for the combobox archetype. Not `searchable`, for the same
//! reason as `Select`.

use dioxus::prelude::*;
use libero::components::{Flex, MultiSelect, OptionList, Text};

use crate::{Routes, common::Fruit};

pub const ROUTES: Routes = &[
    (
        "/multi-select/refused-first",
        || rsx! { MultiSelectRefusedFirstPage {} },
    ),
    ("/multi-select", || rsx! { MultiSelectPage {} }),
    ("/multi-select/search", || rsx! { MultiSelectSearchPage {} }),
    (
        "/multi-select/refused",
        || rsx! { MultiSelectRefusedPage {} },
    ),
    ("/multi-select/echo", || rsx! { MultiSelectEchoPage {} }),
    ("/multi-select/chips", || rsx! { MultiSelectChipsPage {} }),
    (
        "/multi-select/open-on-focus",
        || rsx! { MultiSelectOpenOnFocusPage {} },
    ),
];

/// Three chips held and `open_on_focus`: focus coming back from a chip or a sheet stays closed (todo 3982).
#[component]
fn MultiSelectOpenOnFocusPage() -> Element {
    let mut value = use_signal(|| vec![Fruit::Apple, Fruit::Banana, Fruit::Cherry]);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            button { id: "before", "Before" }
            MultiSelect {
                label: "Fruit",
                open_on_focus: true,
                value: value(),
                onchange: move |next| value.set(next),
            }
            Text { id: "echo", "{value:?}" }
        }
    }
}

/// Three chips held, for the keyboard's walk over them (todo 2976).
#[component]
fn MultiSelectChipsPage() -> Element {
    let mut value = use_signal(|| vec![Fruit::Apple, Fruit::Banana, Fruit::Cherry]);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            MultiSelect {
                label: "Fruit",
                value: value(),
                onchange: move |next| value.set(next),
            }
            Text { id: "echo", "{value:?}" }
        }
    }
}

/// Cherry held, the value echoed in `#echo`, for the shared web/native scenarios.
#[component]
fn MultiSelectEchoPage() -> Element {
    let mut value = use_signal(|| vec![Fruit::Cherry]);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            MultiSelect {
                label: "Fruit",
                value: value(),
                onchange: move |next| value.set(next),
            }
            Text { id: "echo", "{value:?}" }
        }
    }
}

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

/// Apple, the first row, is refused: the list opens on Banana (1271).
#[component]
fn MultiSelectRefusedFirstPage() -> Element {
    let mut value = use_signal(Vec::<Fruit>::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            MultiSelect {
                label: "Fruit",
                options: OptionList::from_options().disabling(|f| matches!(f, Fruit::Apple)),
                value: value(),
                onchange: move |next| value.set(next),
            }
        }
    }
}
