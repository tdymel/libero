//! Button-like controls that turn disabled while focused (todo 3838). `#flip` toggles them all,
//! by their own `disabled` or, on `/soft-disabled/fieldset`, by a `Fieldset` around them (3873).

use dioxus::prelude::*;
use libero::components::{
    ActionIcon, BottomNavigation, BottomNavigationItem, Burger, Button, Chip, Fieldset, Flex,
    NavLink, PasswordField,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/soft-disabled", || rsx! { SoftDisabledPage { in_fieldset: false } }),
    ("/soft-disabled/fieldset", || rsx! { SoftDisabledPage { in_fieldset: true } }),
];

#[component]
fn SoftDisabledPage(in_fieldset: bool) -> Element {
    let mut off = use_signal(|| false);
    let mut clicks = use_signal(|| 0);
    let disabled = off() && !in_fieldset;
    let controls = rsx! {
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
    };
    rsx! {
        Flex { direction: "column", gap: "md", align: "start",
            button { id: "flip", onclick: move |_| off.toggle(), "Flip" }
            span { id: "clicks", "data-clicks": "{clicks}" }
            if in_fieldset {
                Fieldset::<()> { label: "Controls", disabled: off(), {controls} }
            } else {
                {controls}
            }
        }
    }
}
