use crate::components::{Demo, DemoValues, DocPage, Wrap, a11y};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Flex, Text},
    hooks::{LongPressOptions, use_long_press},
};

/// The hook in one component, as `HoldToDelete` renders it.
fn code(_: &DemoValues, _: &str) -> String {
    r#"let mut taps = use_signal(|| 0);
let mut holds = use_signal(|| 0);
let press = use_long_press(
    Callback::new(move |()| holds += 1),
    LongPressOptions::default(),
);

rsx! {
    Flex { direction: "column", align: "flex-start", gap: "sm",
        Button {
            onpointerdown: move |event| press.onpointerdown.call(event),
            onpointermove: move |event| press.onpointermove.call(event),
            onpointerup: move |event| press.onpointerup.call(event),
            onpointerleave: move |event| press.onpointerleave.call(event),
            onpointercancel: move |event| press.onpointercancel.call(event),
            oncontextmenu: move |event| press.oncontextmenu.call(event),
            onclick: move |event| {
                if !press.onclick.call(event) {
                    taps += 1;
                }
            },
            if (press.pressing)() {
                "Keep holding"
            } else {
                "Tap, or hold"
            }
        }
        div { role: "status", "{taps} taps, {holds} holds" }
    }
}"#
    .to_string()
}

#[component]
fn HoldToCount() -> Element {
    let mut taps = use_signal(|| 0);
    let mut holds = use_signal(|| 0);
    let press = use_long_press(
        Callback::new(move |()| holds += 1),
        LongPressOptions::default(),
    );

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            Button {
                onpointerdown: move |event| press.onpointerdown.call(event),
                onpointermove: move |event| press.onpointermove.call(event),
                onpointerup: move |event| press.onpointerup.call(event),
                onpointerleave: move |event| press.onpointerleave.call(event),
                onpointercancel: move |event| press.onpointercancel.call(event),
                oncontextmenu: move |event| press.oncontextmenu.call(event),
                onclick: move |event| {
                    if !press.onclick.call(event) {
                        taps += 1;
                    }
                },
                if (press.pressing)() {
                    "Keep holding"
                } else {
                    "Tap, or hold"
                }
            }
            div { role: "status", "{taps} taps, {holds} holds" }
        }
    }
}

#[component]
pub fn UseLongPressPage() -> Element {
    rsx! {
        DocPage {
            title: "Long press",
            source: "libero/src/hooks/long_press.rs",
            markdown: "/md/use_long_press.md",
            accessibility: a11y()
                .handles([
                    "A tap that ends before the delay runs no callback and keeps its click.",
                    "The press is dropped when the pointer moves beyond the tolerance, leaves, is cancelled (a touch that starts to scroll), or a second finger lands.",
                    "The browser's own long-press menu is suppressed only after the press fired, and the click that follows the release is reported so you can skip it.",
                ])
                .must([
                    "Offer the same action without holding: a long press is a gesture with no keyboard or switch equivalent (WCAG 2.5.1, 2.1.1). Put it on a context menu, a key such as Shift+F10, or a visible button.",
                    "Announce what the press did with a live region, as the demo does.",
                ])
                .limits([
                    "It reacts to pointer events only. Keyboard and screen reader activation arrive as clicks and do not count as a press.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_long_press(on_long_press, options) -> LongPress" }
                    " calls "
                    Code { source: "on_long_press" }
                    " once a pointer stays down for "
                    Code { source: "options.ms" }
                    " (400 by default). It moves no more than "
                    Code { source: "move_tolerance" }
                    " px (10) in that time. Spread the returned callbacks onto one element: "
                    Code { source: "onpointerdown" }
                    ", "
                    Code { source: "onpointermove" }
                    ", "
                    Code { source: "onpointerup" }
                    ", "
                    Code { source: "onpointerleave" }
                    ", "
                    Code { source: "onpointercancel" }
                    ", "
                    Code { source: "oncontextmenu" }
                    " and "
                    Code { source: "onclick" }
                    "."
                }
                Text {
                    "Read "
                    Code { source: "pressing" }
                    ", a signal that is true while a press is held and not yet fired, for a hold cue."
                }
                Text {
                    "Give the element "
                    Code { source: "user-select: none" }
                    " and "
                    Code { source: "-webkit-touch-callout: none" }
                    " so holding does not select text. It runs on the web, Blitz and a WebView, on one timer."
                }
            },

            Demo {
                component: "use_long_press",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { HoldToCount {} },
                wrap: Wrap(code),
            }
        }
    }
}
