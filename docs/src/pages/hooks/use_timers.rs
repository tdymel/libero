use crate::components::{Demo, DemoValues, DocPage, DocSection, Wrap, a11y};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Flex, Text},
    hooks::{use_interval, use_timeout},
};
use web_time::Instant;

/// The two hooks in one component, as `Stopwatch` renders them.
// snippet: mirrors Stopwatch
fn code(_: &DemoValues, _: &str) -> String {
    r#"// `std::time::Instant` panics on wasm; this one reads the browser's clock there.
use web_time::Instant;

let mut since = use_signal(|| None::<Instant>);
let mut banked = use_signal(|| 0.0);
let mut seconds = use_signal(|| 0);
let mut laps = use_signal(Vec::<f64>::new);
let mut saved = use_signal(|| false);
// Reads the clock, so a late tick loses no time.
let elapsed = move || banked() + since().map_or(0.0, |at: Instant| at.elapsed().as_secs_f64());
let interval = use_interval(move || seconds.set(elapsed() as u64), 1000);
let flash = use_timeout(move || saved.set(false), 2000);
let list = laps.read().iter().map(|lap| format!("{lap:.1} s")).collect::<Vec<_>>().join(", ");

rsx! {
    Flex { direction: "column", align: "flex-start", gap: "sm",
        Text { "{seconds} s" }
        Flex { gap: "sm",
            Button {
                onclick: move |_| {
                    match since() {
                        Some(at) => {
                            banked.set(banked() + at.elapsed().as_secs_f64());
                            since.set(None);
                            seconds.set(banked() as u64);
                        }
                        None => since.set(Some(Instant::now())),
                    }
                    interval.toggle();
                },
                if interval.active() { "Stop" } else { "Start" }
            }
            Button {
                variant: "outlined",
                onclick: move |_| {
                    laps.push(elapsed());
                    saved.set(true);
                    flash.start();
                },
                "Save lap"
            }
        }
        div { role: "status",
            if saved() {
                "Lap saved"
            }
        }
        if !list.is_empty() {
            Text { "Laps: {list}" }
        }
    }
}"#
    .to_string()
}

#[component]
fn Stopwatch() -> Element {
    let mut since = use_signal(|| None::<Instant>);
    let mut banked = use_signal(|| 0.0);
    let mut seconds = use_signal(|| 0);
    let mut laps = use_signal(Vec::<f64>::new);
    let mut saved = use_signal(|| false);
    let elapsed = move || banked() + since().map_or(0.0, |at: Instant| at.elapsed().as_secs_f64());
    let interval = use_interval(move || seconds.set(elapsed() as u64), 1000);
    let flash = use_timeout(move || saved.set(false), 2000);
    let list = laps
        .read()
        .iter()
        .map(|lap| format!("{lap:.1} s"))
        .collect::<Vec<_>>()
        .join(", ");

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            Text { "{seconds} s" }
            Flex { gap: "sm",
                Button {
                    onclick: move |_| {
                        match since() {
                            Some(at) => {
                                banked.set(banked() + at.elapsed().as_secs_f64());
                                since.set(None);
                                seconds.set(banked() as u64);
                            }
                            None => since.set(Some(Instant::now())),
                        }
                        interval.toggle();
                    },
                    if interval.active() {
                        "Stop"
                    } else {
                        "Start"
                    }
                }
                Button {
                    variant: "outlined",
                    onclick: move |_| {
                        laps.push(elapsed());
                        saved.set(true);
                        flash.start();
                    },
                    "Save lap"
                }
            }
            div { role: "status",
                if saved() {
                    "Lap saved"
                }
            }
            if !list.is_empty() {
                Text { "Laps: {list}" }
            }
        }
    }
}

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
                component: "use_timers",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { Stopwatch {} },
                wrap: Wrap(code),
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
