use crate::Route;
use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Box, Button, Code, CodeBlock, Flex, ScrollArea, Text, use_scroll_area},
    sx::sx,
};

const LOG: &str = r#"#[component]
fn Log() -> Element {
    let area = use_scroll_area();

    rsx! {
        Box { sx: sx().height("120px").border("1px solid var(--lsx-muted-3)"),
            ScrollArea { aria_label: "Log", handle: area,
                for i in 1..=20 {
                    Text { key: "{i}", "Entry {i}" }
                }
            }
        }
        Flex { direction: "row", gap: "sm",
            Button { variant: "outlined", onclick: move |_| area.scroll_to_percent(None, Some(0.0)), "Top" }
            Button { variant: "outlined", onclick: move |_| area.scroll_to_percent(None, Some(100.0)), "Bottom" }
        }
    }
}"#;

/// `LOG`, rendered.
#[component]
fn Log() -> Element {
    let area = use_scroll_area();

    rsx! {
        Box { sx: sx().height("120px").border("1px solid var(--lsx-muted-3)"),
            ScrollArea { aria_label: "Log", handle: area,
                for i in 1..=20 {
                    Text { key: "{i}", "Entry {i}" }
                }
            }
        }
        Flex { direction: "row", gap: "sm",
            Button {
                variant: "outlined",
                onclick: move |_| area.scroll_to_percent(None, Some(0.0)),
                "Top"
            }
            Button {
                variant: "outlined",
                onclick: move |_| area.scroll_to_percent(None, Some(100.0)),
                "Bottom"
            }
        }
    }
}

#[component]
pub fn UseScrollAreaPage() -> Element {
    rsx! {
        DocPage {
            title: "use_scroll_area",
            source: "libero/src/components/layout/scroll_area/handle.rs",
            markdown: "/md/use_scroll_area.md",
            lead: rsx! {
                Text {
                    Code { source: "use_scroll_area() -> ScrollAreaHandle" }
                    " scrolls a "
                    Code { source: "ScrollArea" }
                    " from code. Pass it as the area's "
                    Code { source: "handle" }
                    ". "
                    Anchor { to: Route::ScrollAreaPage {}, "ScrollArea" }
                    " documents the component and its scroll events."
                }
            },

            DocSection {
                title: "Usage",
                Flex { direction: "column", gap: "sm", Log {} }
                CodeBlock { source: LOG, language: "rust" }
                Text {
                    Code { source: "scroll_to(x, y)" }
                    " takes pixels from the inline start. "
                    Code { source: "scroll_to_percent(x, y)" }
                    " takes percentages, and "
                    Code { source: "None" }
                    " leaves that axis where it is. Unlike the percent props, every call "
                    "scrolls, even back to a position asked for before."
                }
            }
        }
    }
}
