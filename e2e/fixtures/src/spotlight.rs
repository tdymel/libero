//! `Spotlight`, for the overlay archetype.

use dioxus::prelude::*;
use libero::components::{
    Button, Flex, SpotlightAction, SpotlightOptions, Text, spotlight_filter, use_spotlight,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/spotlight", || rsx! { SpotlightPage {} }),
    ("/spotlight-long", || rsx! { SpotlightLongPage {} }),
];

/// One action whose label has no break opportunity, for reflow at 320px.
#[component]
fn SpotlightLongPage() -> Element {
    let all = use_hook(|| {
        vec![SpotlightAction::new(
            "Versandkostenberechnungsgrundlagenverordnungsentwurfsbearbeitungsstelle",
        )]
    });
    let spotlight = use_spotlight(SpotlightOptions {
        actions: Some(Callback::new(move |query: String| {
            spotlight_filter(&query, &all)
        })),
        aria_label: Some("Command palette".into()),
        ..Default::default()
    });

    rsx! {
        Button { id: "open-spotlight", onclick: move |_| spotlight.open(), "Open the palette" }
    }
}

/// The hook lives on the page, which outlives the trigger, as its docs ask.
/// Each action writes its label into `#ran`'s `data-ran`, so the resting
/// accessibility tree is unchanged.
#[component]
fn SpotlightPage() -> Element {
    let mut ran = use_signal(String::new);
    let all = use_hook(|| {
        vec![
            SpotlightAction::new("Home")
                .group("Pages")
                .description("The start page")
                .onclick(move |_| ran.set("Home".into())),
            SpotlightAction::new("Changelog")
                .group("Pages")
                .onclick(move |_| ran.set("Changelog".into())),
            SpotlightAction::new("New file")
                .group("Commands")
                .shortcut("Ctrl N")
                .onclick(move |_| ran.set("New file".into())),
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
            Text { id: "ran", "data-ran": "{ran}", size: "sm", "Or press Ctrl K." }
        }
    }
}
