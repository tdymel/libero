use crate::Route;
use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Button, Code, CodeBlock, Flex, Text, Title},
    hooks::{DrawerOptions, ModalScope, use_drawer},
};

const FILTERS: &str = r#"#[component]
fn FilterPanel() -> Element {
    let filters = use_drawer(
        DrawerOptions {
            anchor: "end".into(),
            size: "sm".into(),
            aria_label: Some("Filters".into()),
            ..Default::default()
        },
        |s: ModalScope<()>| rsx! {
            Title { size: "lg", "Filters" }
            Text { "Nothing to filter yet." }
            Button { variant: "text", onclick: move |_| s.close(), "Close" }
        },
    );

    rsx! {
        Button { variant: "outlined", onclick: move |_| { filters.open(); }, "Filters" }
    }
}"#;

/// `FILTERS`, rendered.
#[component]
fn FilterPanel() -> Element {
    let filters = use_drawer(
        DrawerOptions {
            anchor: "end".into(),
            size: "sm".into(),
            aria_label: Some("Filters".into()),
            ..Default::default()
        },
        |s: ModalScope<()>| {
            rsx! {
                Title { size: "lg", "Filters" }
                Text { "Nothing to filter yet." }
                Button { variant: "text", onclick: move |_| s.close(), "Close" }
            }
        },
    );

    rsx! {
        Button {
            variant: "outlined",
            onclick: move |_| {
                filters.open();
            },
            "Filters"
        }
    }
}

#[component]
pub fn UseDrawerPage() -> Element {
    rsx! {
        DocPage {
            title: "use_drawer",
            source: "libero/src/components/overlay/use_drawer.rs",
            markdown: "/md/use_drawer.md",
            lead: rsx! {
                Text {
                    Code { source: "use_drawer(options, render) -> ModalHandle<S, R>" }
                    " registers a panel docked to one edge and returns the handle that opens "
                    "it. It is "
                    Anchor { to: Route::UseModalPage {}, "use_modal" }
                    " with the docking around it, so it has the same handle, arguments and "
                    "results. "
                    Anchor { to: Route::DrawerPage {}, "Drawer" }
                    " has the full story."
                }
            },

            DocSection {
                title: "Usage",
                Flex { align: "flex-start", FilterPanel {} }
                CodeBlock { source: FILTERS, language: "rust" }
            }

            DocSection {
                title: "Accessibility",
                Text {
                    "The panel is a dialog with no name of its own, so set "
                    Code { source: "aria_label" }
                    ". It has no header close button either, so the content brings its own. "
                    "Like every modal it traps focus, closes on Escape and hands focus back "
                    "to its trigger."
                }
            }
        }
    }
}
