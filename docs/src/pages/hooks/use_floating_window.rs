use crate::Route;
use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Button, Code, CodeBlock, Flex, FloatingWindowOptions, Kbd, Text},
    hooks::use_floating_window,
};

const NOTES: &str = r#"#[component]
fn Notes() -> Element {
    let notes = use_floating_window(
        FloatingWindowOptions {
            title: Some("Notes".into()),
            placement: "bottom-end".into(),
            ..Default::default()
        },
        |window| rsx! {
            Text { "Drag the title bar, or focus it and use the arrow keys." }
            Button { onclick: move |_| window.close(), "Done" }
        },
    );

    rsx! {
        Button { variant: "outlined", onclick: move |_| notes.toggle(), "Notes" }
    }
}"#;

/// `NOTES`, rendered.
#[component]
fn Notes() -> Element {
    let notes = use_floating_window(
        FloatingWindowOptions {
            title: Some("Notes".into()),
            placement: "bottom-end".into(),
            ..Default::default()
        },
        |window| {
            rsx! {
                Text { "Drag the title bar, or focus it and use the arrow keys." }
                Button { onclick: move |_| window.close(), "Done" }
            }
        },
    );

    rsx! {
        Button { variant: "outlined", onclick: move |_| notes.toggle(), "Notes" }
    }
}

#[component]
pub fn UseFloatingWindowPage() -> Element {
    rsx! {
        DocPage {
            title: "use_floating_window",
            source: "libero/src/components/overlay/use_floating_window.rs",
            markdown: "/md/use_floating_window.md",
            lead: rsx! {
                Text {
                    Code { source: "use_floating_window(options, render) -> FloatingWindowHandle" }
                    " registers a movable window over the page and returns the handle that "
                    "opens it. The window is not modal, so the page stays usable. "
                    Anchor { to: Route::FloatingWindowPage {}, "FloatingWindow" }
                    " covers resizing, placement and the window menu."
                }
            },

            DocSection {
                title: "Usage",
                Flex { align: "flex-start", Notes {} }
                CodeBlock { source: NOTES, language: "rust" }
            }

            DocSection {
                title: "Accessibility",
                Text {
                    "The "
                    Code { source: "title" }
                    " names the window. It takes focus on open, and Escape, its close "
                    "button or "
                    Code { source: "close()" }
                    " hand focus back to the trigger. "
                    Kbd { "F6" }
                    " moves between the page and the window."
                }
            }
        }
    }
}
