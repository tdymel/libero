use crate::components::{Demo, DemoValues, DocPage, ExtraTab, Wrap, a11y, prose};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Flex, Table, Text, column},
    hooks::use_media,
};

/// `MediaHandle`'s methods, as the markdown's API table lists them.
const METHODS: [(&str, &str); 10] = [
    (
        "attributes(), mount()",
        "Spread on the element and set as its `onmounted`, so the handle finds it on every platform.",
    ),
    (
        "element()",
        "The `ElementHandle` underneath, for focus or measuring.",
    ),
    (
        "play(), pause(), toggle()",
        "A browser may refuse a `play()` no press started (autoplay policy): it then stays paused.",
    ),
    (
        "seek(seconds)",
        "Jumps from the start, clamped to the duration once known.",
    ),
    (
        "set_volume(0.0..=1.0), set_muted(bool), set_rate(f64)",
        "Volume, mute and speed; `1.0` is normal speed.",
    ),
    (
        "paused(), ended(), current_time(), volume(), muted(), rate()",
        "The element's state, each its own signal.",
    ),
    (
        "duration()",
        "Seconds; `None` until the metadata loads, and for a live stream.",
    ),
    (
        "buffering()",
        "Playing was asked for, but the data has not arrived yet.",
    ),
    (
        "error()",
        "Why the source failed to load or play, if it did: a `MediaError` of `Aborted`, `Network`, `Decode` or `SourceNotSupported`.",
    ),
    (
        "is_supported()",
        "`false` until mounted, then whether anything plays media here.",
    ),
];

/// The hook in one component, as `OwnPlayer` renders it.
fn code(_: &DemoValues, _: &str) -> String {
    r#"let media = use_media();
let time = media.current_time() as u64;
let status = match (media.error(), media.buffering()) {
    (Some(_), _) => "The audio could not be played.",
    (None, true) => "Loading",
    (None, false) => "",
};

rsx! {
    audio { src: "/podcast.mp3", onmounted: media.mount(), ..media.attributes() }
    Flex { gap: "sm", align: "center",
        Button {
            onclick: move |_| media.toggle(),
            if media.paused() { "Play" } else { "Pause" }
        }
        Button { variant: "outlined", onclick: move |_| media.seek(0.0), "Restart" }
        Text { "{time / 60}:{time % 60:02}" }
        div { role: "status", "{status}" }
    }
}"#
    .to_string()
}

#[component]
fn OwnPlayer() -> Element {
    let media = use_media();
    let time = media.current_time() as u64;
    let status = match (media.error(), media.buffering()) {
        (Some(_), _) => "The audio could not be played.",
        (None, true) => "Loading",
        (None, false) => "",
    };
    rsx! {
        audio { src: crate::site::SAMPLE_AUDIO, onmounted: media.mount(), ..media.attributes() }
        Flex { gap: "sm", align: "center",
            Button {
                onclick: move |_| media.toggle(),
                if media.paused() { "Play" } else { "Pause" }
            }
            Button { variant: "outlined", onclick: move |_| media.seek(0.0), "Restart" }
            Text { "{time / 60}:{time % 60:02}" }
            div { role: "status", "{status}" }
        }
    }
}

#[component]
pub fn UseMediaPage() -> Element {
    rsx! {
        DocPage {
            title: "Media",
            source: "libero/src/hooks/media.rs",
            markdown: "/md/use_media.md",
            accessibility: a11y()
                .handles([
                    "It renders nothing and announces nothing: your controls carry the names and states.",
                    "Commands from your controls only: it never starts playing by itself.",
                ])
                .must([
                    "Give every control a name that says what it does now, such as Play or Pause.",
                    "Announce buffering and errors yourself, from `buffering()` and `error()`, as the demo's status line and `Audio` and `Video` do.",
                    "Never autoplay sound (WCAG 1.4.2); offer captions and a transcript as for any media.",
                ])
                .limits([
                    "Blitz and a server render play no media: every command does nothing and `is_supported()` stays false.",
                    "On a WebView each command and state change crosses the IPC, so the time trails by a moment.",
                ]),
            extra_tab: ExtraTab {
                label: "Methods",
                id: "methods",
                content: rsx! {
                    Text {
                        Code { source: "MediaHandle" }
                        " is "
                        Code { source: "Copy" }
                        ". On a WebView the time moves at most every 250 ms."
                    }
                    Table {
                        caption: "Methods of MediaHandle",
                        data: METHODS.to_vec(),
                        columns: vec![
                            column("Method")
                                .value(|(method, _): &(&str, &str)| *method)
                                .render(|(method, _): &(&str, &str)| rsx! { Code { source: *method } }),
                            column("What it does")
                                .value(|(_, doc): &(&str, &str)| *doc)
                                .render(|(_, doc): &(&str, &str)| prose(doc)),
                        ],
                    }
                },
            },
            lead: rsx! {
                Text {
                    Code { source: "use_media() -> MediaHandle" }
                    " plays and reads one "
                    Code { source: "<audio>" }
                    " or "
                    Code { source: "<video>" }
                    " you render yourself: spread its "
                    Code { source: "attributes()" }
                    " and set "
                    Code { source: "mount()" }
                    " as the element's "
                    Code { source: "onmounted" }
                    ". Each read is its own signal, so a time tick re-renders only what shows the time. "
                    Code { source: "Audio" }
                    " and "
                    Code { source: "Video" }
                    " are built on it and take a handle through their "
                    Code { source: "media" }
                    " prop."
                }
            },

            Demo {
                component: "use_media",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { OwnPlayer {} },
                wrap: Wrap(code),
            }
        }
    }
}
