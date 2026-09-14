//! `Alert`: every severity in every variant, a dismissible one and a long title.

use dioxus::prelude::*;
use libero::components::{Alert, Button, Flex};

use crate::Routes;

pub const ROUTES: Routes = &[("/alert", || rsx! { AlertPage {} })];

const COLORS: [&str; 4] = ["info", "success", "warning", "error"];
const VARIANTS: [&str; 5] = ["filled", "tonal", "outlined", "elevated", "standard"];

#[component]
fn AlertPage() -> Element {
    let mut shown = use_signal(|| true);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "show", variant: "outlined", onclick: move |_| shown.set(true), "Show the notice" }
            if shown() {
                Alert {
                    id: "dismissible",
                    title: "Card expiring",
                    onclose: move |_| shown.set(false),
                    "Update it before the next invoice."
                }
            }
            Alert { id: "long-title", color: "warning",
                title: "Your subscription renews on the first of next month at the annual rate",
                "Cancel before then to avoid the charge."
            }
            for color in COLORS {
                for variant in VARIANTS {
                    Alert { key: "{color}-{variant}", id: "{color}-{variant}", color, variant,
                        title: "{color} {variant}",
                        "The message."
                    }
                }
            }
        }
    }
}
