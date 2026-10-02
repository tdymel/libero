//! `Autocomplete`, the combobox archetype's pilot.

use dioxus::prelude::*;
use libero::components::{Autocomplete, Flex, OptionList, Text};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/autocomplete", || rsx! { AutocompletePage {} }),
    ("/autocomplete/echo", || rsx! { AutocompleteEchoPage {} }),
    ("/autocomplete/fetch", || rsx! { AutocompleteFetchPage {} }),
    ("/autocomplete/odd", || rsx! { AutocompleteOddPage {} }),
];

/// Spaces, quotes, non-ASCII and two equal labels, for the option ids (todo 2046).
#[component]
fn AutocompleteOddPage() -> Element {
    let mut value = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Autocomplete {
                label: "Place",
                options: ODD.iter().map(|c| c.to_string()).collect::<Vec<_>>(),
                value: value(),
                oninput: move |next| value.set(next),
            }
        }
    }
}

const ODD: &[&str] = &["New York", "O'Brien \"Jr\"", "Zürich", "Paris", "Paris", "Pärnu"];

/// A server-side search: the answer for one or two letters is still on its way (todo 1585).
#[component]
fn AutocompleteFetchPage() -> Element {
    let mut value = use_signal(String::new);
    let text = value();
    let answer = (text.chars().count() >= 3).then(|| {
        let query = text.to_lowercase();
        OptionList::from(
            CITIES
                .iter()
                .filter(|city| city.to_lowercase().contains(&query))
                .map(|city| city.to_string())
                .collect::<Vec<_>>(),
        )
    });

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Autocomplete {
                label: "City",
                prefiltered: true,
                options: answer,
                value: text,
                oninput: move |next| value.set(next),
            }
        }
    }
}

/// The value echoed in `#echo`, for the shared web/native scenarios.
#[component]
fn AutocompleteEchoPage() -> Element {
    let mut value = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Autocomplete {
                label: "City",
                options: CITIES.iter().map(|c| c.to_string()).collect::<Vec<_>>(),
                value: value(),
                oninput: move |next| value.set(next),
            }
            Text { id: "echo", "{value}" }
        }
    }
}

const CITIES: &[&str] = &[
    "Amsterdam",
    "Berlin",
    "Copenhagen",
    "Dublin",
    "Edinburgh",
    "Florence",
];

/// The combobox archetype: `Autocomplete` owns the whole `aria-activedescendant` contract,
/// where `Combobox` leaves the trigger's aria to the caller.
#[component]
fn AutocompletePage() -> Element {
    let mut value = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Autocomplete {
                label: "City",
                options: CITIES.iter().map(|c| c.to_string()).collect::<Vec<_>>(),
                value: value(),
                oninput: move |next| value.set(next),
            }
        }
    }
}
