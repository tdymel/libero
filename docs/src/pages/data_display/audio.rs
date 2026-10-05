use super::video::volume_parts;
use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, a11y, prop, props};
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
            plays_media: true,
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
                        .doc("How much loads before a press: `none`, `metadata` or `auto`. Unless `none`, a file up to 10 minutes is fetched whole once more to draw the bars, unless its `Content-Length` is over 20 MB."),
                    prop("size", "Size").default("theme").doc("Of the buttons and the seek slider."),
                    prop("onplay", "EventHandler<()>").default("None").doc("Playing started."),
                    prop("onpause", "EventHandler<()>").default("None").doc("Playing paused."),
                    prop("onended", "EventHandler<()>").default("None").doc("Playing reached the end."),
                    prop("onerror", "EventHandler<MediaError>")
                        .default("None")
                        .doc("The source failed: `Aborted`, `Network`, `Decode` or `SourceNotSupported`."),
                    prop("volume_parts", "Parts<VolumePart>").doc("Styles the portaled volume menu and its slider."),
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
                ])
                .volume_parts("VolumePart", volume_parts()),
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
                    "The player is a group named by `label`. Each button and slider is its own Tab stop.",
                    "Buttons say what a press does: Play or Pause, Mute or Unmute. The speed button reads \"Playback speed 1×\" and steps to 1.5×, 2× and back to 1×.",
                    "The seek slider reads the time as \"1:05 of 4:56\". The visible time is hidden from screen readers, so it is not read twice.",
                    "The chevron beside the speaker, named \"Volume\", opens the volume slider and focuses it. Raising the volume unmutes.",
                    "The loudness bars on the seek track are hidden from screen readers; the slider's keys, press and drag work on them.",
                    "Every control fits at 320px wide and 200% zoom: the track shrinks first, and no button gets smaller than 24px (WCAG 1.4.10, 2.5.8).",
                    "The volume bubble's border has 3:1 contrast in light and dark (WCAG 1.4.11).",
                    "A polite \"Loading\" is announced while the sound waits for data, and an alert when the file fails.",
                    "It never plays by itself unless you set `autoplay`; autoplay with sound warns in a debug build (WCAG 1.4.2).",
                ])
                .must([
                    "Give each player a `label` that says what plays.",
                    "Offer a transcript for speech (WCAG 1.2.1): libero cannot write it.",
                ])
                .example("A podcast episode: `Audio { src: \"/episode-12.mp3\", label: \"Episode 12: Accessible forms\" }` with a transcript link under it. A screen reader announces the group \"Episode 12: Accessible forms\", and a deaf user reads the transcript instead.")
                .limits([
                    "Blitz plays no media: the controls give way to `children`, by default a link to the file.",
                    "On a WebView each command and state change crosses the IPC, so the time trails by a moment.",
                    "The bars need the file readable by the page: a file on another origin without CORS, or `preload: none`, keeps placeholder bars drawn from the URL. So does a file over 10 minutes or 20 MB, which is not decoded.",
                ]),
            lead: rsx! {
                Text {
                    "An "
                    Code { source: "<audio>" }
                    " in one compact row, as a chat app's voice message: play, a track of bars to seek, the time, mute with a volume menu and a speed button, "
                    "in the theme's look on every platform that plays media."
                }
            },
            Demo {
                component: "Audio",
                wide_preview: true,
                children_text: "",
                fixed: vec![
                    format!("src: {:?}", crate::samples::SAMPLE_AUDIO),
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
                            src: crate::samples::SAMPLE_AUDIO,
                            label: "Wikipedia guitar solo",
                            size: Input::from(Size::from(values.str("size").as_str())),
                            muted: muted == "true",
                            looping: values.str("looping") == "true",
                        }
                    }
                },
            }

            DocSection {
                title: "Your own layout",
                Text {
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
            }

            DocSection {
                title: "Formats",
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
            }

            DocSection {
                title: "The demo audio",
                Text {
                    "The demo plays \"Wikipedia guitar solo\" (CC0), streamed from Wikimedia "
                    "Commons, so it plays only online."
                }
            }
        }
    }
}
