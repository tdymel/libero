//! `PinField`.

use dioxus::prelude::*;
use libero::components::{Flex, PinField, Text};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/pin-field", || rsx! { PinFieldPage {} }),
    ("/pin-field/error", || rsx! { PinFieldErrorPage {} }),
    ("/pin-field/echo", || rsx! { PinFieldEchoPage {} }),
];

/// The typed code echoed in `#echo`, for the shared web/native scenarios.
#[component]
fn PinFieldEchoPage() -> Element {
    let mut pin = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            PinField {
                label: "Code",
                length: 4usize,
                value: pin(),
                oninput: move |next: String| pin.set(next),
            }
            Text { id: "echo", "{pin}" }
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
