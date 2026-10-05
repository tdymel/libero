//! `use_tour`, for the overlay archetype: three targets, a step whose target never mounts,
//! one whose target is not rendered, and long steps for a short viewport.

use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Text, TourOptions, TourStep, use_tour},
    hooks::use_element,
    sx::sx,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/tour", || rsx! { TourPage {} }),
    ("/tour/missing", || rsx! { MissingTourPage {} }),
    ("/tour/nested", || rsx! { NestedTourPage {} }),
    ("/tour/hidden", || rsx! { HiddenTourPage {} }),
    ("/tour/long", || rsx! { LongTourPage {} }),
];

const LONG: &str = "This stop has a lot to say. It explains the feature in several sentences, so \
    the card grows taller than a short viewport, as at 400 percent zoom or on a phone held \
    sideways. The card must scroll instead of losing its buttons off the screen.";


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
        // Off the window edge: natively the page has no margin, and a hole on x 0 is clipped.
        Flex { direction: "column", gap: "xl", max_width: "320px", sx: sx().padding("16px"),
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

/// `#deep` sits below a vertical scroller's fold and past a horizontal one's edge.
#[component]
fn NestedTourPage() -> Element {
    let deep = use_element();
    let tour = use_tour(TourOptions {
        steps: vec![TourStep::new("deep").target(deep).title("Deep")],
        ..Default::default()
    });

    rsx! {
        Button { id: "start-tour", variant: "outlined", onclick: move |_| tour.start(), "Take the tour" }
        div { id: "outer", style: "height: 200px; width: 300px; overflow-y: auto;",
            div { style: "height: 600px;" }
            div { id: "inner", style: "overflow-x: auto; white-space: nowrap;",
                div { style: "display: inline-block; width: 900px; height: 1px;" }
                Button { id: "deep", onmounted: deep.mount(), attributes: deep.attributes(), "Deep" }
            }
            div { style: "height: 600px;" }
        }
    }
}

/// The step's target is mounted under `display: none`, so its card waits in the middle too.
#[component]
fn HiddenTourPage() -> Element {
    let hidden = use_element();
    let tour = use_tour(TourOptions {
        steps: vec![TourStep::new("hidden").target(hidden).title("Hidden")],
        ..Default::default()
    });

    rsx! {
        Button { id: "start-tour", variant: "outlined", onclick: move |_| tour.start(), "Take the tour" }
        div { display: "none",
            Button { id: "hidden", onmounted: hidden.mount(), attributes: hidden.attributes(), "Hidden" }
        }
    }
}

/// A centred step and a placed one, each taller than a 320x256 viewport.
#[component]
fn LongTourPage() -> Element {
    let target = use_element();
    let tour = use_tour(TourOptions {
        steps: vec![
            TourStep::new("centred").title("Centred").description(LONG),
            TourStep::new("placed").target(target).title("Placed").description(LONG),
        ],
        ..Default::default()
    });

    rsx! {
        Flex { direction: "column", gap: "md", sx: sx().padding("16px"),
            Button { id: "start-tour", variant: "outlined", onclick: move |_| tour.start(), "Take the tour" }
            Button { id: "target", onmounted: target.mount(), attributes: target.attributes(), "Target" }
        }
    }
}
