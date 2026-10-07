//! `Pagination`, answered at once and answered late.

use std::time::Duration;

use dioxus::prelude::*;
use libero::{
    components::{Fieldset, Flex, Pagination},
    platform::timer,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/pagination", || rsx! { PaginationPage { delay_ms: 0 } }),
    (
        "/pagination-async",
        || rsx! { PaginationPage { delay_ms: 150 } },
    ),
    ("/pagination/states", || rsx! { PaginationStatesPage {} }),
    (
        "/pagination/rejecting",
        || rsx! { PaginationRejectingPage {} },
    ),
];

/// `onchange` ignores the pagination; only `#jump` moves the page (todo 1589).
#[component]
fn PaginationRejectingPage() -> Element {
    let mut page = use_signal(|| 2u32);
    rsx! {
        Flex { direction: "column", gap: "md",
            Pagination { total: 10, page: page(), aria_label: "Results pages",
                with_edges: true, onchange: move |_| {},
            }
            input {
                id: "jump",
                "aria-label": "Jump to page",
                oninput: move |event: FormEvent| {
                    if let Ok(next) = event.value().parse() {
                        page.set(next);
                    }
                },
            }
            span { id: "page", "data-page": "{page}" }
        }
    }
}

/// Disabled by its prop and by a `Fieldset`, one page, the smallest size, and a
/// `disabled` that `#flip` toggles (todo 2418).
#[component]
fn PaginationStatesPage() -> Element {
    let mut page = use_signal(|| 20u32);
    let mut flip_page = use_signal(|| 2u32);
    let mut off = use_signal(|| false);
    rsx! {
        Flex { direction: "column", gap: "md",
            Pagination { id: "pg-flip", total: 10, page: flip_page(), aria_label: "Flipped",
                disabled: off(), onchange: move |next| flip_page.set(next),
            }
            button { id: "flip", onclick: move |_| off.toggle(), "Toggle disabled" }
            span { id: "flip-page", "data-page": "{flip_page}" }
            Pagination { id: "pg-disabled", total: 10, page: 5, aria_label: "Disabled",
                with_edges: true, disabled: true, onchange: move |_| {},
            }
            Fieldset::<()> { label: "Off", disabled: true,
                Pagination { id: "pg-fieldset", total: 10, page: 5, aria_label: "In a fieldset",
                    with_edges: true, onchange: move |_| {},
                }
            }
            Pagination { id: "pg-one", total: 1, page: 1, aria_label: "One page",
                with_edges: true, onchange: move |_| {},
            }
            Pagination { id: "pg-xs", total: 42, page: page(), aria_label: "Small", size: "xs",
                with_edges: true, onchange: move |next| page.set(next),
            }
        }
    }
}

/// `delay_ms` sets the page only after a timer, as a caller that fetches
/// first does. The focus repair has to survive that (Karen3's finding).
#[component]
fn PaginationPage(delay_ms: u32) -> Element {
    let mut page = use_signal(|| 2u32);
    let mut changes = use_signal(|| 0u32);
    // libero's timer, not `document::eval`: the page mounts in Blitz too (822).
    let mut pending = use_hook(|| CopyValue::new(None));

    rsx! {
        Flex { direction: "column", gap: "md",
            Pagination {
                total: 10,
                page: page(),
                aria_label: "Results pages",
                with_edges: true,
                onchange: move |next: u32| {
                    changes += 1;
                    if delay_ms == 0 {
                        page.set(next);
                        return;
                    }
                    let delay = Duration::from_millis(u64::from(delay_ms));
                    pending.set(timer().map(|timer| timer.after(delay, Box::new(move || page.set(next)))));
                },
            }
            span { id: "page", "data-page": "{page}", "data-changes": "{changes}" }
        }
    }
}
