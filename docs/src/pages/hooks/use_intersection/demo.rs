use dioxus::prelude::*;
use libero::{
    components::{Box, ScrollArea},
    hooks::{IntersectionOptions, use_intersection},
    sx::sx,
};

#[component]
pub fn Reveal() -> Element {
    let seen = use_intersection(IntersectionOptions {
        thresholds: vec![0.0, 0.5, 1.0],
        ..Default::default()
    });
    let percent = seen
        .entry
        .read()
        .map_or(0, |entry| (entry.ratio * 100.0).round() as u32);

    // No `root`: the viewport's observer also clips by the scrolling area around the element.
    rsx! {
        Box { sx: sx().width("16rem").height("8rem").border("1px solid currentColor"),
            ScrollArea { aria_label: "Scrolling box",
                div { style: "height: 12rem;", "Scroll down" }
                div {
                    onmounted: move |event| seen.on_mounted.call(event),
                    ..seen.attributes,
                    "{percent}% visible"
                }
                div { style: "height: 12rem;" }
            }
        }
    }
}
