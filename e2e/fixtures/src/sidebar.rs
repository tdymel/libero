//! `Sidebar` in a 320px column: overflowing panels with nothing focusable (a tab stop
//! named through `aria-label` and `aria-labelledby`), `top` / `bottom` in a column,
//! content wider than the panel, and the default size beside content.

use dioxus::prelude::*;
use libero::components::{Box as LBox, Flex, Sidebar};
use libero::sx::sx;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/sidebar", || rsx! { SidebarPage {} }),
    ("/sidebar/narrow", || rsx! { NarrowPage {} }),
    ("/sidebar/short", || rsx! { ShortPage {} }),
];

/// The default `md` top and bottom panel, each in a 256px column; not in the baseline
/// (todo 2645).
#[component]
fn ShortPage() -> Element {
    rsx! {
        Flex { id: "short-a", direction: "column", height: "256px",
            Sidebar { id: "short-top", side: "top", aria_label: "Toolbar", "Toolbar" }
            LBox { id: "short-rest", sx: sx().flex("1").min_height("0"), "Body" }
        }
        Flex { id: "short-b", direction: "column", height: "256px",
            LBox { sx: sx().flex("1").min_height("0"), "Body" }
            Sidebar { id: "short-bottom", side: "bottom", aria_label: "Status", "Status" }
        }
    }
}

/// The default `md` panel beside content in a 320px column; not in the baseline
/// (todo 2477).
#[component]
fn NarrowPage() -> Element {
    rsx! {
        div { id: "column", max_width: "320px",
            Flex { direction: "row", align: "stretch", wrap: false, height: "120px",
                Sidebar { id: "default", aria_label: "Filters", "Filters" }
                LBox { id: "default-rest", sx: sx().flex("1").min_width("0"), "Results" }
            }
        }
    }
}

#[component]
fn SidebarPage() -> Element {
    rsx! {
        div { id: "column", max_width: "320px",
            // Overflowing its 120px row with nothing focusable.
            Flex { direction: "row", align: "stretch", wrap: false, height: "120px",
                Sidebar { id: "start", size: "xs", aria_label: "Filters",
                    for i in 1..=8 { p { "Filter {i}" } }
                }
                LBox { sx: sx().flex("1").min_width("0"), "Results" }
            }
            p { id: "inspector-heading", "Inspector" }
            Flex { direction: "row", align: "stretch", wrap: false, height: "80px",
                LBox { sx: sx().flex("1").min_width("0"), "Page" }
                Sidebar { id: "end", side: "end", size: "xs", aria_labelledby: "inspector-heading",
                    for i in 1..=8 { p { "Property {i}" } }
                }
            }
            Flex { id: "stack", direction: "column", height: "480px",
                Sidebar { id: "top", side: "top", size: "xs", aria_label: "Toolbar", "Toolbar" }
                LBox { sx: sx().flex("1"), "Body" }
                Sidebar { id: "bottom", side: "bottom", size: "xs", aria_label: "Status", "Status" }
            }
            Flex { direction: "row", align: "stretch", wrap: false, height: "80px",
                Sidebar { id: "wide", size: "xs", aria_label: "Wide",
                    LBox { id: "wide-content", sx: sx().white_space("nowrap"), "A line of filters far wider than its panel" }
                }
                LBox { sx: sx().flex("1").min_width("0"), "Rest" }
            }
        }
    }
}
