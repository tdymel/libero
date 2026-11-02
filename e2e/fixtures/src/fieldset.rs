//! A Fieldset whose description and legend toggle while a field inside it has focus.

use dioxus::prelude::*;
use libero::components::{Fields, Fieldset, Rule, TextField};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/fieldset", || rsx! { FieldsetPage {} }),
    ("/fieldset/rules", || rsx! { FieldsetRulesPage {} }),
];

#[derive(Clone, PartialEq, Default, Fields)]
struct Address {
    street: String,
    city: String,
    zip: String,
}

/// Todo 5246: a standalone group whose rule names Zip, in a raw `<form>` that resets.
#[component]
fn FieldsetRulesPage() -> Element {
    let address = use_store(Address::default);
    rsx! {
        form { id: "form",
            Fieldset {
                label: "Address",
                value: address,
                validate: (|a: &Address| a.city.is_empty() || !a.zip.is_empty())
                    .error("A city needs its zip code.")
                    .on([Address::FIELDS.zip()]),
                TextField { id: "street", label: "Street", name: Address::FIELDS.street() }
                TextField { id: "city", label: "City", name: Address::FIELDS.city() }
                TextField { id: "zip", label: "Zip", name: Address::FIELDS.zip() }
            }
        }
        button { id: "outside", "Outside" }
    }
}

#[component]
fn FieldsetPage() -> Element {
    let mut described = use_signal(|| false);
    let mut labelled = use_signal(|| true);
    rsx! {
        button { id: "toggle-description", onclick: move |_| described.toggle(), "Description" }
        button { id: "toggle-legend", onclick: move |_| labelled.toggle(), "Legend" }
        Fieldset::<()> {
            id: "fieldset",
            label: if labelled() { "Address" } else { "" },
            description: if described() { "Where we ship to." } else { "" },
            TextField { label: "Street" }
            TextField { label: "City" }
        }
    }
}
