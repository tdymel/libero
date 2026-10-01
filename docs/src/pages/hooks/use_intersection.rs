use crate::components::{Demo, DemoValues, DocPage, Wrap, a11y};
use dioxus::prelude::*;
use libero::{
    components::{Box, Code, ScrollArea, Text},
    hooks::{IntersectionOptions, use_intersection},
    sx::sx,
};

/// The hook in one component, as `Reveal` renders it.
fn code(_: &DemoValues, _: &str) -> String {
    r#"let seen = use_intersection(IntersectionOptions {
    thresholds: vec![0.0, 0.5, 1.0],
    ..Default::default()
});
let percent = seen.entry.read().map_or(0, |entry| (entry.ratio * 100.0).round() as u32);

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
}"#
    .to_string()
}

#[component]
fn Reveal() -> Element {
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

#[component]
pub fn UseIntersectionPage() -> Element {
    rsx! {
        DocPage {
            title: "Intersection",
            source: "libero/src/hooks/intersection.rs",
            markdown: "/md/use_intersection.md",
            accessibility: a11y()
                .handles([
                    "The observer is dropped when the component unmounts, and replaced when an option changes, so one element never has two.",
                ])
                .must([
                    "Keep content that matters in the document and reachable by keyboard; `use_intersection` only reports, it hides nothing. An infinite list still needs a \"Load more\" button.",
                    "Gate reveal animations on the reader's motion setting (`use_accessibility`).",
                ])
                .limits([
                    "The web and a WebView (desktop, Android) use an `IntersectionObserver`. A WebView finds the element by `attributes`, so spread them on it; without them nothing is observed there. A `root` needs `root.attributes()` spread on it the same way; without them a WebView observes against the viewport (a warning in debug builds). On Blitz and in a server render `entry` stays `None`: treat `None` as \"unknown\" and show lazy content, rather than waiting for a sighting that never comes.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_intersection(options) -> Intersection" }
                    " watches one element with the browser's "
                    Code { source: "IntersectionObserver" }
                    ". Give "
                    Code { source: "on_mounted" }
                    " to the element's "
                    Code { source: "onmounted" }
                    ", spread its "
                    Code { source: "attributes" }
                    " on it (a WebView finds the element by them) and read "
                    Code { source: "entry" }
                    ", a signal of "
                    Code { source: "Option<IntersectionEntry>" }
                    " with "
                    Code { source: "is_intersecting" }
                    " and the visible "
                    Code { source: "ratio" }
                    ". The options name a "
                    Code { source: "root" }
                    " element (the viewport by default), a "
                    Code { source: "root_margin" }
                    ", the "
                    Code { source: "thresholds" }
                    " at which the entry updates, and "
                    Code { source: "once" }
                    ", which stops observing after the first sighting."
                }
                Text {
                    Code { source: "use_in_viewport() -> InViewport" }
                    " is the same with the defaults, with "
                    Code { source: "visible" }
                    " as a bool. Where nothing can observe (Blitz, a server render) "
                    Code { source: "entry" }
                    " stays "
                    Code { source: "None" }
                    " and the bool is "
                    Code { source: "false" }
                    "."
                }
            },

            Demo {
                component: "use_intersection",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { Reveal {} },
                wrap: Wrap(code),
            }
        }
    }
}
