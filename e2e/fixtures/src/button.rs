//! `Button`, as a button and as a router link.

use dioxus::prelude::*;
use libero::{
    components::{Box, Button, Flex, Text},
    sx::sx,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/button", || rsx! { ButtonPage {} }),
    ("/button/landing", || rsx! { ButtonLanding {} }),
];

/// A plain button and a busy one, each counting its activations, and a
/// link-mode button whose route is one only this page leads to. Below them a
/// busy submit button in a form, a long label, buttons laying out element
/// children, and a toggle pair per variant.
#[component]
fn ButtonPage() -> Element {
    let mut plain = use_signal(|| 0u32);
    let mut busy = use_signal(|| 0u32);
    let mut submits = use_signal(|| 0u32);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "plain", onclick: move |_| plain += 1, "Plain" }
            Button { id: "busy", loading: true, onclick: move |_| busy += 1, "Busy" }
            Text { id: "presses", "data-plain": "{plain}", "data-busy": "{busy}",
                "Plain {plain}, busy {busy}"
            }
            Button { id: "to-landing", to: "/button/landing", "Go to landing" }
            // Todo 452: one per label rule, for the hover contrast test.
            Button { id: "outlined", variant: "outlined", "Outlined" }
            Button { id: "standard-error", variant: "standard", color: "error", "Standard" }
            Button { id: "elevated", variant: "elevated", color: "secondary", "Elevated" }
            Button { id: "filled-muted", color: "muted", "Filled" }
            Button { id: "disabled-link", to: "/button/landing", disabled: true, "Disabled link" }
            form { id: "form", "data-submits": "{submits}", onsubmit: move |event| { event.prevent_default(); submits += 1; },
                input { id: "field", aria_label: "Field" }
                Button { id: "busy-submit", r#type: "submit", loading: true, "Send" }
            }
            Button { id: "long", "Save every change made to this rather long document title" }
            // The docs search field and colour swatches, cut down: a caller
            // unwraps the label span to lay its element children out itself.
            Button {
                id: "search-like",
                variant: "outlined",
                sx: sx()
                    .width("240px")
                    .gap("sm")
                    .justify_content("flex-start")
                    .selector("& > [data-slot='label']", sx().display("contents")),
                span { id: "search-icon", display: "inline-flex", width: "16px", height: "16px" }
                "Search"
                span { id: "search-kbd", display: "inline-block", width: "24px", height: "12px", margin_left: "auto" }
            }
            Button {
                id: "swatch",
                variant: "standard",
                aria_label: "Blue",
                sx: sx()
                    .width("48px")
                    .padding("0")
                    .selector("& > [data-slot='label']", sx().display("contents")),
                Box { id: "swatch-fill", sx: sx().width("100%").height("100%").background("blue") }
            }
            for variant in ["filled", "tonal", "elevated", "outlined", "standard"] {
                Flex { gap: "sm",
                    Button { id: "off-{variant}", variant, selected: false, "Bold" }
                    Button { id: "on-{variant}", variant, selected: true, "Bold" }
                }
            }
        }
    }
}

#[component]
fn ButtonLanding() -> Element {
    rsx! { Text { id: "landing", "Landed" } }
}
