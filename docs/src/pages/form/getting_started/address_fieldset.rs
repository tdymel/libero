use dioxus::prelude::*;

// demo-code: start
use libero::components::{FieldName, Fields, Fieldset, Rule, TextField, not_empty};

#[derive(Clone, PartialEq, Default, Fields)]
pub struct Address {
    pub street: String,
    pub zip: String,
    pub city: String,
}

/// The address block, for any form whose value holds an `Address` somewhere.
#[component]
pub fn AddressFieldset(#[props(into)] label: String, #[props(into)] path: FieldName<Address>) -> Element {
    rsx! {
        Fieldset {
            label,
            path,
            validate: (|a: &Address| a.city.is_empty() || !a.zip.is_empty())
                .error("A city needs its zip code.")
                .on([Address::FIELDS.zip()]),
            // Paths relative to the group work wherever the group sits.
            TextField { label: "Street", autocomplete: "street-address", name: Address::FIELDS.street(), validate: not_empty.error("Enter a street.") }
            TextField { label: "Zip code", autocomplete: "postal-code", name: Address::FIELDS.zip() }
            TextField { label: "City", autocomplete: "address-level2", name: Address::FIELDS.city() }
        }
    }
}
// demo-code: end
