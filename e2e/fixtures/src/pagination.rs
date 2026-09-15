//! `Pagination`, answered at once and answered late.

use dioxus::prelude::*;
use libero::components::{Fieldset, Flex, Pagination};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/pagination", || rsx! { PaginationPage { delay_ms: 0 } }),
    (
        "/pagination-async",
        || rsx! { PaginationPage { delay_ms: 150 } },
    ),
    ("/pagination/states", || rsx! { PaginationStatesPage {} }),
];

/// Disabled by its prop and by a `Fieldset`, one page, and the smallest size.
#[component]
fn PaginationStatesPage() -> Element {
    let mut page = use_signal(|| 20u32);
    rsx! {
        Flex { direction: "column", gap: "md",
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
                    spawn(async move {
                        let _ = document::eval(&format!(
                            "await new Promise(r => setTimeout(r, {delay_ms})); return 1;"
                        ))
                        .await;
                        page.set(next);
                    });
                },
            }
            span { id: "page", "data-page": "{page}", "data-changes": "{changes}" }
        }
    }
}
