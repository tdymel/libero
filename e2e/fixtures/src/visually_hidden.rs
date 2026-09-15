//! `VisuallyHidden { focusable }`: a skip link that shows itself on focus
//! (todo 613).

use dioxus::prelude::*;
use libero::components::VisuallyHidden;

use crate::Routes;

pub const ROUTES: Routes = &[("/visually-hidden", || rsx! { VisuallyHiddenPage {} })];

#[component]
fn VisuallyHiddenPage() -> Element {
    rsx! {
        VisuallyHidden { id: "skip", focusable: true,
            a { id: "skip-link", href: "#main", "Skip to content" }
        }
        main { id: "main", tabindex: "-1", "Content" }
    }
}
