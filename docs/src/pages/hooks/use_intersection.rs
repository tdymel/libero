use crate::components::{Demo, DemoFile, DemoValues, DocPage, a11y};
use dioxus::prelude::*;
use libero::components::{Code, Text};

mod demo;
use demo::Reveal;

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
                .example("An endless product list that loads the next page once its last row comes into view: a \"Load more\" button under the list does the same for a keyboard user, and every loaded row stays in the document.")
                .limits([
                    "The web and a WebView (desktop, Android) use an `IntersectionObserver`. A WebView finds the element by `attributes`, so spread them on it; without them nothing is observed there. A `root` needs `root.attributes()` spread on it the same way; without them a WebView observes against the viewport (a warning in debug builds). Blitz has no observer and is polled instead: at each render, after a scroll or input, and every 500 ms, so a change no scroll or input caused (a window resize, an animation) shows up to 500 ms late. In a server render `entry` stays `None`: treat `None` as \"unknown\" and show lazy content, rather than waiting for a sighting that never comes.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_intersection(options) -> Intersection" }
                    " watches one element with the browser's "
                    Code { source: "IntersectionObserver" }
                    ", polled on Blitz. Give "
                    Code { source: "on_mounted" }
                    " to the element's "
                    Code { source: "onmounted" }
                    ", spread its "
                    Code { source: "attributes" }
                    " on it and read "
                    Code { source: "entry" }
                    ". The options name a "
                    Code { source: "root" }
                    ", a "
                    Code { source: "root_margin" }
                    ", the "
                    Code { source: "thresholds" }
                    " and "
                    Code { source: "once" }
                    ". "
                    Code { source: "use_in_viewport() -> InViewport" }
                    " is the same with the defaults, with "
                    Code { source: "visible" }
                    " as a bool."
                }
            },

            Demo {
                component: "Reveal",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { Reveal {} },
                file: DemoFile(include_str!("use_intersection/demo.rs")),
            }
        }
    }
}
