//! `Burger` as a disclosure button for a panel.

use dioxus::prelude::*;
use libero::components::{Burger, Flex, Text};

use crate::Routes;

pub const ROUTES: Routes = &[("/burger", || rsx! { BurgerPage {} })];

#[component]
fn BurgerPage() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        Flex { direction: "row", gap: "md", align: "center",
            Text { "Site" }
            Burger {
                id: "burger",
                open: open(),
                "aria-controls": "panel",
                onclick: move |_| open.toggle(),
            }
            Burger { id: "locked", open: false, "aria-controls": "panel", disabled: true }
            Text { id: "panel", hidden: !open(), "Menu" }
        }
    }
}
