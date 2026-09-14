//! The plain layout wrappers: `Box` as other tags, `Container`, `Flex`,
//! `Center`, `Float` and `AspectRatio`, in a 320px column (WCAG 1.4.10).

use dioxus::prelude::*;
use libero::components::{AspectRatio, Box as LBox, Center, Container, Flex, Float};

use crate::Routes;

pub const ROUTES: Routes = &[("/layout", || rsx! { LayoutPage {} })];

#[component]
fn LayoutPage() -> Element {
    rsx! {
        div { id: "column", max_width: "320px",
            LBox { id: "poly-nav", component: "nav", aria_label: "Sections",
                LBox { component: "ul",
                    LBox { component: "li",
                        LBox { id: "poly-link", component: "a", href: "#container", "Container" }
                    }
                }
            }
            LBox { id: "poly-button", component: "button", r#type: "button", aria_pressed: "false", "Toggle" }
            Container { id: "container", component: "section", aria_label: "Container",
                "A container keeps its gutters and never runs past a narrow screen."
            }
            Flex { id: "row", direction: "row", wrap: true,
                LBox { component: "span", "First" }
                LBox { component: "span", "Second" }
                LBox { component: "span", "Third" }
            }
            Center { id: "stage", position: "relative", min_height: "64px",
                "Centred"
                Float { id: "float", placement: "top-end", LBox { component: "span", "New" } }
            }
            AspectRatio { id: "ratio", ratio: 16.0 / 9.0,
                LBox { id: "ratio-link", component: "a", href: "#ratio", "Play the video" }
            }
        }
    }
}
