//! `FocusTrap` on its own: its Tab stops, a trap nested in another, and a
//! modal `Dialog` whose trap starts from the dialog itself.

use dioxus::prelude::*;
use libero::{
    components::{Button, Dialog, FocusTrap, Switch, Text},
    hooks::{ModalScope, use_focus_return, use_modal},
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/focus-trap", || rsx! { StopsPage {} }),
    ("/focus-trap/radios", || rsx! { RadiosPage {} }),
    ("/focus-trap/nested", || rsx! { NestedPage {} }),
    ("/focus-trap/dialog", || rsx! { DialogPage {} }),
    ("/focus-trap/exit", || rsx! { ExitPage {} }),
];

/// The docs demo's shape: a switch mounts the trap, and Release or Escape inside
/// switches it off and hands focus back (todo 1528).
#[component]
fn ExitPage() -> Element {
    let mut on = use_signal(|| false);
    let back = use_focus_return();
    let mut release = move || {
        on.set(false);
        back.restore();
    };
    rsx! {
        Switch { id: "switch", aria_label: "activate focus trap", checked: on(), onchange: move |next| on.set(next) }
        if on() {
            FocusTrap {
                Remember { onrender: move |_| back.remember_active() }
                div {
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() == Key::Escape {
                            release();
                        }
                    },
                    button { id: "first", "First" }
                    button { id: "release", onclick: move |_| release(), "Release" }
                }
            }
        }
        button { id: "after", "After" }
    }
}

#[component]
fn Remember(onrender: Callback) -> Element {
    use_hook(|| onrender.call(()));
    rsx! {}
}

/// Two radio groups (one checked, one not), a `display: none` button and a
/// `<summary>` inside a trap.
#[component]
fn StopsPage() -> Element {
    rsx! {
        button { id: "before", "Before" }
        FocusTrap {
            button { id: "first", "First" }
            input { id: "r1", r#type: "radio", name: "plan", value: "a" }
            input { id: "r2", r#type: "radio", name: "plan", value: "b", checked: true }
            input { id: "r3", r#type: "radio", name: "plan", value: "c" }
            input { id: "s1", r#type: "radio", name: "size", value: "s" }
            input { id: "s2", r#type: "radio", name: "size", value: "m" }
            button { id: "gone", style: "display: none", "Gone" }
            details { summary { id: "summary", "More" } "Details" }
            button { id: "last", "Last" }
        }
        button { id: "after", "After" }
    }
}

/// Two radio groups in a trap opened by a click, as a dialog's trap is.
#[component]
fn RadiosPage() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        button { id: "open", onclick: move |_| open.set(true), "Open" }
        if open() {
            FocusTrap {
                button { id: "first", "First" }
                input { id: "r1", r#type: "radio", name: "plan", value: "a" }
                input { id: "r2", r#type: "radio", name: "plan", value: "b", checked: true }
                input { id: "s1", r#type: "radio", name: "size", value: "s" }
                input { id: "s2", r#type: "radio", name: "size", value: "m" }
                button { id: "last", "Last" }
            }
        }
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
