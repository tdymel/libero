//! `Select`, for the combobox archetype.
//!
//! Not `searchable`, deliberately: a searchable list moves DOM focus into its
//! search box, which is another pattern, and the archetype would be asserting
//! the wrong one.

use dioxus::prelude::*;
use libero::components::{Flex, Select};

use crate::{Routes, common::Fruit};

pub const ROUTES: Routes = &[("/select", || rsx! { SelectPage {} })];

#[component]
fn SelectPage() -> Element {
    let mut value = use_signal(|| Some(Fruit::Banana));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Select {
                label: "Fruit",
                placeholder: "Pick a fruit",
                value: value(),
                onchange: move |next| value.set(next),
            }
        }
    }
}
