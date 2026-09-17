use crate::Route;
use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Box, Button, Chip, Code, CodeBlock, Flex, Scroller, Text, use_scroller},
    sx::sx,
};

const TAGS: &str = r#"const NAMES: [&str; 6] = ["Rust", "Dioxus", "WebAssembly", "Accessibility", "Layout", "Theming"];

#[component]
fn Tags() -> Element {
    let strip = use_scroller();

    rsx! {
        Box { sx: sx().max_width("16rem"),
            Scroller { aria_label: "Tags", controls: "never", handle: strip,
                Flex { direction: "row", gap: "sm", wrap: "nowrap",
                    for name in NAMES {
                        Chip { key: "{name}", "{name}" }
                    }
                }
            }
        }
        Flex { direction: "row", gap: "sm",
            Button { variant: "outlined", onclick: move |_| strip.step_back(), "Back" }
            Button { variant: "outlined", onclick: move |_| strip.step_forward(), "Forward" }
        }
    }
}"#;

const NAMES: [&str; 6] = [
    "Rust",
    "Dioxus",
    "WebAssembly",
    "Accessibility",
    "Layout",
    "Theming",
];

/// `TAGS`, rendered.
#[component]
fn Tags() -> Element {
    let strip = use_scroller();

    rsx! {
        Box { sx: sx().max_width("16rem"),
            Scroller { aria_label: "Tags", controls: "never", handle: strip,
                Flex { direction: "row", gap: "sm", wrap: "nowrap",
                    for name in NAMES {
                        Chip { key: "{name}", "{name}" }
                    }
                }
            }
        }
        Flex { direction: "row", gap: "sm",
            Button { variant: "outlined", onclick: move |_| strip.step_back(), "Back" }
            Button { variant: "outlined", onclick: move |_| strip.step_forward(), "Forward" }
        }
    }
}

#[component]
pub fn UseScrollerPage() -> Element {
    rsx! {
        DocPage {
            title: "use_scroller",
            source: "libero/src/components/layout/scroller.rs",
            markdown: "/md/use_scroller.md",
            lead: rsx! {
                Text {
                    Code { source: "use_scroller() -> ScrollerHandle" }
                    " steps a "
                    Code { source: "Scroller" }
                    " from controls of your own, by the same amount its built-in arrows "
                    "would. Pass it as the scroller's "
                    Code { source: "handle" }
                    " from the first render. "
                    Anchor { to: Route::ScrollerPage {}, "Scroller" }
                    " documents the component."
                }
            },

            DocSection {
                title: "Usage",
                Flex { direction: "column", gap: "sm", Tags {} }
                CodeBlock { source: TAGS, language: "rust" }
                Text {
                    "A step starts from where the strip is now, so a touch scroll in between "
                    "is kept. A call before the scroller has mounted does nothing."
                }
            }
        }
    }
}
