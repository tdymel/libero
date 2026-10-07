use dioxus::prelude::*;
use libero::{
    components::{Anchor, Button, CodeBlock, Divider, Flex, Paper, Text, Title},
    sx::sx,
    theme::{ColorCss, ColorShade},
};

use super::{CtaStack, card_background, cta_row_sx, forced_colors_edge};
use crate::{Route, site::GITHUB};

/// The way in once more, and the page's footer line.
#[component]
pub fn Closing() -> Element {
    rsx! {
        section { "aria-labelledby": "closing-title",
            Flex { direction: "column", gap: "lg", align: "center",
                Paper {
                    radius: "xl",
                    shadow: "xs",
                    sx: sx()
                        .background(card_background())
                        .width("100%")
                        .padding("48px 24px")
                        .text_align("center")
                        .and(forced_colors_edge()),
                    Flex { direction: "column", gap: "lg", align: "center",
                        Title {
                            size: "xxl",
                            component: "h2",
                            id: "closing-title",
                            color: ColorCss::PRIMARY.role_value("text-", ColorShade::S6),
                            "The ball is in your hands"
                        }
                        CtaStack {
                            Flex { direction: "row", gap: "md", wrap: "wrap", justify: "center", sx: cta_row_sx(),
                                Button { to: Route::GettingStarted {}, size: "lg", "Get started" }
                                Button { to: Route::BoxPage {}, size: "lg", variant: "outlined", color: "primary.9", "Browse components" }
                            }
                            CodeBlock {
                                // copy: install
                                source: "cargo add libero --git https://github.com/tdymel/libero",
                                language: "shell",
                                label: "Add libero to your project",
                                header: false,
                                line_numbers: false,
                                // copy: end
                            }
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
