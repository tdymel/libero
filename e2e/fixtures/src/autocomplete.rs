//! `Autocomplete`, the combobox archetype's pilot.

use dioxus::prelude::*;
use libero::components::{Autocomplete, Flex};

use crate::Routes;

pub const ROUTES: Routes = &[("/autocomplete", || rsx! { AutocompletePage {} })];

const CITIES: &[&str] = &[
    "Amsterdam",
    "Berlin",
    "Copenhagen",
    "Dublin",
    "Edinburgh",
    "Florence",
];

/// The combobox archetype, fully assembled.
///
/// `Autocomplete` rather than `Combobox` itself: `Combobox` is a wrapper whose
/// caller supplies the trigger and its aria, so a fixture built on it would be
/// testing the fixture's own wiring as much as the library's. `Autocomplete`
/// owns the whole `aria-activedescendant` contract, which is what the pass is
/// about.
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
