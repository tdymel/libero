use crate::components::{Demo, DemoFile, DemoValues, DocPage, DocSection, a11y};
use dioxus::prelude::*;
use libero::components::{Code, Text};

mod demo;
use demo::SwipePad;

#[component]
pub fn UseSwipePage() -> Element {
    rsx! {
        DocPage {
            title: "Swipe",
            source: "libero/src/hooks/swipe.rs",
            markdown: "/md/use_swipe.md",
            accessibility: a11y()
                .handles([
                    "A tap, a scroll and a pinch keep working: the hooks prevent no default but an edge swipe's sideways pan of an inner scroller, and a second finger drops the press.",
                    "The edge swipe starts past Android's system back zone, so Back from the screen edge still works.",
                    "A mouse is ignored, so a desktop click or text selection never turns into a swipe.",
                ])
                .must([
                    "Offer the same action without a gesture: a swipe is a path-based gesture (WCAG 2.5.1, 2.5.7). The docs keep the burger as the single-pointer way to open the navigation; the demo has buttons.",
                    "Announce what the swipe did with a live region, or move focus where it leads.",
                ])
                .example("A photo strip that goes to the next photo on a left swipe: Previous and Next buttons do the same, and a status line says \"Photo 3 of 8\" after each move.")
                .limits([
                    "Touch and pen only. On iOS and in a mobile browser tab the system or the browser owns the very screen edge; the edge swipe band starts past it, but its inset was measured on Android only.",
                    "Blitz has no touch input, so the hooks are inert in native windows.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_swipe(on_swipe, options) -> Swipe" }
                    " calls "
                    Code { source: "on_swipe" }
                    " once a touch or pen moves "
                    Code { source: "options.distance" }
                    " px (48) along one axis, more than along the other. It fires during the move, once per press, with the "
                    Code { source: "direction" }
                    ", the start point and the delta."
                }
            },

            Demo {
                component: "SwipePad",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { SwipePad {} },
                file: DemoFile(include_str!("use_swipe/demo.rs")),
            }

            DocSection {
                title: "Wiring",
                Text {
                    "Spread "
                    Code { source: "onpointerdown" }
                    ", "
                    Code { source: "onpointermove" }
                    ", "
                    Code { source: "onpointerup" }
                    " and "
                    Code { source: "onpointercancel" }
                    " onto one element, and give it a "
                    Code { source: "touch-action" }
                    ": "
                    Code { source: "none" }
                    " for all four directions, "
                    Code { source: "pan-y" }
                    " for sideways swipes on a page that scrolls."
                }
            }

            DocSection {
                title: "Edge swipes",
                Text {
                    Code { source: "use_edge_swipe(on_swipe, options) -> Swipe" }
                    " is built on it: an inward swipe that starts in a band near the start edge (the right edge under RTL) opens your drawer. The band begins "
                    Code { source: "inset" }
                    " px (44) in, past Android 16's system back zone at its highest sensitivity, and is "
                    Code { source: "width" }
                    " px (48) wide, so it needs no gesture exclusion. Spread it with "
                    Code { source: "edge_swipe_sx()" }
                    " on a box covering the page: no strip lies over the content. A swipe from the band that goes inward, more sideways than up or down, also starts on a code block or another inner scroller: the hook keeps that box from scrolling sideways for the press. These docs open their navigation this way on a phone, and the drawer follows the finger: past a third of its width or on a flick it opens."
                }
            }
        }
    }
}
