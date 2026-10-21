use crate::components::{Demo, DemoFile, DemoValues, DocPage, a11y};
use dioxus::prelude::*;
use libero::components::{Code, Text};

mod demo;
use demo::Offer;

#[component]
pub fn UseCountdownPage() -> Element {
    rsx! {
        DocPage {
            title: "Countdown",
            source: "libero/src/hooks/countdown.rs",
            markdown: "/md/use_countdown.md",
            accessibility: a11y()
                .handles([
                    "`DurationStyle::Spoken` says the time in the active language's words, `1 minute 30 seconds`, where a clock face would be read as a ratio or a time of day.",
                    "Every countdown on the page turns on the same second, so a reader comparing two never sees them disagree.",
                ])
                .must([
                    "Hide the clock face from screen readers (`aria-hidden`) and put the spoken text beside it in a `VisuallyHidden`, or in the element's `aria-label`.",
                    "Keep the label out of a live region: it is not `aria-live`, since a polite region would speak every second. Announce a milestone once instead, such as \"One minute left\", in a `role=\"status\"`.",
                    "Give the reader a way to ask for more time when the countdown ends something they are doing (WCAG 2.2.1).",
                ])
                .example("A checkout hold: \"Seats held for 9:41\" with \"9 minutes 41 seconds\" in a `VisuallyHidden`, a status line saying \"One minute left\" once, and an Extend button.")
                .limits([
                    "Whole seconds, rounded up: the label reads `0:00` only once the end has passed, up to a second after it.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_countdown(end: SystemTime) -> ReadSignal<Duration>" }
                    " is the time left until "
                    Code { source: "end" }
                    ", live, and "
                    Code { source: "Duration::ZERO" }
                    " once it passed. All mounted countdowns share one 1 s ticker, aligned to the second, which stops when none is mounted or all reached zero. "
                    Code { source: "format_duration(span, DurationStyle::Clock | Short | Spoken)" }
                    " writes it as "
                    Code { source: "1:30" }
                    ", "
                    Code { source: "1 min 30 s" }
                    " or "
                    Code { source: "1 minute 30 seconds" }
                    " in the active language. Build "
                    Code { source: "end" }
                    " from "
                    Code { source: "platform::clock().now()" }
                    ": "
                    Code { source: "SystemTime::now()" }
                    " panics on wasm."
                }
            },

            Demo {
                component: "Offer",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { Offer {} },
                file: DemoFile(include_str!("use_countdown/demo.rs")),
            }
        }
    }
}
