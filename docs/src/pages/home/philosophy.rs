use dioxus::prelude::*;
use libero::{
    components::{Anchor, Box, Flex, Paper, Text, Title},
    sx::sx,
    theme::Size,
};

use super::tint;
use crate::Route;

const PRINCIPLES: [&str; 4] = [
    "Developer experience",
    "Accessibility",
    "Batteries included",
    "Simple yet modern",
];

/// The four principles by name only; the Philosophy page explains them.
#[component]
pub fn Philosophy() -> Element {
    rsx! {
        Paper {
            component: "section",
            "aria-labelledby": "philosophy-title",
            radius: "xl",
            sx: sx()
                .padding("xl")
                .box_shadow("none")
                .background(tint(8))
                .border(format!("1px solid {}", tint(20))),
            Flex { direction: "column", gap: "lg",
                Flex { direction: "column", gap: "sm",
                    Title { size: "lg", component: "h2", id: "philosophy-title", "What decides what goes in" }
                    Text { "Four principles, ranked: when two pull apart, the higher one wins." }
                }
                Box {
                    component: "ol",
                    sx: sx()
                        .display("flex")
                        .flex_direction("column")
                        .gap("md")
                        .list_style("none")
                        .margin("0")
                        .padding("0")
                        .breakpoint(Size::Md, sx().flex_direction("row")),
                    for (at, name) in PRINCIPLES.iter().enumerate() {
                        Box {
                            key: "{name}",
                            component: "li",
                            sx: sx()
                                .display("flex")
                                .align_items("center")
                                .gap("sm")
                                .breakpoint(Size::Md, sx().flex("1 1 0")),
                            Box {
                                component: "span",
                                "aria-hidden": "true",
                                sx: sx()
                                    .display("inline-flex")
                                    .align_items("center")
                                    .justify_content("center")
                                    .flex_shrink("0")
                                    .width("32px")
                                    .height("32px")
                                    .border_radius("50%")
                                    .background("primary")
                                    .color("primary-contrast")
                                    .font_weight("700"),
                                "{at + 1}"
                            }
                            Text { component: "span", sx: sx().font_weight("600"), "{name}" }
                        }
                    }
                }
                Anchor { to: Route::PhilosophyPage {}, "Read the philosophy" }
            }
        }
    }
}
