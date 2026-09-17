use crate::Route;
use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Code, CodeBlock, Text},
    use_theme,
};

const SPACING: &str = r#"#[component]
fn PopoverSpacing() -> Element {
    let theme = use_theme();

    rsx! {
        Text {
            "Popovers sit {theme.popover.gap}px from their anchor, "
            "with {theme.popover.padding}px of padding."
        }
    }
}"#;

/// `SPACING`, rendered.
#[component]
fn PopoverSpacing() -> Element {
    let theme = use_theme();

    rsx! {
        Text {
            "Popovers sit {theme.popover.gap}px from their anchor, "
            "with {theme.popover.padding}px of padding."
        }
    }
}

#[component]
pub fn UseThemePage() -> Element {
    rsx! {
        DocPage {
            title: "use_theme",
            source: "libero/src/hooks/theme.rs",
            markdown: "/md/use_theme.md",
            lead: rsx! {
                Text {
                    Code { source: "use_theme() -> &'static Theme" }
                    " returns the active theme. Read it for a value CSS cannot carry, such as "
                    "a number a hook computes with. The component re-renders when the theme "
                    "changes. "
                    Anchor { to: Route::ThemingPage {}, "Theming" }
                    " explains the theme itself."
                }
            },

            DocSection {
                title: "Usage",
                PopoverSpacing {}
                CodeBlock { source: SPACING, language: "rust" }
                Text {
                    "For a colour or a size in your own styles, reach for "
                    Code { source: "sx" }
                    " and the theme's CSS variables instead. They follow a scheme switch "
                    "without a re-render."
                }
            }
        }
    }
}
