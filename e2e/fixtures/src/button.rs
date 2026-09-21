//! `Button`, as a button and as a router link.

use dioxus::prelude::*;
use libero::{
    components::{Badge, Box, Button, Flex, Text},
    sx::sx,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/button", || rsx! { ButtonPage {} }),
    ("/button/landing", || rsx! { ButtonLanding {} }),
];

/// Plain and busy buttons counting activations, a link-mode button, a busy submit, a long
/// label, element children and a toggle pair per variant.
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
            // Todo 877: a literal fill's label is black or white, not the page text.
            Button { id: "literal-filled", color: "#ffeb3b", "Yellow" }
            Button { id: "literal-tonal", variant: "tonal", color: "#123456", "Navy" }
            Button { id: "literal-named", color: "gold", "Gold" }
            Button { id: "disabled-link", to: "/button/landing", disabled: true, "Disabled link" }
            form { id: "form", "data-submits": "{submits}", onsubmit: move |event| { event.prevent_default(); submits += 1; },
                input { id: "field", aria_label: "Field" }
                Button { id: "busy-submit", r#type: "submit", loading: true, "Send" }
            }
            Button { id: "long", "Save every change made to this rather long document title" }
            // The docs search field and colour swatches, cut down: element
            // children are the button's own flex items (todo 662).
            Button {
                id: "search-like",
                variant: "outlined",
                sx: sx()
                    .width("240px")
                    .gap("sm")
                    .justify_content("flex-start"),
                span { id: "search-icon", display: "inline-flex", width: "16px", height: "16px" }
                "Search"
                span { id: "search-kbd", display: "inline-block", width: "24px", height: "12px", margin_left: "auto" }
            }
            Button {
                id: "swatch",
                variant: "standard",
                aria_label: "Blue",
                sx: sx().width("48px").padding("0"),
                Box { id: "swatch-fill", sx: sx().width("100%").height("100%").background("blue") }
            }
            Button { id: "with-icon",
                icon: rsx! { svg { id: "with-icon-glyph", width: "16", height: "16", view_box: "0 0 24 24",
                    circle { cx: "12", cy: "12", r: "8" }
                } },
                span { id: "with-icon-text", "Save" }
            }
            for variant in ["filled", "tonal", "elevated", "outlined", "standard"] {
                Flex { gap: "sm",
                    Button { id: "off-{variant}", variant, selected: false, "Bold" }
                    Button { id: "on-{variant}", variant, selected: true, "Bold" }
                }
            }
            // Todo 686: a badge with its own fill inside a pressed button. `currentColor`:
            // an error badge's text failed axe on the pressed fill (715).
            Flex { gap: "sm",
                Button { id: "off-badge", selected: false, "Inbox" Badge { "3" } }
                Button { id: "on-badge", selected: true, "Inbox"
                    Badge { "3" }
                    Badge { variant: "outlined", color: "currentColor", "New" }
                }
            }
        }
    }
}

#[component]
fn ButtonLanding() -> Element {
    rsx! { Text { id: "landing", "Landed" } }
}
