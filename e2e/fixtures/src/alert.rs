//! `Alert`: every severity in every variant, a dismissible one, a long title and styled parts.

use dioxus::prelude::*;
use libero::components::{Alert, AlertPart, Anchor, Button, Flex, Part, Parts};
use libero::sx::sx;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/alert", || rsx! { AlertPage {} }),
    ("/alert/long-action", || rsx! { LongActionPage {} }),
];

const COLORS: [&str; 4] = ["info", "success", "warning", "error"];
const VARIANTS: [&str; 5] = ["filled", "tonal", "outlined", "elevated", "standard"];

/// An action label with no break point, wider than the 320px stage (todo 2762).
#[component]
fn LongActionPage() -> Element {
    rsx! {
        Flex { direction: "column", max_width: "320px",
            Alert { id: "long-action",
                actions: rsx! {
                    Button { size: "sm", "Supercalifragilisticexpialidocious_unbreakable_label" }
                },
                "Cancel before then."
            }
        }
    }
}

#[component]
fn AlertPage() -> Element {
    let mut shown = use_signal(|| true);
    let mut wide = use_signal(|| true);

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
            Button { id: "toggle-parts", variant: "outlined", onclick: move |_| wide.toggle(), "Toggle the message inset" }
            Alert { id: "styled-parts",
                title: "Styled parts",
                parts: Parts::new()
                    .part(AlertPart::Title, sx().font_style("italic").letter_spacing("2px"))
                    .part(
                        AlertPart::Message,
                        sx().with("padding-left", if wide() { "8px" } else { "4px" }),
                    ),
                // The instance `sx` wins the tie on the title's letter spacing.
                sx: sx().selector(AlertPart::Title.selector(), sx().letter_spacing("4px")),
                "Parts styled."
                Alert { id: "nested", title: "Nested", "Not reached." }
            }
            for color in COLORS {
                for variant in VARIANTS {
                    Alert { key: "{color}-{variant}", id: "{color}-{variant}", color, variant,
                        title: "{color} {variant}",
                        "The message, with "
                        // The page's link colour is unreadable on a fill (todo 1576).
                        Anchor { to: "#{color}-{variant}", "a link" }
                        "."
                    }
                }
            }
        }
    }
}
