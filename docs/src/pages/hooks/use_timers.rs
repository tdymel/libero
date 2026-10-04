use crate::components::{Demo, DemoFile, DemoValues, DocPage, DocSection, a11y};
use dioxus::prelude::*;
use libero::components::{Code, Text};

mod demo;
use demo::Stopwatch;

#[component]
pub fn UseTimersPage() -> Element {
    rsx! {
        DocPage {
            title: "Timers",
            source: "libero/src/hooks/timers.rs",
            markdown: "/md/use_timers.md",
            accessibility: a11y()
                .handles([
                    "Both cancel when the component unmounts, so a callback never runs into a screen the reader has left.",
                ])
                .must([
                    "Announce what a timer changes with a live region (`role=\"status\"`), as the demo does for \"Lap saved\": a sighted user sees it appear, a screen reader hears nothing otherwise.",
                    "Give the reader a way to stop anything that moves on its own for more than five seconds (WCAG 2.2.2). A `use_interval` that starts itself needs a Stop control.",
                ])
                .example("A quiz countdown on `use_interval`: a Pause button stops it, and a status line says \"One minute left\" once, rather than every second.")
                .limits([
                    "A tick that lands while the component is still rendering the previous one is skipped, so a callback that counts should read a clock, not add one per tick, if the count must stay exact.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_timeout(callback, ms) -> TimeoutHandle" }
                    " runs a callback once, "
                    Code { source: "ms" }
                    " after "
                    Code { source: "start()" }
                    ". "
                    Code { source: "use_interval(callback, ms) -> IntervalHandle" }
                    " runs it every "
                    Code { source: "ms" }
                    " between "
                    Code { source: "start()" }
                    " and "
                    Code { source: "stop()" }
                    ". Neither starts itself: call "
                    Code { source: "start()" }
                    " from a handler, or from "
                    Code { source: "use_hook" }
                    " to run from mount. "
                    Code { source: "start()" }
                    " on a running timer restarts it, "
                    Code { source: "toggle()" }
                    " flips an interval, and "
                    Code { source: "pending()" }
                    " and "
                    Code { source: "active()" }
                    " are reactive."
                }
            },

            Demo {
                component: "Stopwatch",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { Stopwatch {} },
                file: DemoFile(include_str!("use_timers/demo.rs")),
            }

            DocSection {
                title: "Where it runs",
                Text {
                    "One timer serves the web, Blitz and a webview. The callback runs in "
                    "your component's scope, so it may write signals, spawn or read "
                    "elements. A server render never fires it. Natively every interval "
                    "tick costs a thread, so keep the period to a second or more."
                }
            }

            DocSection {
                title: "The stopwatch",
                Text {
                    "The stopwatch above reads a clock, not the count of ticks, so a late tick "
                    "loses no time. The clock is the "
                    Code { source: "web-time" }
                    " crate's "
                    Code { source: "Instant" }
                    ": "
                    Code { source: "std::time::Instant" }
                    " panics on wasm."
                }
            }
        }
    }
}
