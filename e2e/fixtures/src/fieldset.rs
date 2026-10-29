//! A Fieldset whose description and legend toggle while a field inside it has focus.

use dioxus::prelude::*;
use libero::components::{Fieldset, TextField};

use crate::Routes;

pub const ROUTES: Routes = &[("/fieldset", || rsx! { FieldsetPage {} })];

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
