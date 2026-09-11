//! An active `NavLink` with `scroll_into_view`, below the fold of its sidebar.

use dioxus::prelude::*;
use libero::components::NavLink;

use crate::Routes;

pub const ROUTES: Routes = &[("/nav-link", || rsx! { NavLinkPage {} })];

#[component]
fn NavLinkPage() -> Element {
    rsx! {
        div { id: "sidebar", style: "height: 400px; overflow-y: auto;",
            div { style: "height: 1000px;" }
            NavLink {
                id: "here",
                to: "https://example.com",
                active: true,
                scroll_into_view: true,
                "Here"
            }
            div { style: "height: 1000px;" }
        }
    }
}
