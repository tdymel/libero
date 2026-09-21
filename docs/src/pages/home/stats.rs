use dioxus::prelude::*;
use libero::{
    components::{Box, Flex, Paper, Text},
    sx::sx,
    theme::Size,
};

/// The library in four numbers, each a floor the tests below hold.
#[component]
pub fn Stats() -> Element {
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
                Stat { value: "100+", title: "Components and Hooks" }
                Stat { value: "20+", title: "ThemeSets" }
                Stat { value: "4", title: "Platform Targets" }
                Stat { value: "AA", title: "WCAG 2.2" }
            }
        }
    }
}

#[component]
fn Stat(value: &'static str, title: &'static str) -> Element {
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
                Text { component: "span", sx: sx().font_weight("700"), "{title}" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::exports::COMPONENTS;
    use libero::theme::ThemeSet;

    #[test]
    fn the_cards_claims_hold() {
        assert!(COMPONENTS.len() >= 100, "{} components", COMPONENTS.len());
        assert!(
            ThemeSet::CATALOGUE.len() >= 20,
            "{} themes",
            ThemeSet::CATALOGUE.len()
        );
    }
}
