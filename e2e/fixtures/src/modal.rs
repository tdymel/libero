//! `Dialog` through `use_modal`, the overlay archetype's pilot.

use dioxus::prelude::*;
use libero::{
    components::{Button, Dialog, Flex, Menu, MenuItem, Text, use_menu},
    hooks::{ModalScope, use_modal},
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/modal", || rsx! { ModalPage {} }),
    ("/modal/static", || rsx! { StaticModalPage {} }),
    ("/modal/menu", || rsx! { MenuModalPage {} }),
    ("/modal/nested", || rsx! { NestedModalPage {} }),
    ("/modal/tall", || rsx! { TallModalPage {} }),
];

/// A dialog taller than the viewport, as at 200% zoom or on a 320px phone.
#[component]
fn TallModalPage() -> Element {
    let tall = use_modal(|s: ModalScope<()>| {
        rsx! {
            Dialog { title: "Terms",
                for line in 0..40 {
                    Text { "Clause {line}: a long line of terms the reader has to scroll to." }
                }
                Button { id: "accept", onclick: move |_| s.close(), "Accept" }
            }
        }
    });

    rsx! {
        Button {
            id: "open-modal",
            onclick: move |_| {
                tall.open();
            },
            "Open terms"
        }
    }
}

/// A modal opened from inside a modal, on a page tall enough to scroll.
#[component]
fn NestedModalPage() -> Element {
    let inner = use_modal(|s: ModalScope<()>| {
        rsx! {
            Dialog { title: "Inner",
                Button { id: "inner-close", onclick: move |_| s.close(), "Close inner" }
                Button { id: "inner-other", variant: "text", "Other" }
            }
        }
    });
    let outer = use_modal(move |s: ModalScope<()>| {
        rsx! {
            Dialog { title: "Outer",
                Button {
                    id: "open-inner",
                    onclick: move |_| {
                        inner.open();
                    },
                    "Open inner"
                }
                Button { id: "outer-close", variant: "text", onclick: move |_| s.close(), "Close outer" }
            }
        }
    });

    rsx! {
        Button {
            id: "open-modal",
            onclick: move |_| {
                outer.open();
            },
            "Open outer"
        }
        div { style: "height: 3000px;", "Tall page" }
    }
}

/// A menu inside a dialog: Android's Back closes the menu, then the dialog (1275).
#[component]
fn MenuModalPage() -> Element {
    let dialog = use_modal(|_: ModalScope<()>| {
        rsx! {
            Dialog { title: "Notes",
                DialogMenu {}
            }
        }
    });

    rsx! {
        Button {
            id: "open-modal",
            onclick: move |_| {
                dialog.open();
            },
            "Open notes"
        }
    }
}

#[component]
fn DialogMenu() -> Element {
    let menu = use_menu();
    rsx! {
        Menu {
            state: menu,
            items: vec![
                MenuItem::new("Rename").onselect(|_| {}).into(),
                MenuItem::new("Delete").onselect(|_| {}).into(),
            ],
            Button { attributes: menu.a11y_attributes(), "Actions" }
        }
    }
}

/// A dialog with nothing to focus: focus has to land on the dialog itself.
#[component]
fn StaticModalPage() -> Element {
    let notice = use_modal(|_: ModalScope<()>| {
        rsx! {
            Dialog { title: "Saved", close_button: false,
                Text { "Your changes are saved." }
            }
        }
    });

    rsx! {
        Button {
            id: "open-modal",
            onclick: move |_| {
                notice.open();
            },
            "Save"
        }
    }
}

/// The overlay archetype: opens, traps focus, Escape closes, focus returns.
#[component]
fn ModalPage() -> Element {
    let prompt = use_modal(|s: ModalScope<()>| {
        rsx! {
            Dialog {
                title: "Unsaved changes",
                size: "sm",
                Text { id: "modal-text", "notes.md has changes you have not saved." }
                Button { id: "keep", variant: "text", onclick: move |_| s.close(), "Keep editing" }
                Button { id: "discard", variant: "filled", onclick: move |_| s.close(), "Discard" }
            }
        }
    });

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            // Own id: with the dialog open, `querySelector("button")` picks whichever comes first.
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
