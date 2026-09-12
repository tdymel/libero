//! `Chip`, in every kind its docs page shows: tag, filter, action, link.

use dioxus::prelude::*;
use libero::components::{Chip, Flex};

use crate::Routes;

pub const ROUTES: Routes = &[("/chip", || rsx! { ChipPage {} })];

/// Every chip reports what it last emitted into `data-emitted`.
#[component]
fn ChipPage() -> Element {
    let mut rust = use_signal(|| true);
    let mut css = use_signal(|| false);
    let mut small = use_signal(|| false);
    let mut emitted = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px", "data-emitted": emitted(),
            Flex { direction: "row", gap: "md", wrap: "wrap",
                Chip { id: "tag", "Tag" }
                Chip {
                    id: "rust",
                    checked: rust(),
                    onchange: move |next| {
                        rust.set(next);
                        emitted.set(format!("rust:{next}"));
                    },
                    "rust"
                }
                Chip {
                    id: "css",
                    variant: "outlined",
                    checked: css(),
                    onchange: move |next| {
                        css.set(next);
                        emitted.set(format!("css:{next}"));
                    },
                    "css"
                }
                Chip { id: "named", name: "tags", value: "html", "html" }
                Chip {
                    id: "off",
                    disabled: true,
                    checked: false,
                    onchange: move |next| emitted.set(format!("off:{next}")),
                    "Unavailable"
                }
            }
            Flex { direction: "row", gap: "md", wrap: "wrap",
                Chip {
                    id: "action",
                    onclick: move |_| emitted.set("action".to_string()),
                    "Action"
                }
                Chip { id: "link", to: "/chip", "Link" }
                Chip { id: "dead-link", to: "/chip", disabled: true, "Dead link" }
                // The other variants, so axe sees every look's text.
                Chip { id: "tonal", variant: "tonal", name: "look", value: "tonal", "tonal" }
                Chip { id: "elevated", variant: "elevated", name: "look", value: "elevated", "elevated" }
                Chip { id: "text", variant: "text", name: "look", value: "text", "text" }
                Chip {
                    id: "small",
                    size: "xs",
                    checked: small(),
                    onchange: move |next| {
                        small.set(next);
                        emitted.set(format!("small:{next}"));
                    },
                    "small"
                }
            }
        }
    }
}
