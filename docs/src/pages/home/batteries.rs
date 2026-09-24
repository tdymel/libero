use dioxus::prelude::*;
use libero::{
    components::{Anchor, Box, Code, Flex, Icon, List, ListItem, Paper, Pictogram, Text, Title},
    sx::sx,
    theme::Size,
};
use pictogram_icons_lucide as lucide;

use super::SectionTitle;
use crate::site::GITHUB;

/// The core batteries, one card each.
#[component]
pub fn Batteries() -> Element {
    rsx! {
        section { "aria-labelledby": "batteries-title",
            Flex { direction: "column", gap: "lg",
                SectionTitle { id: "batteries-title", "Batteries included" }
                Box {
                    component: "ul",
                    // Safari drops the list role with `list-style: none`.
                    "role": "list",
                    sx: sx()
                        .display("flex")
                        .flex_wrap("wrap")
                        .gap("md")
                        .list_style("none")
                        .margin("0")
                        .padding("0"),
                    Battery { title: "Rich developer experience", icon: rsx! { Pictogram { icon: lucide::code::outlined } },
                        Checks {
                            ListItem {
                                "Typed "
                                Code { source: "sx" }
                                " builder for styling"
                            }
                            ListItem { "Modern JSX-like dx with Rust semantics" }
                        }
                    }
                    Battery { title: "Documentation", icon: rsx! { Pictogram { icon: lucide::book_open::outlined } },
                        Checks {
                            ListItem { "Rich documentation with live demos" }
                            ListItem { "LLM optimized view" }
                        }
                    }
                    Battery { title: "Custom themes", icon: rsx! { Pictogram { icon: lucide::palette::outlined } },
                        Checks {
                            ListItem { "20+ ready-made theme sets" }
                            ListItem { "Adjust them or make your own" }
                        }
                    }
                    Battery { title: "More to come", icon: rsx! { Pictogram { icon: lucide::sparkles::outlined } },
                        Text { "We are warming up for our first season, will you join our team?" }
                        Anchor { to: GITHUB, target: "_blank", "Join us on GitHub" }
                    }
                }
            }
        }
    }
}

/// A card: a tinted icon, a bold blue title, then the body.
#[component]
fn Battery(title: &'static str, icon: Element, children: Element) -> Element {
    rsx! {
        Paper {
            component: "li",
            bordered: true,
            shadow: "xs",
            sx: sx()
                .flex("1 1 100%")
                .min_width("0")
                .padding("lg")
                .breakpoint(Size::Sm, sx().flex("1 1 calc(50% - 8px)")),
            Flex { direction: "column", gap: "md",
                Flex { direction: "row", gap: "md", align: "center",
                    Icon { variant: "gradient", size: "lg", radius: "md", {icon} }
                    Title {
                        size: "md",
                        component: "h3",
                        sx: sx().color("primary.7").font_weight("800"),
                        "{title}"
                    }
                }
                {children}
            }
        }
    }
}

/// A list whose bullets are checkmarks.
#[component]
fn Checks(children: Element) -> Element {
    rsx! {
        List {
            size: "sm",
            icon: rsx! {
                Icon { variant: "standard", color: "primary", size: "xs", Pictogram { icon: lucide::check::outlined } }
            },
            {children}
        }
    }
}
