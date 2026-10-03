//! `use_intersection` and `use_in_viewport`, as their docs page uses them: a
//! target below the fold of the page, and one inside a scroller.

use dioxus::prelude::*;
use libero::hooks::{IntersectionOptions, use_element, use_in_viewport, use_intersection};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/use-intersection/viewport", || rsx! { Viewport {} }),
    ("/use-intersection/clipped", || rsx! { Clipped {} }),
    ("/use-intersection/root-clip", || rsx! { RootClip {} }),
    ("/use-intersection/once", || rsx! { Once {} }),
    ("/use-intersection/root", || rsx! { Rooted {} }),
];

#[component]
fn Viewport() -> Element {
    let seen = use_in_viewport();
    // A handle's attributes on the same target (1255).
    let handle = use_element();
    rsx! {
        div { style: "height: 200vh;" }
        div {
            id: "target",
            style: "height: 40px;",
            onmounted: move |event| {
                handle.mount()(event.clone());
                seen.on_mounted.call(event);
            },
            ..handle.attributes(),
            ..seen.attributes,
            "target"
        }
        p { id: "state",
            if (seen.visible)() {
                "in"
            } else {
                "out"
            }
        }
        div { style: "height: 200vh;" }
    }
}

/// A target whose box lies in the viewport but is clipped by a scroller: out. A plain one beside it: in.
#[component]
fn Clipped() -> Element {
    let hidden = use_in_viewport();
    let shown = use_in_viewport();
    rsx! {
        div { style: "height: 40px; overflow: hidden;",
            div { style: "height: 100px;" }
            div {
                id: "hidden",
                style: "height: 20px;",
                onmounted: move |event| hidden.on_mounted.call(event),
                ..hidden.attributes,
                "hidden"
            }
        }
        div {
            id: "shown",
            style: "height: 20px;",
            onmounted: move |event| shown.on_mounted.call(event),
            ..shown.attributes,
            "shown"
        }
        p { id: "state",
            if (shown.visible)() {
                "shown in"
            } else {
                "shown out"
            }
            if (hidden.visible)() {
                ", hidden in"
            } else {
                ", hidden out"
            }
        }
    }
}

/// Two targets in a root that clips one of them, though both lie in the viewport.
#[component]
fn RootClip() -> Element {
    let root = use_element();
    let options = || IntersectionOptions {
        root: Some(root),
        ..Default::default()
    };
    let hidden = use_intersection(options());
    let shown = use_intersection(options());
    let is_in = |seen: &libero::hooks::Intersection| {
        seen.entry.read().is_some_and(|entry| entry.is_intersecting)
    };
    rsx! {
        div {
            style: "height: 40px; overflow: hidden;",
            onmounted: root.mount(),
            ..root.attributes(),
            div {
                id: "shown",
                style: "height: 20px;",
                onmounted: move |event| shown.on_mounted.call(event),
                ..shown.attributes.clone(),
                "shown"
            }
            div { style: "height: 100px;" }
            div {
                id: "hidden",
                style: "height: 20px;",
                onmounted: move |event| hidden.on_mounted.call(event),
                ..hidden.attributes.clone(),
                "hidden"
            }
        }
        p { id: "state",
            if is_in(&shown) {
                "shown in"
            } else {
                "shown out"
            }
            if is_in(&hidden) {
                ", hidden in"
            } else {
                ", hidden out"
            }
        }
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
            ..seen.attributes,
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
            ..scroller.attributes(),
            // Focus scrolls the scroller on every backend: `#top` back, `#target` in.
            div { id: "top", tabindex: "-1", style: "height: 1px;" }
            div { style: "height: 300px;" }
            div {
                id: "target",
                tabindex: "-1",
                style: "height: 60px;",
                onmounted: move |event| seen.on_mounted.call(event),
                ..seen.attributes,
                "target"
            }
            div { style: "height: 300px;" }
        }
    }
}
