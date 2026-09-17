//! The small display labels: `Badge` in every variant and colour, `Indicator`
//! counts, `Kbd` at every size, and a nested `List`.

use dioxus::prelude::*;
use libero::components::{Badge, Flex, Indicator, Kbd, List, ListItem};

use crate::Routes;

pub const ROUTES: Routes = &[("/badge", || rsx! { BadgePage {} })];

const VARIANTS: [&str; 5] = ["filled", "tonal", "elevated", "outlined", "standard"];
const COLORS: [&str; 6] = [
    "primary",
    "secondary",
    "error",
    "info",
    "success",
    "warning",
];
const SIZES: [&str; 6] = ["xs", "sm", "md", "lg", "xl", "xxl"];

#[component]
fn BadgePage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "480px",
            // The smallest size in every variant and colour: the worst text case.
            for variant in VARIANTS {
                Flex { gap: "xs", wrap: "wrap",
                    for color in COLORS {
                        Badge { variant, color, size: "xs", "{color}" }
                    }
                }
            }
            Flex { id: "badge-sizes", gap: "xs", wrap: "wrap", align: "center",
                for size in SIZES {
                    Badge { size, "{size}" }
                }
                Badge { circle: true, "9" }
            }
            Flex { gap: "md", wrap: "wrap", align: "center",
                for color in COLORS {
                    Indicator { label: 128, color }
                }
                for size in SIZES {
                    Indicator { label: 7, size }
                }
                Indicator { id: "dot" }
            }
            Flex { gap: "xs", wrap: "wrap", align: "center",
                for size in SIZES {
                    Kbd { size, "Ctrl" }
                }
            }
            List { id: "list",
                ListItem { "First" }
                ListItem {
                    "Second"
                    List {
                        ListItem { "Nested" }
                    }
                }
            }
            List { id: "ordered-list", ordered: true,
                ListItem { "Install" }
                ListItem { "Configure" }
            }
        }
    }
}
