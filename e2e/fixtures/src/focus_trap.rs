//! `FocusTrap` on its own: its Tab stops, a trap nested in another, and a
//! modal `Dialog` whose trap starts from the dialog itself.

use dioxus::prelude::*;
use libero::{
    components::{Button, Dialog, FocusTrap, FocusTrapInitialFocus, Switch, Text},
    hooks::{ModalScope, use_modal},
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/focus-trap", || rsx! { StopsPage {} }),
    ("/focus-trap/radios", || rsx! { RadiosPage {} }),
    ("/focus-trap/nested", || rsx! { NestedPage {} }),
    ("/focus-trap/dialog", || rsx! { DialogPage {} }),
    ("/focus-trap/exit", || rsx! { ExitPage {} }),
    ("/focus-trap/autofocus", || rsx! { AutofocusPage {} }),
    ("/focus-trap/leaving", || rsx! { LeavingPage {} }),
    ("/focus-trap/placeholder", || rsx! { PlaceholderPage {} }),
];

/// A `FocusTrapInitialFocus` placeholder, then a stop that removes itself (2812).
#[component]
fn PlaceholderPage() -> Element {
    let mut shown = use_signal(|| true);
    rsx! {
        FocusTrap {
            FocusTrapInitialFocus {}
            button { id: "first", "First" }
            if shown() {
                button { id: "remove", onclick: move |_| shown.set(false), "Remove" }
            }
            button { id: "last", "Last" }
        }
    }
}

/// Ways focus leaves without a Tab: the last stop removes itself, a click on text outside (2297).
#[component]
fn LeavingPage() -> Element {
    let mut shown = use_signal(|| true);
    rsx! {
        button { id: "before", "Before" }
        p { id: "outside", "Text outside the trap." }
        FocusTrap {
            button { id: "first", "First" }
            button { id: "middle", "Middle" }
            if shown() {
                button { id: "remove", onclick: move |_| shown.set(false), "Remove" }
            }
        }
        button { id: "after", "After" }
    }
}

/// An autofocus target that cannot take focus (`display: none`), before the first stop (2299).
#[component]
fn AutofocusPage() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        button { id: "open", onclick: move |_| open.set(true), "Open" }
        if open() {
            FocusTrap {
                button { id: "hidden", style: "display: none", "data-autofocus": true, "Hidden" }
                button { id: "first", "First" }
                button { id: "last", "Last" }
            }
        }
    }
}

/// The docs demo's shape: a switch mounts the trap, and Release or Escape inside
/// switches it off; `restore_focus` hands focus back (todos 1528, 1534).
#[component]
fn ExitPage() -> Element {
    let mut on = use_signal(|| false);
    rsx! {
        Switch { id: "switch", aria_label: "activate focus trap", checked: on(), onchange: move |next| on.set(next) }
        if on() {
            FocusTrap { restore_focus: true,
                div {
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() == Key::Escape {
                            on.set(false);
                        }
                    },
                    button { id: "first", "First" }
                    button { id: "release", onclick: move |_| on.set(false), "Release" }
                }
            }
        }
        button { id: "after", "After" }
    }
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
