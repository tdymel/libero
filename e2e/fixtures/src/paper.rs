//! `Paper` as a plain surface, as a link and as a button: only the pressable two take the
//! interactive look.

use dioxus::prelude::*;
use libero::components::{Flex, Paper, Text};
use libero::sx::sx;

use crate::Routes;

pub const ROUTES: Routes = &[("/paper", || rsx! { PaperPage {} })];

#[component]
fn PaperPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Paper { id: "plain", sx: sx().padding("lg"), Text { "Plain" } }
            Paper {
                id: "link",
                component: "a",
                href: "#order-4021",
                sx: sx().padding("lg"),
                Text { "Link" }
            }
            form {
                Paper { id: "press", component: "button", Text { "Press" } }
                Paper { id: "press-bordered", component: "button", bordered: true, Text { "Bordered" } }
                Paper { id: "press-primary", component: "button", color: "primary", Text { "Primary" } }
            }
            Paper {
                id: "gradient-link",
                component: "a",
                href: "#order-4021",
                gradient: ("info", 90),
                sx: sx().padding("lg"),
                Text { "Gradient link" }
            }
            Paper {
                id: "glass-link",
                component: "a",
                href: "#order-4021",
                glass: true,
                color: "primary",
                sx: sx().padding("lg"),
                Text { "Glass link" }
            }
            Paper {
                id: "glass-gradient-link",
                component: "a",
                href: "#order-4021",
                glass: true,
                gradient: ("info", 90),
                sx: sx().padding("lg"),
                Text { "Glass gradient link" }
            }
        }
    }
}
