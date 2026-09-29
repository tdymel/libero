//! The plain layout wrappers: `Box` as other tags, `Container`, `Flex`,
//! `Center`, `Float`, `AspectRatio` and `Sidebar`, in a 320px column (WCAG 1.4.10).

use dioxus::prelude::*;
use libero::components::{AspectRatio, Box as LBox, Center, Container, Flex, Float, Sidebar};
use libero::sx::sx;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/layout", || rsx! { LayoutPage {} }),
    ("/center-overflow", || rsx! { CenterOverflowPage {} }),
];

/// A child wider and taller than its `Center`: centring must not push it past the start edges.
#[component]
fn CenterOverflowPage() -> Element {
    rsx! {
        div { id: "column", max_width: "320px",
            Center { id: "stage", sx: sx().height("40px"),
                LBox { id: "wide", sx: sx().white_space("nowrap").height("80px"), "Wider and taller than the stage, and it will not wrap at all" }
            }
        }
    }
}

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
            // No `wrap`: a row wraps by default, or these overflow 320px.
            Flex { id: "row", direction: "row",
                LBox { component: "span", "The first row item" }
                LBox { component: "span", "The second row item" }
                LBox { component: "span", "The third row item" }
            }
            Center { id: "stage", position: "relative", min_height: "64px",
                "Centred"
                Float { id: "float", placement: "top-end", LBox { component: "span", "New" } }
            }
            AspectRatio { id: "ratio", ratio: 16.0 / 9.0,
                LBox { id: "ratio-link", component: "a", href: "#ratio", "Play the video" }
            }
            Sidebar { id: "sidebar", aria_label: "Filters", size: "xs", "Filters" }
            // An outer `wrap`, `align` and `justify` must not reach the inner Flex.
            Flex { id: "outer", direction: "row", wrap: false, align: "flex-end", justify: "center",
                Flex { id: "inner", direction: "row",
                    LBox { component: "span", "Inner" }
                }
            }
        }
    }
}
