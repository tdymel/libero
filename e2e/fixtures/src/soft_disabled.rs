//! Button-like controls that turn disabled while focused (todo 3838). `#flip` toggles them all.

use dioxus::prelude::*;
use libero::components::{
    ActionIcon, BottomNavigation, BottomNavigationItem, Burger, Button, Chip, Flex, NavLink,
    PasswordField,
};

use crate::Routes;

pub const ROUTES: Routes = &[("/soft-disabled", || rsx! { SoftDisabledPage {} })];

#[component]
fn SoftDisabledPage() -> Element {
    let mut off = use_signal(|| false);
    let mut clicks = use_signal(|| 0);
    let disabled = off();
    rsx! {
        Flex { direction: "column", gap: "md", align: "start",
            button { id: "flip", onclick: move |_| off.toggle(), "Flip" }
            span { id: "clicks", "data-clicks": "{clicks}" }
            Button { id: "button", disabled, onclick: move |_| clicks += 1, "Save" }
            ActionIcon { id: "icon", aria_label: "Close", disabled, onclick: move |_| clicks += 1, "x" }
            Burger { id: "burger", disabled, onclick: move |_| clicks += 1 }
            Chip { id: "chip", disabled, onclick: move |_| clicks += 1, "Filter" }
            div { id: "nav",
                NavLink {
                    to: "/soft-disabled",
                    disabled,
                    onchange: move |_| clicks += 1,
                    nested: rsx! {
                        NavLink { to: "/soft-disabled", "Child" }
                    },
                    "Parent"
                }
            }
            div { id: "password",
                PasswordField { label: "Password", disabled }
            }
            BottomNavigation { "aria-label": "Main",
                BottomNavigationItem {
                    id: "item",
                    disabled,
                    onclick: move |_| clicks += 1,
                    icon: rsx! { "H" },
                    "Home"
                }
            }
        }
    }
}
