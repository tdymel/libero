use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, a11y, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Code, CodeBlock, Input, Text, Video, VideoPart},
    theme::Size,
};

const FORMATS: &str = r#"Video {
    src: "/launch.mp4",
    sources: vec![MediaSource::new("/launch.webm", "video/webm")],
    label: "Launch day",
}"#;

#[component]
pub fn VideoPage() -> Element {
    rsx! {
        DocPage {
            title: "Video",
            source: "libero/src/components/data_display/video.rs",
            markdown: "/md/video.md",
            properties: vec![
                props("Video", vec![
                    prop("src", "String").default("required").doc("The file's URL."),
                    prop("sources", "Vec<MediaSource>")
                        .default("[]")
                        .doc("The file in other formats, `MediaSource::new(src, mime)`, tried in order before `src`. `src` stays the fallback and the download link."),
                    prop("label", "String")
                        .default("required")
                        .doc("Names the player, e.g. the video's title."),
                    prop("poster", "Option<String>")
                        .default("None")
                        .doc("A picture shown until playing starts."),
                    prop("aspect_ratio", "Option<String>")
                        .default("None")
                        .doc("The picture's CSS `aspect-ratio`, such as `\"16 / 9\"`, so the box holds its shape before the file loads. Unset, the file's own."),
                    prop("tracks", "Vec<MediaTrack>")
                        .default("[]")
                        .doc("WebVTT files: `src`, `kind` (`Captions`, `Subtitles`, `Descriptions`, `Chapters`), `srclang`, `label`, `default`. A captions or subtitles track adds the captions button."),
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
                        .doc("How much loads before a press: `none`, `metadata` or `auto`."),
                    prop("size", "Size").default("theme").doc("Of the buttons and sliders."),
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
                .parts("VideoPart", vec![
                    (VideoPart::Media, "The `<video>` element."),
                    (VideoPart::Controls, "The row of controls, a `Toolbar`."),
                    (VideoPart::Time, "The elapsed and total time."),
                    (VideoPart::Seek, "The seek slider's wrapper."),
                    (VideoPart::Volume, "The volume slider's wrapper."),
                    (VideoPart::Message, "The error text, or the fallback where nothing plays media."),
                ]),
            ],
            accessibility: a11y()
                .key(["K"], "Plays or pauses, with focus anywhere in the player.")
                .key(["Space"], "On a slider: plays or pauses. On a button: presses it.")
                .key(["J", "L"], "Jumps 10 seconds back or ahead.")
                .key(["M"], "Mutes or unmutes.")
                .key(["C"], "Shows or hides the captions, with a captions or subtitles track.")
                .key(["F"], "Enters or leaves fullscreen.")
                .key(["Escape"], "Leaves fullscreen.")
                .key(["Left", "Right"], "On the seek slider: 1 second; on the volume slider: 5%.")
                .handles([
                    "The player is a `group` named by `label`; its buttons are one `toolbar` stop, each slider its own.",
                    "The play, mute and fullscreen buttons change their names (Play/Pause, Mute/Unmute, Fullscreen/Exit fullscreen); the captions button uses `aria-pressed`.",
                    "Where the page may not go fullscreen, the player covers the window as a fixed box instead, which Escape, F and a Tab out of the player leave, so focus never hides behind it.",
                    "In fullscreen the controls overlay the bottom of the picture and fade after 3 seconds of playing untouched. A pointer move, a tap or a key brings them back. After a key they stay until the next click or tap, so keyboard focus never sits on a faded control; they never leave the Tab order, and a tap on the picture only shows them.",
                    "Below 28rem the volume slider hides and the mute button stays; below 22rem the total time hides too, so the row fits at 320px (WCAG 1.4.10).",
                    "The seek slider's `aria-valuetext` reads \"1:05 of 4:56\" (the localization's `media.position`).",
                    "A polite status says \"Loading\" while playing waits for data; a failed source shows an alert.",
                    "No autoplay unless asked, and a debug warning for autoplay with sound (WCAG 1.4.2).",
                ])
                .must([
                    "Give each player a `label` that says what plays.",
                    "Add a captions track for speech (WCAG 1.2.2), and a described version or a transcript for what only the picture shows (WCAG 1.2.3, 1.2.5): libero cannot write them.",
                    "Do not let a clip flash more than three times a second (WCAG 2.3.1).",
                ])
                .limits([
                    "Blitz plays no media: the controls give way to `children`, by default a link to the file.",
                    "On a WebView each command and state change crosses the IPC, so the time trails by a moment.",
                    "In fullscreen the shown controls cover the bottom of the picture, captions included.",
                    "The demo's VP9 WebM may not play in older Safari: give `sources` a WebM and keep an MP4 (H.264) as `src`.",
                ]),
            lead: rsx! {
                Text {
                    "A "
                    Code { source: "<video>" }
                    " with libero's own controls: play, seek, time, mute, volume, captions and fullscreen, the same row as "
                    Code { source: "Audio" }
                    "'s, in the theme's look on every platform that plays media. "
                    Code { source: "use_media()" }
                    " drives the same engine for a layout of your own."
                }
            },
            Demo {
                component: "Video",
                children_text: "",
                fixed: vec![
                    format!("src: {:?}", crate::site::SAMPLE_VIDEO),
                    r#"label: "Big Buck Bunny""#.to_string(),
                    format!("poster: {:?}", crate::site::SAMPLE_POSTER),
                    r#"aspect_ratio: "16 / 9""#.to_string(),
                ],
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl"]).default("md"),
                    Control::switch("muted"),
                    Control::switch("looping"),
                ],
                render: move |values: DemoValues| rsx! {
                    Video {
                        src: crate::site::SAMPLE_VIDEO,
                        label: "Big Buck Bunny",
                        poster: crate::site::SAMPLE_POSTER,
                        aspect_ratio: "16 / 9",
                        size: Input::from(Size::from(values.str("size").as_str())),
                        muted: values.str("muted") == "true",
                        looping: values.str("looping") == "true",
                    }
                },
            }
            DocSection {
                title: "Several formats",
                Text {
                    "A browser plays the first "
                    Code { source: "sources" }
                    " entry whose type it supports, then "
                    Code { source: "src" }
                    ". WebM first and an MP4 as "
                    Code { source: "src" }
                    " reach every browser, Safari included."
                }
                CodeBlock { source: FORMATS, language: "rust" }
            }
        }
    }
}
