use crate::Route;
use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{
    Anchor, Button, Code, CodeBlock, Flex, SpotlightAction, SpotlightOptions, Text,
    spotlight_filter, use_spotlight,
};

const PALETTE: &str = r#"#[component]
fn Palette() -> Element {
    let mut last = use_signal(|| String::from("nothing yet"));
    let all = use_hook(|| {
        let run = move |name: &'static str| move |_| last.set(name.to_string());
        vec![
            SpotlightAction::new("New file").onclick(run("New file")),
            SpotlightAction::new("Open recent").onclick(run("Open recent")),
            SpotlightAction::new("Settings").onclick(run("Settings")),
        ]
    });
    let spotlight = use_spotlight(SpotlightOptions {
        actions: Some(Callback::new(move |query: String| spotlight_filter(&query, &all))),
        aria_label: Some("Commands".into()),
        // No hotkey: this page's own search owns Ctrl K.
        shortcut: None,
        ..Default::default()
    });

    rsx! {
        Flex { direction: "row", gap: "md",
            Button { variant: "outlined", onclick: move |_| spotlight.open(), "Commands" }
            Text { "Last run: {last}" }
        }
    }
}"#;

/// `PALETTE`, rendered.
#[component]
fn Palette() -> Element {
    let mut last = use_signal(|| String::from("nothing yet"));
    let all = use_hook(|| {
        let run = move |name: &'static str| move |_| last.set(name.to_string());
        vec![
            SpotlightAction::new("New file").onclick(run("New file")),
            SpotlightAction::new("Open recent").onclick(run("Open recent")),
            SpotlightAction::new("Settings").onclick(run("Settings")),
        ]
    });
    let spotlight = use_spotlight(SpotlightOptions {
        actions: Some(Callback::new(move |query: String| {
            spotlight_filter(&query, &all)
        })),
        aria_label: Some("Commands".into()),
        // No hotkey: this page's own search owns Ctrl K.
        shortcut: None,
        ..Default::default()
    });

    rsx! {
        Flex { direction: "row", gap: "md",
            Button { variant: "outlined", onclick: move |_| spotlight.open(), "Commands" }
            Text { "Last run: {last}" }
        }
    }
}

#[component]
pub fn UseSpotlightPage() -> Element {
    rsx! {
        DocPage {
            title: "use_spotlight",
            source: "libero/src/components/overlay/spotlight/spotlight.rs",
            markdown: "/md/use_spotlight.md",
            lead: rsx! {
                Text {
                    Code { source: "use_spotlight(options) -> SpotlightHandle" }
                    " registers a command palette and returns the handle that opens it. "
                    "By default Ctrl K, or Cmd K on a Mac, toggles it from anywhere on the "
                    "page. "
                    Anchor { to: Route::SpotlightPage {}, "Spotlight" }
                    " covers actions, groups, search and loading."
                }
            },

            DocSection {
                title: "Usage",
                Palette {}
                CodeBlock { source: PALETTE, language: "rust" }
                Text {
                    Code { source: "actions" }
                    " is called with the live query and returns what to show. "
                    Code { source: "spotlight_filter" }
                    " is the common matcher. Open it from the handler of what the user "
                    "acted on, so focus goes back there on close."
                }
            }

            DocSection {
                title: "Web and native",
                Text {
                    "The hotkey is web only. No other renderer has a document-level key "
                    "listener yet, so give the palette a visible trigger as well."
                }
            }
        }
    }
}
