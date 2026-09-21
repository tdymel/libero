use dioxus::prelude::*;
use libero::{
    components::{Anchor, Button, CodeBlock, Divider, Flex, Text, Title},
    sx::sx,
    theme::{ColorCss, ColorShade},
};

use super::tint;
use crate::{GITHUB, Route};

/// The way in once more, and the page's footer line.
#[component]
pub fn Closing() -> Element {
    rsx! {
        section { "aria-labelledby": "closing-title",
            Flex {
                direction: "column",
                gap: "lg",
                align: "center",
                sx: sx()
                    .padding("48px 24px")
                    .border_radius("xl")
                    .text_align("center")
                    .background(format!(
                        "linear-gradient(135deg, {} 0%, {} 100%)",
                        tint(22),
                        tint(6),
                    )),
                Title {
                    size: "xxl",
                    component: "h2",
                    id: "closing-title",
                    span { color: ColorCss::PRIMARY.role_value("text-", ColorShade::S6), "The ball is in your hands" }
                }
                CodeBlock {
                    source: "cargo add libero",
                    language: "shell",
                    header: false,
                    line_numbers: false,
                    sx: sx().width("100%").max_width("320px").text_align("start"),
                }
                Flex { direction: "row", gap: "md", wrap: "wrap", justify: "center",
                    Button { to: Route::GettingStarted {}, size: "lg", "Get started" }
                    // Text role 8: role 6 misses 4.5:1 on the tint (899).
                    Button { to: Route::BoxPage {}, size: "lg", variant: "outlined", color: "primary.8", "Browse components" }
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
