use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, a11y, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Code, CodeBlock, Input, MediaTrack, Text, TrackKind, Video, VideoPart},
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
                        .default("16 / 9")
                        .doc("The picture's CSS `aspect-ratio`, which the box holds before the file loads, poster or not. A portrait clip wants `\"9 / 16\"`; `\"auto\"` follows the file, and the box jumps as it loads. A picture of another shape is letterboxed."),
                    prop("tracks", "Vec<MediaTrack>")
                        .default("[]")
                        .doc("WebVTT files: `src`, `kind` (`Captions`, `Subtitles`, `Descriptions`, `Chapters`), `srclang`, `label`, `default`. A captions or subtitles track enables the captions button."),
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
                    (VideoPart::Controls, "The bar of controls over the bottom of the picture, a named `group`."),
                    (VideoPart::Time, "The elapsed and total time."),
                    (VideoPart::Seek, "The seek slider's wrapper."),
                    (VideoPart::Volume, "The mute button and the volume menu's trigger."),
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
                .key(["Shift+?"], "Lists these keys in a `ShortcutHelp` dialog, inside the player in fullscreen.")
                .key(["Escape"], "Closes the speed or volume menu and returns to its button; else leaves fullscreen.")
                .key(["Left", "Right"], "On the seek slider: 1 second; on the volume slider: 5%.")
                .key(["Tab"], "Moves through every control in visual order: seek, play, mute, volume, captions, speed, fullscreen. In the volume menu, returns to its button.")
                .handles([
                    "The player is a `group` named by `label`, its controls a `group` named \"Player controls\"; every button and slider is its own Tab stop, as in the browser's own controls.",
                    "The play, mute and fullscreen buttons change their names (Play/Pause, Mute/Unmute, Fullscreen/Exit fullscreen); the captions button uses `aria-pressed`, and a bar under its icon shows it pressed.",
                    "In fullscreen the speed and volume menus and the tooltips open inside the player, so they show over the fullscreen picture.",
                    "A press anywhere on the seek track jumps there; a drag scrubs.",
                    "The speed button shows the rate (\"1×\", \"1,5×\" under `Formats::GERMAN`) and is named \"Playback speed 1×\"; its menu offers 0.5× to 2× as radio items.",
                    "Without a captions or subtitles track the captions button stays, disabled but focusable, and says \"No captions for this video\".",
                    "The speaker button mutes and unmutes. The chevron beside it, named \"Volume\", opens a `dialog` holding the volume slider and focuses it, as `Audio`'s.",
                    "At volume 0 the mute button offers Unmute, which brings back the last audible volume; moving the volume up while muted unmutes.",
                    "Where the page may not go fullscreen, the player covers the window as a fixed box instead, which Escape, F and a Tab out of the player leave, so focus never hides behind it. Focus stays on the control that was pressed, inside the box, and is there again when the box closes.",
                    "The controls overlay the bottom of the picture, as YouTube's, and fade after 3 seconds of playing untouched or as the mouse leaves the player; while paused they stay. A pointer move, a tap or a key brings them back. After a key they stay until the next click or tap, so keyboard focus never sits on a faded control, and focus moved into them by code or a screen reader shows them too. They never leave the Tab order.",
                    "A click on the picture plays or pauses, as YouTube's. A tap on it while the controls are faded only shows them; once shown, a tap plays or pauses.",
                    "No control leaves the player at any width: the seek track has a row of its own and shrinks with the player, below 22rem the total time goes, and below 15rem the bar moves under the picture and wraps, so it fits at 320px and 200% zoom (WCAG 1.4.10).",
                    "A black scrim under the bar keeps its white text at 4.5:1 and its icons, tracks and thumbs at 3:1 over any picture, a white one included, in light and dark (WCAG 1.4.3, 1.4.11).",
                    "The seek slider's `aria-valuetext` reads \"1:05 of 4:56\" (the localization's `media.position`).",
                    "A polite status says \"Loading\" while playing waits for data; a failed source shows an alert.",
                    "No autoplay unless asked, and a debug warning for autoplay with sound (WCAG 1.4.2).",
                ])
                .must([
                    "Give each player a `label` that says what plays.",
                    "Add a captions track for speech (WCAG 1.2.2), and a described version or a transcript for what only the picture shows (WCAG 1.2.3, 1.2.5): libero cannot write them.",
                    "Do not let a clip flash more than three times a second (WCAG 2.3.1).",
                    "In a parent that shrink-wraps its content, such as a flex column, give the parent `max-width: 100%` or `min-width: 0`: an unsized player asks for 40rem and would push it past a narrow screen (WCAG 1.4.10).",
                ])
                .limits([
                    "Blitz plays no media: the controls give way to `children`, by default a link to the file.",
                    "On a WebView each command and state change crosses the IPC, so the time trails by a moment.",
                    "The shown controls cover the bottom of the picture. Chromium-based browsers draw the captions above them, through the WebKit captions box Safari shares; Firefox offers no hook to move them, so there the bar covers them until it fades.",
                    "Captions show once playing starts: browsers draw none over the poster.",
                    "Fullscreen is libero's own `use_fullscreen` over the browser's Fullscreen API, no library; where the API is refused, a fixed box over the window. A floating mini player is the browser's own Picture-in-Picture (Firefox's button on the video, Chrome's context menu): its window is outside the page, so focus cannot follow it.",
                    "The demo's VP9 WebM may not play in older Safari: give `sources` a WebM and keep an MP4 (H.264) as `src`.",
                ]),
            lead: rsx! {
                Text {
                    "A "
                    Code { source: "<video>" }
                    " with libero's own controls: play, seek, time, mute, volume, speed, captions and fullscreen, in a bar over the picture as YouTube's, in the theme's look on every platform that plays media. "
                    Code { source: "use_media()" }
                    " drives the same engine for a layout of your own."
                }
                Text {
                    "The demo plays \"Big Buck Bunny\", (c) 2008 Blender Foundation, "
                    Anchor { to: "https://peach.blender.org", target: "_blank", "peach.blender.org" }
                    ", under CC BY 3.0, streamed from Wikimedia Commons, so it plays only online. "
                    "Its English track is one placeholder caption, not the film's sound."
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
                    format!(
                        r#"tracks: vec![MediaTrack {{ src: {:?}.into(), kind: TrackKind::Captions, srclang: "en".into(), label: "English".into(), default: true }}]"#,
                        crate::site::SAMPLE_CAPTIONS,
                    ),
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
                        Video {
                            key: "{muted}",
                            src: crate::site::SAMPLE_VIDEO,
                            label: "Big Buck Bunny",
                            poster: crate::site::SAMPLE_POSTER,
                            aspect_ratio: "16 / 9",
                            tracks: vec![MediaTrack {
                                src: crate::site::SAMPLE_CAPTIONS.into(),
                                kind: TrackKind::Captions,
                                srclang: "en".into(),
                                label: "English".into(),
                                default: true,
                            }],
                            size: Input::from(Size::from(values.str("size").as_str())),
                            muted: muted == "true",
                            looping: values.str("looping") == "true",
                        }
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
