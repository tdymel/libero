//! `use_tour`, for the overlay archetype: three targets, and a step whose target never mounts.

use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Text, TourOptions, TourStep, use_tour},
    hooks::use_element,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/tour", || rsx! { TourPage {} }),
    ("/tour/missing", || rsx! { MissingTourPage {} }),
];

/// `#ended` reads how the tour last ended: `finished`, or `closed at <index>`.
#[component]
fn TourPage() -> Element {
    let first = use_element();
    let second = use_element();
    let third = use_element();
    let mut ended = use_signal(String::new);
    let finished = use_callback(move |()| ended.set("finished".into()));
    let closed = use_callback(move |index: usize| ended.set(format!("closed at {index}")));
    let tour = use_tour(TourOptions {
        steps: vec![
            TourStep::new("first")
                .target(first)
                .title("First")
                .description("The first stop."),
            TourStep::new("second")
                .target(second)
                .title("Second")
                .description("The second stop."),
            TourStep::new("third")
                .target(third)
                .title("Third")
                .description("The last stop."),
        ],
        onfinish: Some(finished),
        onclose: Some(closed),
        ..Default::default()
    });

    rsx! {
        Flex { direction: "column", gap: "xl", max_width: "320px",
            Button { id: "start-tour", variant: "outlined", onclick: move |_| tour.start(), "Take the tour" }
            Button { id: "first", onmounted: first.mount(), attributes: first.attributes(), "First" }
            Button { id: "second", onmounted: second.mount(), attributes: second.attributes(), "Second" }
            Button { id: "third", onmounted: third.mount(), attributes: third.attributes(), "Third" }
            Text { id: "ended", size: "sm", "{ended}" }
        }
    }
}

/// The step's target is never mounted, so its card waits in the middle.
#[component]
fn MissingTourPage() -> Element {
    let nowhere = use_element();
    let tour = use_tour(TourOptions {
        steps: vec![TourStep::new("nowhere").target(nowhere).title("Nowhere")],
        ..Default::default()
    });

    rsx! {
        Button { id: "start-tour", variant: "outlined", onclick: move |_| tour.start(), "Take the tour" }
    }
}
