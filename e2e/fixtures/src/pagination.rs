//! `Pagination`, answered at once and answered late.

use dioxus::prelude::*;
use libero::components::{Flex, Pagination};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/pagination", || rsx! { PaginationPage { delay_ms: 0 } }),
    (
        "/pagination-async",
        || rsx! { PaginationPage { delay_ms: 150 } },
    ),
];

/// `delay_ms` sets the page only after a timer, as a caller that fetches
/// first does. The focus repair has to survive that (Karen3's finding).
#[component]
fn PaginationPage(delay_ms: u32) -> Element {
    let mut page = use_signal(|| 2u32);

    rsx! {
        Flex { direction: "column", gap: "md",
            Pagination {
                total: 10,
                page: page(),
                aria_label: "Results pages",
                with_edges: true,
                onchange: move |next: u32| {
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
            span { id: "page", "data-page": "{page}" }
        }
    }
}
