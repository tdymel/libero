//! `PinField`.

use dioxus::prelude::*;
use libero::components::{Button, Flex, PinField, Rule, Text, min_length};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/pin-field", || rsx! { PinFieldPage {} }),
    ("/pin-field/error", || rsx! { PinFieldErrorPage {} }),
    ("/pin-field/echo", || rsx! { PinFieldEchoPage {} }),
    ("/pin-field/wide", || rsx! { PinFieldWidePage {} }),
    ("/pin-field/rule", || rsx! { PinFieldRulePage {} }),
];

/// A length rule, and a button to Tab to: the rule shows only once focus leaves the cells (1564).
#[component]
fn PinFieldRulePage() -> Element {
    let mut pin = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            PinField {
                label: "Code",
                length: 4usize,
                value: pin(),
                oninput: move |next: String| pin.set(next),
                validate: min_length(4).error("Enter all four digits."),
            }
            Button { "Next" }
        }
    }
}

/// Eight letter cells with a separator, centred as the docs demo is, in a
/// 320px screen less its margins (todos 1597, 1598, 1600).
#[component]
fn PinFieldWidePage() -> Element {
    let mut pin = use_signal(String::new);

    rsx! {
        Flex { id: "box", direction: "column", align: "center", max_width: "288px",
            PinField {
                label: "Code",
                length: 8usize,
                kind: "alphanumeric",
                separator: rsx! { "-" },
                value: pin(),
                oninput: move |next: String| pin.set(next),
            }
        }
    }
}

/// The typed code echoed in `#echo` and its `oninput` calls counted in
/// `#inputs`, for the shared web/native scenarios.
#[component]
fn PinFieldEchoPage() -> Element {
    let mut pin = use_signal(String::new);
    let mut inputs = use_signal(|| 0);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            PinField {
                label: "Code",
                length: 4usize,
                value: pin(),
                oninput: move |next: String| {
                    pin.set(next);
                    inputs += 1;
                },
            }
            Text { id: "echo", "{pin}" }
            Text { id: "inputs", "{inputs}" }
        }
    }
}

/// Four cells under a label, which names the cells' group by id.
#[component]
fn PinFieldPage() -> Element {
    let mut pin = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            PinField {
                label: "Code",
                length: 4usize,
                value: pin(),
                oninput: move |next: String| pin.set(next),
            }
        }
    }
}

/// A required field showing an error under a helper.
#[component]
fn PinFieldErrorPage() -> Element {
    let mut pin = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            PinField {
                label: "Code",
                helper: "It expires in ten minutes.",
                status: "That code is wrong.",
                required: true,
                length: 4usize,
                value: pin(),
                oninput: move |next: String| pin.set(next),
            }
        }
    }
}
