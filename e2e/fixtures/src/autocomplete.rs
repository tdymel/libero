//! `Autocomplete`, the combobox archetype's pilot.

use dioxus::prelude::*;
use libero::components::{Autocomplete, Flex, Text};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/autocomplete", || rsx! { AutocompletePage {} }),
    ("/autocomplete/echo", || rsx! { AutocompleteEchoPage {} }),
];

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
