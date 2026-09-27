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
    ("/modal/self-removing", || rsx! { SelfRemovingModalPage {} }),
    ("/modal/scrollbar", || rsx! { ScrollbarModalPage {} }),
    ("/modal/owner", || rsx! { OwnerModalPage {} }),
];

/// A dialog whose only control removes itself: focus falls to `<body>` (todo 1304).
#[component]
fn SelfRemovingModalPage() -> Element {
    let notice = use_modal(|_: ModalScope<()>| rsx! { SelfRemoving {} });

    rsx! {
        Button {
            id: "open-modal",
            onclick: move |_| {
                notice.open();
            },
            "Open"
        }
    }
}

#[component]
fn SelfRemoving() -> Element {
    let mut shown = use_signal(|| true);
    rsx! {
        Dialog { title: "One step", close_button: false,
            if shown() {
                Button { id: "remove-me", onclick: move |_| shown.set(false), "Done" }
            } else {
                Text { "All done." }
            }
        }
    }
}

/// A classic scrollbar on a page that scrolls: opening must not shift it (todo 1305).
#[component]
fn ScrollbarModalPage() -> Element {
    let prompt = use_modal(|s: ModalScope<()>| {
        rsx! {
            Dialog { title: "Locked",
                Button { id: "close", onclick: move |_| s.close(), "Close" }
            }
        }
    });

    rsx! {
        // Headless Chromium draws overlay scrollbars; this one takes room.
        style { "::-webkit-scrollbar {{ width: 17px; }} ::-webkit-scrollbar-thumb {{ background: #888; }}" }
        Button {
            id: "open-modal",
            onclick: move |_| {
                prompt.open();
            },
            "Open"
        }
        div { id: "ruler", style: "height: 3000px; border-right: 1px solid;", "Tall page" }
    }
}

/// The component that called `use_modal` goes away while it is open (todo 1306).
#[component]
fn OwnerModalPage() -> Element {
    let mut owner_shown = use_signal(|| true);
    let mut requests = use_signal(|| 0_u32);
    let result = use_signal(String::new);

    rsx! {
        // Outside the owner, so focus has somewhere to return to.
        Button { id: "open-modal", onclick: move |_| requests += 1, "Open" }
        div { id: "result", "{result}" }
        if owner_shown() {
            Owner { requests, result, onleave: move |_| owner_shown.set(false) }
        }
    }
}

#[component]
fn Owner(requests: ReadSignal<u32>, result: Signal<String>, onleave: EventHandler<()>) -> Element {
    let modal = use_modal(move |_: ModalScope<()>| {
        rsx! {
            Dialog { title: "Leaving",
                Button { id: "leave", onclick: move |_| onleave.call(()), "Leave" }
            }
        }
    });
    use_effect(move || {
        if requests() > 0 {
            let mut result = result;
            modal
                .open()
                .onresult(move |outcome| result.set(format!("{outcome:?}")));
        }
    });
    rsx! {}
}

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
