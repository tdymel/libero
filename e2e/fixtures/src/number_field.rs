//! `NumberField` with its steppers, and one field per state its docs page shows.

use dioxus::prelude::*;
use libero::components::{Button, Flex, Form, NumberField, Text, use_form};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/number-field", || rsx! { NumberFieldPage {} }),
    ("/number-field/echo", || rsx! { NumberFieldEchoPage {} }),
    ("/number-field/reset", || rsx! { NumberFieldResetPage {} }),
];

/// Todo 2327: an empty field in a `Form`, reset by a button; the caller then sets and clears it.
#[component]
fn NumberFieldResetPage() -> Element {
    let value = use_store(String::new);
    let form = use_form();
    let mut quantity = use_signal(|| None::<i32>);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Form { value, form,
                NumberField {
                    id: "quantity",
                    label: "Quantity",
                    value: quantity(),
                    onchange: move |next| quantity.set(next),
                }
            }
            Button { id: "reset", onclick: move |_| form.reset(), "Reset" }
            Button { id: "set", onclick: move |_| quantity.set(Some(5)), "Set 5" }
            Button { id: "clear", onclick: move |_| quantity.set(None), "Clear" }
        }
    }
}

/// Starts on 3, the value echoed in `#echo`, for the shared web/native scenarios.
#[component]
fn NumberFieldEchoPage() -> Element {
    let mut value = use_signal(|| Some(3i32));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            NumberField {
                label: "Quantity",
                steppers: true,
                value: value(),
                onchange: move |next| value.set(next),
            }
            Text { id: "echo", {value().map(|v| v.to_string()).unwrap_or_default()} }
        }
    }
}

/// Starts on 3, so both steppers have somewhere to go. The ranged field starts
/// mid-range with a floor of two digits; `#held` shows what its caller holds.
#[component]
pub fn NumberFieldPage() -> Element {
    let mut quantity = use_signal(|| Some(3i32));
    let mut ranged = use_signal(|| Some(50i32));
    let mut weight = use_signal(|| Some(1.5f64));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            NumberField {
                id: "quantity",
                label: "Quantity",
                steppers: true,
                value: quantity(),
                onchange: move |next| quantity.set(next),
            }
            NumberField {
                id: "ranged",
                label: "Seats",
                helper: "10 to 99.",
                steppers: true,
                min: 10,
                max: 99,
                value: ranged(),
                onchange: move |next| ranged.set(next),
            }
            div { id: "held", "data-value": ranged().map(|v| v.to_string()).unwrap_or_default() }
            NumberField {
                id: "weight",
                label: "Weight",
                step: 0.5,
                steppers: true,
                value: weight(),
                onchange: move |next| weight.set(next),
            }
            NumberField {
                id: "readonly",
                label: "Locked",
                readonly: true,
                steppers: true,
                value: 7,
                onchange: move |_| {},
            }
            NumberField {
                id: "disabled",
                label: "Disabled",
                disabled: true,
                steppers: true,
                value: 7,
                onchange: move |_| {},
            }
        }
    }
}
