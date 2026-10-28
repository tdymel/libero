//! A picker field under a finger, then a button: Tab out of its sheet (todo 4606), the refused
//! text box (4607) and its name (4608). The same field last in a modal (4777).

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    chrono::NaiveDate,
    components::{Button, ChronoField, Dialog, Flex},
    hooks::{ModalScope, use_modal},
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/keyboardless", || rsx! { KeyboardlessPage {} }),
    ("/keyboardless/modal", || rsx! { ModalPage {} }),
];

#[component]
fn KeyboardlessPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", padding: "md",
            DueField {}
            button { id: "after", "After" }
        }
    }
}

#[component]
fn DueField() -> Element {
    let day = |day| NaiveDate::from_ymd_opt(2026, 9, day);
    let mut due = use_signal(move || day(14));
    rsx! {
        div { id: "due",
            ChronoField::<NaiveDate> {
                label: "Due",
                today: day(9),
                value: due(),
                onchange: move |next| due.set(next),
            }
        }
    }
}

/// A nested provider's modal, so the page has a stop after the modal in the DOM.
#[component]
fn ModalPage() -> Element {
    rsx! {
        LiberoProvider { ModalOpener {} }
        button { id: "behind", "Behind" }
    }
}

#[component]
fn ModalOpener() -> Element {
    let prompt = use_modal(|_: ModalScope<()>| {
        rsx! {
            Dialog { id: "modal", title: "Plan", DueField {} }
        }
    });
    rsx! {
        Button { id: "open", onclick: move |_| { prompt.open(); }, "Open" }
    }
}
