use dioxus::prelude::*;
use libero::{
    components::{Box, Flex, Paper, Text},
    sx::sx,
    theme::{Size, ThemeSet},
};

use crate::exports::COMPONENTS;

/// The library in four numbers, each computed where it can be.
#[component]
pub fn Stats() -> Element {
    let components = COMPONENTS.len().to_string();
    let palettes = ThemeSet::CATALOGUE.len().to_string();

    rsx! {
        section { "aria-label": "Libero in numbers",
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
                Stat { value: components, label: "components, from Button to DatePicker" }
                Stat { value: palettes, label: "palettes, each light and dark" }
                Stat { value: "2", label: "targets from one codebase: web and native" }
                Stat { value: "AA", label: "WCAG 2.2, the accessibility target" }
            }
        }
    }
}

#[component]
fn Stat(value: String, label: &'static str) -> Element {
    rsx! {
        Paper {
            component: "li",
            bordered: true,
            shadow: "xs",
            sx: sx()
                .flex("1 1 calc(50% - 8px)")
                .min_width("0")
                .padding("lg")
                .breakpoint(Size::Md, sx().flex("1 1 0")),
            Flex { direction: "column", gap: "xs",
                Text {
                    component: "span",
                    sx: sx()
                        .font_size("2.25rem")
                        .font_weight("800")
                        .line_height("1")
                        .color("primary.7"),
                    "{value}"
                }
                Text { component: "span", size: "sm", "{label}" }
            }
        }
    }
}
