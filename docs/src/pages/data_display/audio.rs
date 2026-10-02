use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Audio, AudioPart, Code, CodeBlock, Input, Text},
    theme::Size,
};

const FORMATS: &str = r#"Audio {
    src: "/message.mp3",
    sources: vec![MediaSource::new("/message.ogg", "audio/ogg")],
    label: "Voice message",
}"#;

#[component]
pub fn AudioPage() -> Element {
    rsx! {
        DocPage {
            title: "Audio",
            source: "libero/src/components/data_display/audio.rs",
            markdown: "/md/audio.md",
            properties: vec![
                props("Audio", vec![
                    prop("src", "String").default("required").doc("The file's URL."),
                    prop("sources", "Vec<MediaSource>")
                        .default("[]")
                        .doc("The file in other formats, `MediaSource::new(src, mime)`, tried in order before `src`. `src` stays the fallback and the download link."),
                    prop("label", "String")
                        .default("required")
                        .doc("Names the player, e.g. the track's title."),
                    prop("media", "Option<MediaHandle>")
                        .default("None")
                        .doc("A handle from `use_media()`, to drive or read the player from outside."),
                    prop("autoplay", "bool")
                        .default("false")
                        .doc("Starts on load. Browsers refuse it with sound: pair it with `muted`; a debug build warns otherwise."),
                    prop("muted", "bool").default("false").doc("Starts muted."),
                    prop("looping", "bool").default("false").doc("Starts again at the end."),
                    prop("preload", "MediaPreload")
                        .default("metadata")
                        .doc("How much loads before a press: `none`, `metadata` or `auto`. Unless `none`, the whole file is fetched once to draw the bars."),
                    prop("size", "Size").default("theme").doc("Of the buttons and the seek slider."),
                    prop("onplay", "EventHandler<()>").default("None").doc("Playing started."),
                    prop("onpause", "EventHandler<()>").default("None").doc("Playing paused."),
                    prop("onended", "EventHandler<()>").default("None").doc("Playing reached the end."),
                    prop("onerror", "EventHandler<MediaError>")
                        .default("None")
                        .doc("The source failed: `Aborted`, `Network`, `Decode` or `SourceNotSupported`."),
                    prop("children", "Element")
                        .default("a sentence and a link")
                        .doc("Shown instead of the controls where nothing plays media (Blitz)."),
                ])
                .parts("AudioPart", vec![
                    (AudioPart::Controls, "The row of controls, a named `group`."),
                    (AudioPart::Time, "The time: the total until playing starts, then the elapsed."),
                    (AudioPart::Seek, "The seek slider's wrapper; its `SliderTrack::Bars` track draws the bars."),
                    (AudioPart::Volume, "The mute button and the volume menu's trigger."),
                    (AudioPart::Message, "The error text, or the fallback where nothing plays media."),
                ]),
            ],
            accessibility: a11y()
                .key(["K"], "Plays or pauses, with focus anywhere in the player.")
                .key(["Space"], "On a slider: plays or pauses. On a button: presses it.")
                .key(["J", "L"], "Jumps 10 seconds back or ahead.")
                .key(["M"], "Mutes or unmutes.")
                .key(["Shift+?"], "Lists these keys in a `ShortcutHelp` dialog.")
                .key(["Left", "Right"], "On the seek slider: 1 second; on the volume slider: 5%.")
                .key(["Escape"], "Closes the volume menu and returns to its button.")
                .key(["Tab"], "Moves through every control in visual order: play, seek, mute, volume, speed. In the volume menu, returns to its button.")
                .handles([
                    "The player is a `group` named by `label`, its controls a `group` named \"Player controls\"; every button and slider is its own Tab stop, as in the browser's own controls.",
                    "The speed button shows the rate (\"1×\") and is named \"Playback speed 1×\"; a press steps to 1.5×, 2× and back to 1×, and the name follows.",
                    "The play button changes its name (Play/Pause) rather than using `aria-pressed`.",
                    "The visible time is hidden from screen readers: the seek slider's `aria-valuetext` reads \"1:05 of 4:56\" (the localization's `media.position`).",
                    "The speaker button mutes and unmutes, its name following (Mute/Unmute). The chevron beside it, named \"Volume\", opens a `dialog` holding the volume slider and focuses it; moving the volume up unmutes.",
                    "The seek track is a row of bars, the played ones filled: the seek slider's `SliderTrack::Bars`, so it keeps the slider's keys, press and drag. The bars follow the sound's loudness, decoded from the file, and are hidden from screen readers. A file over 10 minutes or 20 MB is not decoded: its bars stay drawn from the URL.",
                    "The row never wraps: the seek track shrinks first, then the time goes, then the buttons shrink, to 24px at the least (WCAG 2.5.8): every control stays at 320px and 200% zoom (WCAG 1.4.10).",
                    "The bubble has a border at 3:1 against the page in light and dark (WCAG 1.4.11).",
                    "A polite status says \"Loading\" while playing waits for data; a failed source shows an alert.",
                    "No autoplay unless asked, and a debug warning for autoplay with sound (WCAG 1.4.2).",
                ])
                .must([
                    "Give each player a `label` that says what plays.",
                    "Offer a transcript for speech (WCAG 1.2.1): libero cannot write it.",
                ])
                .limits([
                    "Blitz plays no media: the controls give way to `children`, by default a link to the file.",
                    "On a WebView each command and state change crosses the IPC, so the time trails by a moment.",
                    "The bars need the file readable by the page: a file on another origin without CORS, or `preload: none`, keeps placeholder bars drawn from the URL.",
                ]),
            lead: rsx! {
                Text {
                    "An "
                    Code { source: "<audio>" }
                    " in one compact row, as a chat app's voice message: play, a track of bars to seek, the time, mute with a volume menu and a speed button, "
                    "in the theme's look on every platform that plays media. "
                    Code { source: "use_media()" }
                    " drives the same engine for a layout of your own: spread its "
                    Code { source: "attributes()" }
                    " and "
                    Code { source: "mount()" }
                    " on a bare element and read "
                    Code { source: "paused()" }
                    ", "
                    Code { source: "current_time()" }
                    " and friends."
                }
                Text {
                    "A browser plays the first "
                    Code { source: "sources" }
                    " entry whose type it supports, then "
                    Code { source: "src" }
                    ". An Ogg first and an MP3 as "
                    Code { source: "src" }
                    " reach every browser, Safari included."
                }
                CodeBlock { source: FORMATS, language: "rust" }
                Text {
                    "The demo plays \"Wikipedia guitar solo\" (CC0), streamed from Wikimedia "
                    "Commons, so it plays only online."
                }
            },
            Demo {
                component: "Audio",
                children_text: "",
                fixed: vec![
                    format!("src: {:?}", crate::site::SAMPLE_AUDIO),
                    r#"label: "Wikipedia guitar solo""#.to_string(),
                ],
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl"]).default("md"),
                    Control::switch("muted"),
                    Control::switch("looping"),
                ],
                // `muted` only sets the start, so the switch remounts the player (todo 1419).
                render: move |values: DemoValues| {
                    let muted = values.str("muted");
                    rsx! {
                        Audio {
                            key: "{muted}",
                            src: crate::site::SAMPLE_AUDIO,
                            label: "Wikipedia guitar solo",
                            size: Input::from(Size::from(values.str("size").as_str())),
                            muted: muted == "true",
                            looping: values.str("looping") == "true",
                        }
                    }
                },
            }
        }
    }
}
