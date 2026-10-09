//! The small display labels: `Badge` in every variant and colour, `Indicator`
//! counts, a ringed and a processing one on an `Avatar`, `Kbd` at every size, and
//! nested, ordered and icon `List`s.

use dioxus::prelude::*;
use libero::components::{Avatar, Badge, Flex, Float, Indicator, Kbd, List, ListItem};

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
            // 2105: a literal fill takes black or white, not the page's text colour.
            Flex { gap: "xs", wrap: "wrap",
                Badge { id: "literal-light", color: "#ffeb3b", "Light" }
                Badge { id: "literal-dark", color: "#123456", "Dark" }
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
            // The real use: on a picture, in a `Float` on a positioned parent.
            Flex { gap: "md", align: "center",
                div { id: "ringed-host", position: "relative", width: "fit-content",
                    Avatar { name: "Ada Lovelace", initials: "AL" }
                    Float { Indicator { id: "ringed", with_border: true, color: "success" } }
                }
                div { id: "processing-host", position: "relative", width: "fit-content",
                    Avatar { name: "Grace Hopper", initials: "GH" }
                    Float { Indicator { id: "processing", processing: true } }
                }
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
            List { id: "icon-list", icon: rsx! { span { "✓" } },
                ListItem { "Keyboard" }
                ListItem { "Screen reader" }
            }
            List { id: "ordered-icon-list", ordered: true, icon: rsx! { span { "✓" } },
                ListItem { "Install" }
                ListItem { "Configure" }
            }
            List { id: "ordered-mixed-list", ordered: true,
                ListItem { icon: rsx! { span { "✓" } }, "Own icon" }
                ListItem { "Numbered" }
            }
        }
    }
}
