//! `Dialog` through `use_modal`, the overlay archetype's pilot.

use dioxus::prelude::*;
use libero::{
    components::{Button, Dialog, Flex, Text},
    hooks::{ModalScope, use_modal},
};

use crate::Routes;

pub const ROUTES: Routes = &[("/modal", || rsx! { ModalPage {} })];

/// The overlay archetype: opens, traps focus, Escape closes, focus returns.
#[component]
fn ModalPage() -> Element {
    let prompt = use_modal(|s: ModalScope<()>| {
        rsx! {
            Dialog {
                title: "Unsaved changes",
                size: "sm",
                Text { "notes.md has changes you have not saved." }
                Button { variant: "text", onclick: move |_| s.close(), "Keep editing" }
                Button { variant: "filled", onclick: move |_| s.close(), "Discard" }
            }
        }
    });

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            // A stable hook of the fixture's own, rather than leaving the
            // test to guess at `button`: once the dialog is open there are
            // three buttons on the page and `querySelector` would pick whichever
            // came first.
            Button {
                id: "open-modal",
                onclick: move |_| {
                    prompt.open();
                },
                "Close editor"
            }
        }
    }
}
