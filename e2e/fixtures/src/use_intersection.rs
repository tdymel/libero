//! `use_intersection` and `use_in_viewport`, as their docs page uses them: a
//! target below the fold of the page, and one inside a scroller.

use dioxus::prelude::*;
use libero::hooks::{IntersectionOptions, use_element, use_in_viewport, use_intersection};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/use-intersection/viewport", || rsx! { Viewport {} }),
    ("/use-intersection/once", || rsx! { Once {} }),
    ("/use-intersection/root", || rsx! { Rooted {} }),
];

#[component]
fn Viewport() -> Element {
    let (on_mounted, visible) = use_in_viewport();
    rsx! {
        div { style: "height: 200vh;" }
        div {
            id: "target",
            style: "height: 40px;",
            onmounted: move |event| on_mounted.call(event),
            "target"
        }
        p { id: "state",
            if visible() {
                "in"
            } else {
                "out"
            }
        }
        div { style: "height: 200vh;" }
    }
}

#[component]
fn Once() -> Element {
    let seen = use_intersection(IntersectionOptions {
        once: true,
        ..Default::default()
    });
    let seen_it = seen.entry.read().is_some_and(|entry| entry.is_intersecting);
    rsx! {
        div { style: "height: 200vh;" }
        div {
            id: "target",
            style: "height: 40px;",
            onmounted: move |event| seen.on_mounted.call(event),
            "target"
        }
        p { id: "state",
            if seen_it {
                "seen"
            } else {
                "unseen"
            }
        }
        div { style: "height: 200vh;" }
    }
}

#[component]
fn Rooted() -> Element {
    let scroller = use_element();
    let seen = use_intersection(IntersectionOptions {
        root: Some(scroller),
        thresholds: vec![0.0, 0.5, 1.0],
        ..Default::default()
    });
    let percent = seen
        .entry
        .read()
        .map_or(0, |entry| (entry.ratio * 100.0).round() as u32);
    rsx! {
        p { id: "state", "{percent}" }
        div {
            id: "scroller",
            onmounted: scroller.mount(),
            style: "height: 100px; overflow: auto;",
            div { style: "height: 300px;" }
            div {
                id: "target",
                style: "height: 60px;",
                onmounted: move |event| seen.on_mounted.call(event),
                "target"
            }
            div { style: "height: 300px;" }
        }
    }
}
