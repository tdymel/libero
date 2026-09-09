//! `Spotlight`, for the overlay archetype.

use dioxus::prelude::*;
use libero::components::{
    Button, Flex, SpotlightAction, SpotlightOptions, Text, spotlight_filter, use_spotlight,
};

use crate::Routes;

pub const ROUTES: Routes = &[("/spotlight", || rsx! { SpotlightPage {} })];

/// The hook lives on the page, which outlives the trigger, as its docs ask.
#[component]
fn SpotlightPage() -> Element {
    let all = use_hook(|| {
        vec![
            SpotlightAction::new("Home")
                .group("Pages")
                .description("The start page"),
            SpotlightAction::new("Changelog").group("Pages"),
            SpotlightAction::new("New file")
                .group("Commands")
                .shortcut("Ctrl N"),
        ]
    });
    let spotlight = use_spotlight(SpotlightOptions {
        actions: Some(Callback::new(move |query: String| {
            spotlight_filter(&query, &all)
        })),
        // Named, or every open logs the missing-name warning and the console
        // pass fails.
        // The theme's own name, so the fixture reads as a caller doing it
        // right rather than renaming the component.
        aria_label: Some("Command palette".into()),
        ..Default::default()
    });

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button {
                id: "open-spotlight",
                variant: "outlined",
                onclick: move |_| spotlight.open(),
                "Open the palette"
            }
            Text { size: "sm", "Or press Ctrl K." }
        }
    }
}
