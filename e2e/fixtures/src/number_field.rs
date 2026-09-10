//! `NumberField` with its steppers.

use dioxus::prelude::*;
use libero::components::{Flex, NumberField};

use crate::Routes;

pub const ROUTES: Routes = &[("/number-field", || rsx! { NumberFieldPage {} })];

/// Starts on 3, so both steppers have somewhere to go.
#[component]
pub fn NumberFieldPage() -> Element {
    let mut quantity = use_signal(|| 3i32);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            NumberField {
                label: "Quantity",
                steppers: true,
                value: quantity(),
                onchange: move |next| quantity.set(next),
            }
        }
    }
}
