//! `FocusTrap` on its own: its Tab stops, a trap nested in another, and a
//! modal `Dialog` whose trap starts from the dialog itself.

use dioxus::prelude::*;
use libero::{
    components::{Button, Dialog, FocusTrap, Text},
    hooks::{ModalScope, use_modal},
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/focus-trap", || rsx! { StopsPage {} }),
    ("/focus-trap/nested", || rsx! { NestedPage {} }),
    ("/focus-trap/dialog", || rsx! { DialogPage {} }),
];

/// A radio group, a `display: none` button and a `<summary>` inside a trap.
#[component]
fn StopsPage() -> Element {
    rsx! {
        button { id: "before", "Before" }
        FocusTrap {
            button { id: "first", "First" }
            input { id: "r1", r#type: "radio", name: "plan", value: "a" }
            input { id: "r2", r#type: "radio", name: "plan", value: "b", checked: true }
            input { id: "r3", r#type: "radio", name: "plan", value: "c" }
            button { id: "gone", style: "display: none", "Gone" }
            details { summary { id: "summary", "More" } "Details" }
            button { id: "last", "Last" }
        }
        button { id: "after", "After" }
    }
}

/// An inner trap mounted inside an outer one, as a nested dialog would be.
#[component]
fn NestedPage() -> Element {
    let mut inner = use_signal(|| false);
    rsx! {
        FocusTrap {
            button { id: "outer-1", onclick: move |_| inner.set(true), "Open inner" }
            button { id: "outer-2", "Outer two" }
            if inner() {
                FocusTrap {
                    button { id: "inner-1", "Inner one" }
                    button { id: "inner-2", "Inner two" }
                    button { id: "inner-3", onclick: move |_| inner.set(false), "Close inner" }
                }
            }
        }
    }
}

#[component]
fn DialogPage() -> Element {
    let prompt = use_modal(|s: ModalScope<()>| {
        rsx! {
            Dialog { title: "Rename",
                Text { id: "dialog-text", "Pick a new name." }
                Button { id: "keep", variant: "text", onclick: move |_| s.close(), "Keep" }
                Button { id: "rename", variant: "filled", onclick: move |_| s.close(), "Rename" }
            }
        }
    });
    rsx! {
        Button { id: "open-dialog", onclick: move |_| { prompt.open(); }, "Rename file" }
    }
}
