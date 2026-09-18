//! `use_focus_return`, as its docs page uses it: a panel of the caller's own.

use dioxus::prelude::*;
use libero::{
    components::{Button, Checkbox, Flex},
    hooks::use_focus_return,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/focus-return", || rsx! { FiltersPage {} }),
    ("/focus-return/mounted", || rsx! { MountedPage {} }),
];

/// The `remember(event)` arm: the trigger is named once from `onmounted`.
#[component]
fn MountedPage() -> Element {
    let mut open = use_signal(|| false);
    let trigger = use_focus_return();

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            button {
                id: "trigger",
                aria_expanded: open(),
                onmounted: move |event| trigger.remember(event),
                onclick: move |_| open.toggle(),
                "Details"
            }
            if open() {
                Button {
                    id: "close",
                    onclick: move |_| {
                        open.set(false);
                        trigger.restore();
                    },
                    "Close"
                }
            }
        }
    }
}

#[component]
fn FiltersPage() -> Element {
    let mut open = use_signal(|| false);
    let mut in_stock = use_signal(|| false);
    let trigger = use_focus_return();

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button {
                id: "filters",
                variant: "outlined",
                aria_expanded: open(),
                aria_controls: "filters-panel",
                onclick: move |_| {
                    if !open() {
                        trigger.remember_active();
                    }
                    open.toggle();
                },
                "Filters"
            }
            if open() {
                Flex {
                    id: "filters-panel",
                    direction: "column",
                    gap: "sm",
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() == Key::Escape {
                            open.set(false);
                            trigger.restore();
                        }
                    },
                    Checkbox {
                        label: "In stock only",
                        checked: in_stock(),
                        onchange: move |next| in_stock.set(next),
                    }
                    Button {
                        id: "apply",
                        onclick: move |_| {
                            open.set(false);
                            trigger.restore();
                        },
                        "Apply"
                    }
                }
            }
        }
    }
}
