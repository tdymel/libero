use dioxus::prelude::*;
use libero::{
    components::{Anchor, Drawer, Flex, Icon, List, ListItem, Title},
    sx::sx,
};

use crate::Route;

fn chevron() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M9 18l6-6-6-6" }
        }
    }
}

#[component]
pub fn Sidebar() -> Element {
    rsx! {
        Drawer {
            variant: "static",
            anchor: "left",
            size: "sm",
            role: "navigation",
            sx: sx().flex_shrink("0").padding("32px 24px"),
            Flex {
                direction: "column",
                gap: "12px",
                Title { variant: "h3", component: "p", "Documentation" }
                List {
                    ListItem {
                        Anchor {
                            to: Route::GettingStarted {},
                            underline: "never",
                            sx: sx().display("flex").align_items("center").gap("6px").font_weight("600"),
                            Icon { variant: "transparent", size: "xs", color: "primary", {chevron()} }
                            "Getting Started"
                        }
                    }
                    ListItem {
                        Anchor {
                            to: Route::IconPage {},
                            underline: "never",
                            sx: sx().display("flex").align_items("center").gap("6px").font_weight("600"),
                            Icon { variant: "transparent", size: "xs", color: "primary", {chevron()} }
                            "Icon"
                        }
                    }
                }
            }
        }
    }
}
