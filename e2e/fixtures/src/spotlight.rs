//! `Spotlight`, for the overlay archetype.

use dioxus::prelude::*;
use libero::components::{
    Button, Flex, SpotlightAction, SpotlightOptions, Text, spotlight_filter, use_spotlight,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/spotlight", || rsx! { SpotlightPage {} }),
    ("/spotlight-long", || rsx! { SpotlightLongPage {} }),
    ("/spotlight-tall", || rsx! { SpotlightTallPage {} }),
    ("/spotlight-fetch", || rsx! { SpotlightFetchPage {} }),
    ("/spotlight-options", || rsx! { SpotlightOptionsPage {} }),
];

/// Four actions, two drawn, none highlighted by typing (todo 2387).
#[component]
fn SpotlightOptionsPage() -> Element {
    let mut ran = use_signal(String::new);
    let all = use_hook(|| {
        ["Alpha", "Beta", "Gamma", "Delta"]
            .map(|label| SpotlightAction::new(label).onclick(move |_| ran.set(label.into())))
            .to_vec()
    });
    let spotlight = use_spotlight(SpotlightOptions {
        actions: Some(Callback::new(move |query: String| {
            spotlight_filter(&query, &all)
        })),
        aria_label: Some("Command palette".into()),
        limit: Some(2),
        highlight_first_on_query: false,
        ..Default::default()
    });

    rsx! {
        Button { id: "open-spotlight", onclick: move |_| spotlight.open(), "Open the palette" }
        Text { id: "ran", "data-ran": "{ran}", size: "sm", "Ran" }
    }
}

/// A server-side search: the answer for one or two letters is still on its way (todo 1810).
#[component]
fn SpotlightFetchPage() -> Element {
    let mut query = use_signal(String::new);
    let mut ran = use_signal(String::new);
    let all = use_hook(|| {
        ["Home", "Changelog"]
            .map(|label| SpotlightAction::new(label).onclick(move |_| ran.set(label.into())))
            .to_vec()
    });
    let actions = use_callback(move |query: String| spotlight_filter(&query, &all));
    let onquery = use_callback(move |next: String| query.set(next));
    let typed = query().trim().chars().count();
    let spotlight = use_spotlight(SpotlightOptions {
        actions: Some(actions),
        onquery: Some(onquery),
        loading: (1..3).contains(&typed),
        aria_label: Some("Command palette".into()),
        ..Default::default()
    });

    rsx! {
        Button { id: "open-spotlight", onclick: move |_| spotlight.open(), "Open the palette" }
        Text { id: "ran", "data-ran": "{ran}", size: "sm", "Ran" }
    }
}

/// Many described actions: the palette must not outgrow the viewport (todo 1307).
#[component]
fn SpotlightTallPage() -> Element {
    let all = use_hook(|| {
        (0..60)
            .map(|i| {
                SpotlightAction::new(format!("Action {i}"))
                    .group(format!("Group {}", i / 6))
                    .description("A description long enough to wrap onto a second line on a phone")
            })
            .collect::<Vec<_>>()
    });
    let spotlight = use_spotlight(SpotlightOptions {
        actions: Some(Callback::new(move |query: String| {
            spotlight_filter(&query, &all)
        })),
        aria_label: Some("Command palette".into()),
        limit: Some(60),
        ..Default::default()
    });

    rsx! {
        Button { id: "open-spotlight", onclick: move |_| spotlight.open(), "Open the palette" }
    }
}

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

/// The hook lives on the page, which outlives the trigger. Actions write into `#ran`'s
/// `data-ran`, leaving the resting accessibility tree unchanged.
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
        // Unnamed, every open logs the missing-name warning and the console pass fails.
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
