//! `VisuallyHidden { focusable }`: a skip link that shows itself on focus
//! (todo 613).

use dioxus::prelude::*;
use libero::components::{Header, VisuallyHidden};

use crate::Routes;

pub const ROUTES: Routes = &[("/visually-hidden", || rsx! { VisuallyHiddenPage {} })];

/// The skip link comes first, ahead of the sticky `Header` that would cover it (2.4.11).
/// Its target is not `#main`: the app root owns that id.
#[component]
fn VisuallyHiddenPage() -> Element {
    rsx! {
        VisuallyHidden { id: "skip", focusable: true,
            a { id: "skip-link", href: "#page", "Skip to content" }
        }
        Header { id: "banner", "Banner" }
        main { id: "page", tabindex: "-1", "Content" }
    }
}
