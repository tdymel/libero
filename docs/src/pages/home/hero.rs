use dioxus::prelude::*;
use libero::{
    components::{Badge, Button, CodeBlock, Flex, Text, Title},
    hooks::use_localization,
    sx::sx,
    theme::Size,
};

use super::booking::BookingCard;
use crate::{GITHUB, Route};

/// The pitch, the way in, and a live card, so something is there to touch
/// on the first screen.
#[component]
pub fn Hero() -> Element {
    let localization = use_localization();

    rsx! {
        Flex {
            direction: "column",
            gap: "xl",
            sx: sx().breakpoint(Size::Md, sx().flex_direction("row").align_items("flex-start")),
            Flex { direction: "column", gap: "lg", sx: sx().min_width("0").breakpoint(Size::Md, sx().flex("1 1 0")),
                Badge { variant: "outlined", color: "muted", "Pre-release" }
                Title { size: "xxl", component: "h1",
                    "Accessible, themeable components for Dioxus"
                }
                Text { size: "lg",
                    "Libero is a component library written in Rust. You style it with a typed "
                    "builder, theme it with a struct, and run the same code in the browser and "
                    "natively through Blitz."
                }
                Flex { direction: "row", gap: "md", wrap: "wrap",
                    Button { to: Route::GettingStarted {}, size: "lg", "Get started" }
                    Button { to: Route::BoxPage {}, size: "lg", variant: "outlined", "Browse components" }
                    Button {
                        to: GITHUB,
                        target: "_blank",
                        size: "lg",
                        variant: "standard",
                        aria_label: format!("GitHub {}", localization.anchor.new_tab),
                        "GitHub"
                    }
                }
                CodeBlock {
                    source: "cargo add libero",
                    language: "shell",
                    header: false,
                    line_numbers: false,
                }
            }
            Flex { direction: "column", gap: "sm", align: "center",
                BookingCard {}
                Text { size: "sm", color: "muted.7",
                    "Every part of this card is a libero component. Try it with the keyboard."
                }
            }
        }
    }
}
