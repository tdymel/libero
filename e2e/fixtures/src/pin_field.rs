//! `PinField`.

use dioxus::prelude::*;
use libero::components::{Flex, PinField};

use crate::Routes;

pub const ROUTES: Routes = &[("/pin-field", || rsx! { PinFieldPage {} })];

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
