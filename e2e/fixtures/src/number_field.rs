//! `NumberField` with its steppers, and one field per state its docs page shows.

use dioxus::prelude::*;
use libero::components::{
    Button, Decimals, Flex, Form, NumberField, Text, TouchInput, use_form,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/number-field", || rsx! { NumberFieldPage {} }),
    ("/number-field/echo", || rsx! { NumberFieldEchoPage {} }),
    ("/number-field/reset", || rsx! { NumberFieldResetPage {} }),
    ("/number-field/rtl", || rsx! { NumberFieldRtlPage {} }),
    ("/number-field/grouped", || rsx! { NumberFieldGroupedPage {} }),
    ("/number-field/decimals", || rsx! { NumberFieldDecimalsPage {} }),
    ("/number-field/spin", || rsx! { NumberFieldSpinPage {} }),
];

/// Todo 4133: a tap spins in steps of 5. `#spun` shows the value and the `onchange` count.
/// `#lock` disables the field (4268); `#pause` is named by `aria-label` alone (4272).
#[component]
fn NumberFieldSpinPage() -> Element {
    let mut minutes = use_signal(|| Some(30u32));
    let mut changes = use_signal(|| 0u32);
    let mut locked = use_signal(|| false);
    let mut pause = use_signal(|| Some(5u32));
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px", padding: "md",
            NumberField {
                id: "minutes",
                label: "Break",
                trailing: rsx! { "min" },
                describe_trailing: true,
                step: 5,
                touch_input: TouchInput::Spin,
                disabled: locked(),
                value: minutes(),
                onchange: move |next| {
                    minutes.set(next);
                    changes += 1;
                },
            }
            Text { id: "spun", {format!("{:?} {}", minutes(), changes())} }
            Button { id: "lock", onclick: move |_| locked.toggle(), "Lock" }
            NumberField {
                id: "pause",
                "aria-label": "Pause",
                touch_input: TouchInput::Spin,
                value: pause(),
                onchange: move |next| pause.set(next),
            }
        }
    }
}

/// Two places, a `$` before the field and described by it; `#readout` shows the value as `{:?}`.
#[component]
fn NumberFieldDecimalsPage() -> Element {
    let mut price = use_signal(|| Some(0.2f64));
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            NumberField {
                id: "price",
                label: "Price",
                decimals: Decimals::Exactly(2),
                leading: rsx! { "$" },
                describe_leading: true,
                steppers: true,
                step: 0.1,
                value: price(),
                onchange: move |next| price.set(next),
            }
            Text { id: "readout", {format!("{:?}", price())} }
        }
    }
}

/// Todo 3390: `group_digits` groups the number until the field takes the focus.
/// `#fee` pads two places (4717).
#[component]
fn NumberFieldGroupedPage() -> Element {
    let mut amount = use_signal(|| Some(1_234_567.5f64));
    let mut fee = use_signal(|| Some(1_250.5f64));
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            NumberField {
                id: "amount",
                label: "Amount",
                group_digits: true,
                value: amount(),
                onchange: move |next| amount.set(next),
            }
            NumberField {
                id: "fee",
                label: "Fee",
                group_digits: true,
                decimals: Decimals::Exactly(2),
                value: fee(),
                onchange: move |next| fee.set(next),
            }
            Button { id: "after", "After" }
        }
    }
}

/// Todo 2608: a negative number under `dir=rtl` still reads `-5`.
#[component]
fn NumberFieldRtlPage() -> Element {
    let mut delta = use_signal(|| Some(-5i32));
    rsx! {
        div { dir: "rtl",
            NumberField { id: "delta", label: "Delta", value: delta(), onchange: move |next| delta.set(next) }
        }
    }
}

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
