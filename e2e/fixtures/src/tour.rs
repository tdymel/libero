//! `use_tour`, for the overlay archetype: three targets, a step whose target never mounts,
//! one whose target is not rendered, and long steps for a short viewport.

use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Select, Text, TourOptions, TourStep, TourView, use_tour},
    hooks::use_element,
    sx::sx,
};

use crate::{Routes, common::Fruit};

pub const ROUTES: Routes = &[
    ("/tour", || rsx! { TourPage { custom: false } }),
    ("/tour/custom", || rsx! { TourPage { custom: true } }),
    ("/tour/missing", || rsx! { MissingTourPage {} }),
    ("/tour/nested", || rsx! { NestedTourPage {} }),
    ("/tour/hidden", || rsx! { HiddenTourPage {} }),
    ("/tour/long", || rsx! { LongTourPage {} }),
    ("/tour/more", || rsx! { MoreTourPage {} }),
    ("/tour/form", || rsx! { FormTourPage {} }),
    ("/tour/tabindex", || rsx! { TabindexTourPage {} }),
];

const LONG: &str = "This stop has a lot to say. It explains the feature in several sentences, so \
    the card grows taller than a short viewport, as at 400 percent zoom or on a phone held \
    sideways. The card must scroll instead of losing its buttons off the screen.";


/// `#ended` reads how the tour last ended: `finished`, or `closed at <index>`. `custom` draws
/// the card itself.
#[component]
fn TourPage(custom: bool) -> Element {
    let first = use_element();
    let second = use_element();
    let third = use_element();
    let mut ended = use_signal(String::new);
    let finished = use_callback(move |()| ended.set("finished".into()));
    let closed = use_callback(move |index: usize| ended.set(format!("closed at {index}")));
    let card = use_callback(|view: TourView| {
        let next = view.clone();
        rsx! {
            Text { "{view.progress}" }
            Button { id: "custom-next", onclick: move |_| next.next(), "Next" }
        }
    });
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
        card: custom.then_some(card),
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
        // A flex row puts the target past the side edge on every renderer; the margin
        // keeps the hole off the window's edge, where it clamps.
        div { id: "outer", style: "height: 200px; width: 300px; margin: 24px; overflow-y: auto;",
            div { style: "height: 600px;" }
            div { id: "inner", style: "overflow-x: auto; display: flex;",
                div { style: "flex: none; width: 900px; height: 1px;" }
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

/// A step targeted by selector in another component, then an interactive one: `#presses`
/// counts presses on `#pressable`. `#seen` reads the stored `seen()`.
#[component]
fn MoreTourPage() -> Element {
    let pressable = use_element();
    let mut presses = use_signal(|| 0u32);
    let tour = use_tour(TourOptions {
        steps: vec![
            TourStep::new("picked")
                .target_selector("#picked")
                .title("Picked"),
            TourStep::new("pressable")
                .target(pressable)
                .interactive(true)
                .title("Pressable"),
        ],
        storage_key: Some("e2e-tour-seen".into()),
        ..Default::default()
    });

    rsx! {
        Flex { direction: "column", gap: "xl", max_width: "320px", sx: sx().padding("16px"),
            Button { id: "start-tour", variant: "outlined", onclick: move |_| tour.start(), "Take the tour" }
            Picked {}
            Button {
                id: "pressable",
                onmounted: pressable.mount(),
                attributes: pressable.attributes(),
                onclick: move |_| presses += 1,
                "Pressable"
            }
            Text { id: "presses", size: "sm", "{presses}" }
            Text { id: "seen", size: "sm", if tour.seen() { "seen" } else { "unseen" } }
            Button { id: "forget", variant: "text", onclick: move |_| tour.forget(), "Forget" }
        }
    }
}

/// A target the tour's owner holds no handle to.
#[component]
fn Picked() -> Element {
    rsx! { Button { id: "picked", "Picked" } }
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

/// An interactive step over a form whose first and last focusables are `display: none`
/// (2673), with a Select (2674) and `#drop`, which empties the steps (2677).
#[component]
fn FormTourPage() -> Element {
    let form = use_element();
    let mut dropped = use_signal(|| false);
    let mut fruit = use_signal(|| None::<Fruit>);
    let steps = match dropped() {
        true => Vec::new(),
        false => vec![
            TourStep::new("form")
                .target(form)
                .interactive(true)
                .title("Form"),
        ],
    };
    let tour = use_tour(TourOptions {
        steps,
        storage_key: Some("e2e-tour-form-seen".into()),
        ..Default::default()
    });

    rsx! {
        Flex { direction: "column", gap: "xl", max_width: "320px", sx: sx().padding("16px"),
            Button {
                id: "start-tour",
                variant: "outlined",
                onclick: move |_| {
                    dropped.set(false);
                    tour.start();
                },
                "Take the tour"
            }
            div { id: "form", onmounted: form.mount(), ..form.attributes(),
                button { id: "hidden-first", style: "display: none", "Hidden first" }
                Select {
                    label: "Fruit",
                    value: fruit(),
                    onchange: move |next| fruit.set(next),
                }
                Button { id: "drop", onclick: move |_| dropped.set(true), "Drop the steps" }
                button { id: "hidden-last", style: "display: none", "Hidden last" }
            }
            Text { id: "seen", size: "sm", if tour.seen() { "seen" } else { "unseen" } }
            Button { id: "forget", variant: "text", onclick: move |_| tour.forget(), "Forget" }
        }
    }
}

/// An interactive target whose DOM order is not its Tab order: `tabindex` 2, 1 and none (2720).
#[component]
fn TabindexTourPage() -> Element {
    let group = use_element();
    let tour = use_tour(TourOptions {
        steps: vec![
            TourStep::new("group")
                .target(group)
                .interactive(true)
                .title("Group"),
        ],
        ..Default::default()
    });

    rsx! {
        Flex { direction: "column", gap: "xl", max_width: "320px", sx: sx().padding("16px"),
            Button { id: "start-tour", variant: "outlined", onclick: move |_| tour.start(), "Take the tour" }
            div { id: "group", onmounted: group.mount(), ..group.attributes(),
                Button { id: "group-middle", "tabindex": "2", "Profile" }
                Button { id: "group-first", "tabindex": "1", "Follow" }
                Button { id: "group-last", "Message" }
            }
        }
    }
}
