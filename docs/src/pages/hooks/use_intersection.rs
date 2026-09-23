use crate::components::{Demo, DemoValues, DocPage, Wrap, a11y};
use dioxus::prelude::*;
use libero::{
    components::{Code, Text},
    hooks::{IntersectionOptions, use_element, use_intersection},
};

/// The hook in one component, as `Reveal` renders it.
fn code(_: &DemoValues, _: &str) -> String {
    r#"let scroller = use_element();
let seen = use_intersection(IntersectionOptions {
    root: Some(scroller),
    thresholds: vec![0.0, 0.5, 1.0],
    ..Default::default()
});
let percent = seen.entry.read().map_or(0, |entry| (entry.ratio * 100.0).round() as u32);

rsx! {
    div {
        onmounted: scroller.mount(),
        style: "height: 8rem; overflow: auto;",
        div { style: "height: 12rem;", "Scroll down" }
        div { onmounted: move |event| seen.on_mounted.call(event), "{percent}% visible" }
        div { style: "height: 12rem;" }
    }
}"#
    .to_string()
}

#[component]
fn Reveal() -> Element {
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
        div {
            onmounted: scroller.mount(),
            tabindex: "0",
            style: "height: 8rem; overflow: auto; border: 1px solid currentColor;",
            div { style: "height: 12rem;", "Scroll down" }
            div { onmounted: move |event| seen.on_mounted.call(event), "{percent}% visible" }
            div { style: "height: 12rem;" }
        }
    }
}

#[component]
pub fn UseIntersectionPage() -> Element {
    rsx! {
        DocPage {
            title: "use_intersection, use_in_viewport",
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
                    "Only the web observes. On Blitz, in a WebView and in a server render `entry` stays `None`: treat `None` as \"unknown\" and show lazy content, rather than waiting for a sighting that never comes.",
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
                    " and read "
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
                    Code { source: "use_in_viewport() -> (handler, ReadSignal<bool>)" }
                    " is the same with the defaults, as a bool. Where nothing can observe (Blitz, a WebView, a server render) "
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
