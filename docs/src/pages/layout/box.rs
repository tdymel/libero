use dioxus::prelude::*;
use libero::{
    components::{Box, Code, Flex, Text, Title},
    sx::sx,
};

#[component]
pub fn BoxPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "xxl",
            Flex {
                direction: "column",
                gap: "lg",
                Title { size: "xxl", "Box" }
                Text {
                    "The polymorphic primitive every other component is built on - renders "
                    "as any tag via "
                    Code { "component" }
                    ", plus "
                    Code { "sx" }
                    "/"
                    Code { "states" }
                    " styling and escape-hatch attributes like "
                    Code { "href" }
                    "/"
                    Code { "src" }
                    "."
                }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Example" }
                Box {
                    component: "section",
                    sx: sx().padding("16px").background("grey.1").border_radius("md"),
                    Text { "Rendered as a section, styled entirely via sx." }
                }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "As a link" }
                Box {
                    component: "a",
                    href: "https://dioxuslabs.com",
                    target: "_blank",
                    sx: sx().color("primary.6").font_weight("600"),
                    "Open Dioxus docs"
                }
            }
        }
    }
}
