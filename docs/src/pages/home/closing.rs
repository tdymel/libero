use dioxus::prelude::*;
use libero::{
    components::{Anchor, Button, CodeBlock, Divider, Flex, Paper, Text, Title},
    sx::sx,
    theme::{Gradient, NamedColorCss},
};

use super::cta_row_sx;
use crate::{Route, site::GITHUB};

/// The way in once more, and the page's footer line.
#[component]
pub fn Closing() -> Element {
    rsx! {
        section { "aria-labelledby": "closing-title",
            Flex { direction: "column", gap: "lg", align: "center",
                Paper {
                    gradient: Gradient::default(),
                    radius: "xl",
                    sx: sx().width("100%").padding("48px 24px").text_align("center"),
                    Flex { direction: "column", gap: "lg", align: "center",
                        Title { size: "xxl", component: "h2", id: "closing-title", "The ball is in your hands" }
                        Flex { direction: "row", gap: "md", wrap: "wrap", justify: "center", sx: cta_row_sx(),
                            Button { to: Route::GettingStarted {}, size: "lg", color: "surface", "Get started" }
                            Button { to: Route::BoxPage {}, size: "lg", variant: "outlined", color: "currentColor", "Browse components" }
                        }
                        CodeBlock {
                            // copy: install
                            source: "cargo add libero",
                            language: "shell",
                            label: "Add libero to your project",
                            header: false,
                            line_numbers: false,
                            // copy: end
                            // The block keeps its own surface, so not the gradient's label colour.
                            sx: sx()
                                .width("100%")
                                .max_width("320px")
                                .text_align("start")
                                .color(NamedColorCss::INK.value()),
                        }
                    }
                }
            }
        }
        footer {
            Flex { direction: "column", gap: "lg",
                Divider {}
                Text { size: "sm", color: "muted.7",
                    "MIT or Apache-2.0. Source on "
                    Anchor { to: GITHUB, target: "_blank", "GitHub" }
                    ". Built with "
                    Anchor { to: "https://dioxuslabs.com", target: "_blank", "Dioxus" }
                    "."
                }
            }
        }
    }
}
